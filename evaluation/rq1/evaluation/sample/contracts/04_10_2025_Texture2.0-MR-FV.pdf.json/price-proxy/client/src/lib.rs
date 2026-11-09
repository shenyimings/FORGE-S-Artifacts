use std::collections::HashMap;
use std::time::Duration;

use anchor_lang::Space;
use anyhow::{anyhow, bail, Result};
use derive_more::From;
use futures::future::join_all;
use futures::TryFutureExt;
use hex::ToHex;
use pyth_solana_receiver_sdk::pda::{get_config_address, get_treasury_address};
use pythnet_sdk::wire::v1::{AccumulatorUpdateData, Proof};
use serde_wormhole::RawMessage;
use solana_client::client_error::{reqwest, ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::{RpcError, RpcResponseErrorData};
use solana_client::rpc_response::RpcSimulateTransactionResult;
use solana_program::rent::Rent;
use solana_program::{system_instruction, system_program};
use solana_sdk::account::Account;
use solana_sdk::clock::Slot;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;
use super_lendy::instruction::RefreshReserve;
use super_lendy::state::reserve::Reserve;
use texture_common::account::loaders::{load_accounts, load_accounts_by_key};
use texture_common::account::PodAccount;
use texture_common::math::Decimal;
use tracing::debug;
use wormhole_sdk::vaa::{Body, Header};

use price_proxy::instruction::{
    AlterPriceFeed, CreatePriceFeed, DeletePriceFeed, UpdatePrice, Version, WritePrice,
};
use price_proxy::state::price_feed::{
    FeedType, PriceFeed, PriceFeedParams, PriceFeedSource, QuoteSymbol, WormholeVerificationLevel,
};
use price_proxy::state::utils::str_to_array;
use pyth_solana_receiver_interface::instruction::{PostUpdate, PostUpdateAtomic, ReclaimRent};
use pyth_solana_receiver_interface::types::{
    MerklePriceUpdate, PostUpdateAtomicParams, PostUpdateParams,
};
use wormhole_core_bridge_solana_interface::instruction::{
    InitEncodedVaa, VerifyEncodedVaaV1, WriteEncodedVaa,
};
use wormhole_core_bridge_solana_interface::types::WriteEncodedVaaArgs;

mod core_bridge_state;

const PYTH_PRICE_MESSAGE_API_URL: &str = "https://hermes.pyth.network/api/latest_vaas";
/**
 * This constant is used to efficiently pack transactions when writing an encoded Pyth VAA to the Wormhole contract.
 * Posting a VAA requires two transactions. If you split the VAA at this index when writing it, the first transaction will be almost full.
 */
pub const VAA_SPLIT_INDEX: usize = 755;
const VAA_START: usize = 8 // DISCRIMINATOR
    + core_bridge_state::Header::INIT_SPACE
    + 4 // bytes.len()
;

pub async fn load_price_feeds(rpc: &RpcClient) -> Result<(HashMap<Pubkey, PriceFeed>, Slot)> {
    Ok(load_accounts(rpc, &price_proxy::ID).await?)
}

pub async fn load_price_feeds_by_key(
    rpc: &RpcClient,
    keys: &[Pubkey],
) -> Result<(HashMap<Pubkey, PriceFeed>, Slot)> {
    Ok(load_accounts_by_key(rpc, keys).await?)
}

#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, display_json::DisplayAsJsonPretty, From)]
pub struct SignatureView {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub signature: Signature,
}

#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize, display_json::DisplayAsJsonPretty)]
pub struct PriceFeedSignatureView {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub price_feed: Pubkey,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub signature: Option<Signature>,
    pub error: Option<String>,
}
impl PriceFeedSignatureView {
    pub fn success(price_feed: Pubkey, signature: Signature) -> Self {
        Self {
            price_feed,
            signature: Some(signature),
            error: None,
        }
    }

    pub fn failure(price_feed: Pubkey, error: impl ToString) -> Self {
        Self {
            price_feed,
            signature: None,
            error: Some(error.to_string()),
        }
    }
}

#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, display_json::DisplayAsJsonPretty)]
pub struct PriceFeedView {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub key: Pubkey,
    pub price_feed: PriceFeed,
    pub slot: Slot,
}
impl From<(Pubkey, PriceFeed, Slot)> for PriceFeedView {
    fn from((key, price_feed, slot): (Pubkey, PriceFeed, Slot)) -> Self {
        Self {
            key,
            price_feed,
            slot,
        }
    }
}

#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize, display_json::DisplayAsJsonPretty)]
pub struct PriceFeedsView {
    #[serde_as(as = "Vec<(serde_with::DisplayFromStr, _)>")]
    pub price_feeds: HashMap<Pubkey, PriceFeed>,
    pub slot: Slot,
}
impl From<(HashMap<Pubkey, PriceFeed>, Slot)> for PriceFeedsView {
    fn from((price_feeds, slot): (HashMap<Pubkey, PriceFeed>, Slot)) -> Self {
        Self { price_feeds, slot }
    }
}

pub struct PriceProxyClient {
    pub rpc: RpcClient,
    pub authority: Keypair,
    pub priority_fee: Option<u64>,
    pub show_spinner: bool,
}

impl PriceProxyClient {
    pub fn with_spinner(mut self, with_spinner: bool) -> Self {
        self.show_spinner = with_spinner;
        self
    }

    pub async fn send_transaction_by(
        &self,
        mut ixs: Vec<Instruction>,
        signers: &impl Signers,
    ) -> Result<Signature> {
        if let Some(priority_fee) = self.priority_fee {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
            ixs.push(priority_fee_ix);
        }

        let mut tx = Transaction::new_with_payer(ixs.as_ref(), Some(&self.authority.pubkey()));
        let blockhash = self.rpc.get_latest_blockhash().await?;
        tx.sign(signers, blockhash);

        let signature = if self.show_spinner {
            self.rpc
                .send_and_confirm_transaction_with_spinner(&tx)
                .await
                .map_err(with_logs)?
        } else {
            self.rpc
                .send_and_confirm_transaction(&tx)
                .await
                .map_err(with_logs)?
        };

        Ok(signature)
    }

    pub async fn account_exists(&self, key: &Pubkey) -> Result<bool> {
        match self.rpc.get_account(key).await {
            Ok(_) => Ok(true),
            Err(ClientError {
                kind: ClientErrorKind::RpcError(RpcError::ForUser(msg)),
                ..
            }) if msg.starts_with("AccountNotFound") => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn get_account_with_slot(&self, key: &Pubkey) -> Result<(Account, Slot)> {
        let resp = self
            .rpc
            .get_account_with_commitment(key, self.rpc.commitment())
            .await?;
        let account = resp
            .value
            .ok_or_else(|| RpcError::ForUser(format!("AccountNotFound: pubkey={key}")))?;
        Ok((account, resp.context.slot))
    }

    pub async fn get_pod_account<A: PodAccount>(&self, key: &Pubkey) -> Result<(A, Slot)> {
        let (account, slot) = self.get_account_with_slot(key).await?;
        Ok((*A::try_from_bytes(&account.data)?, slot))
    }

    pub async fn create_price_feed(
        &self,
        params: PriceFeedParams,
        source_address: Pubkey,
        transform_source_address: Pubkey,
    ) -> Result<PriceFeedSignatureView> {
        let authority = self.authority.pubkey();

        let price_feed_keypair = Keypair::new();
        let price_feed = price_feed_keypair.pubkey();

        let ixs = vec![CreatePriceFeed {
            price_feed,
            authority,
            source_address,
            transform_source_address,
            params,
        }
        .into_instruction()];

        let signature = self
            .send_transaction_by(ixs, &[&self.authority, &price_feed_keypair])
            .await?;

        Ok(PriceFeedSignatureView::success(price_feed, signature))
    }

    pub async fn write_encoded_vaa_ix(
        &self,
        message: &String,
        encoded_vaa: Pubkey,
    ) -> Vec<Instruction> {
        let authority = self.authority.pubkey();
        let wormhole_address = wormhole_core_bridge_solana_interface::ID;

        let payload_bytes = base64::decode(message).expect("decode");
        let (vaa, _) = deserialize_accumulator_update_data(payload_bytes)
            .expect("deserialize_accumulator_update_data");

        let encoded_vaa_size: usize = vaa.len() + VAA_START;

        let create_encoded_vaa = system_instruction::create_account(
            &authority,
            &encoded_vaa,
            Rent::default().minimum_balance(encoded_vaa_size),
            encoded_vaa_size as u64,
            &wormhole_address,
        );

        let init_encoded_vaa_ix = InitEncodedVaa {
            program_id: wormhole_address,
            write_authority: authority,
            encoded_vaa,
            trailing_accounts: vec![],
        }
        .into_instruction();

        let write_encoded_vaa_ix = WriteEncodedVaa {
            program_id: wormhole_address,
            write_authority: authority,
            draft_vaa: encoded_vaa,
            trailing_accounts: vec![],
            args: WriteEncodedVaaArgs {
                index: 0,
                data: vaa[..VAA_SPLIT_INDEX].to_vec(),
            },
        }
        .into_instruction();

        vec![
            create_encoded_vaa,
            init_encoded_vaa_ix,
            write_encoded_vaa_ix,
        ]
    }

    pub async fn post_update_ix(
        &self,
        price_update: Pubkey,
        message: &String,
        encoded_vaa: Pubkey,
    ) -> Vec<Instruction> {
        let authority = self.authority.pubkey();
        let wormhole_address = wormhole_core_bridge_solana_interface::ID;

        let payload_bytes = base64::decode(message).expect("decode");
        let (vaa, merkle_price_updates) = deserialize_accumulator_update_data(payload_bytes)
            .expect("deserialize_accumulator_update_data");

        let write_encoded_vaa_ix2 = WriteEncodedVaa {
            program_id: wormhole_address,
            write_authority: authority,
            draft_vaa: encoded_vaa,
            trailing_accounts: vec![],
            args: WriteEncodedVaaArgs {
                index: VAA_SPLIT_INDEX.try_into().expect("into vaa split"),
                data: vaa[VAA_SPLIT_INDEX..].to_vec(),
            },
        }
        .into_instruction();

        let (header, _): (Header, Body<&RawMessage>) = serde_wormhole::from_slice(&vaa).unwrap();
        let guardian_set = get_guardian_set_address(header.guardian_set_index);

        let request_compute_units_ix = ComputeBudgetInstruction::set_compute_unit_limit(600_000);

        let verify_encoded_vaa_ix = VerifyEncodedVaaV1 {
            program_id: wormhole_address,
            write_authority: authority,
            draft_vaa: encoded_vaa,
            guardian_set,
            trailing_accounts: vec![],
        }
        .into_instruction();

        let merkle_price_update = merkle_price_updates.first().unwrap();
        let config = get_config_address();
        let treasury = get_treasury_address(0);

        // Create list of hashes that form a proof for membership in a tree.
        let hashes = merkle_price_update
            .clone()
            .proof
            .to_bytes()
            .chunks(20)
            .map(|x| x.to_vec())
            .collect::<Vec<_>>();
        let mut proof = vec![];
        for hash in hashes.into_iter() {
            let mut buf = [0u8; 20];
            buf.copy_from_slice(&hash.as_slice()[0..20]);
            proof.push(buf);
        }
        let merkle_price_update = MerklePriceUpdate {
            message: Vec::from(merkle_price_update.clone().message),
            proof,
        };

        // Create PostUpdate ix
        let params = PostUpdateParams {
            merkle_price_update,
            treasury_id: 0,
        };
        let post_update_ix = PostUpdate {
            program_id: pyth_solana_receiver_sdk::id(),
            payer: authority,
            encoded_vaa,
            config,
            treasury,
            price_update_account: price_update,
            system_program: system_program::id(),
            write_authority: authority,
            trailing_accounts: vec![],
            params,
        }
        .into_instruction();

        vec![
            request_compute_units_ix,
            write_encoded_vaa_ix2,
            verify_encoded_vaa_ix,
            post_update_ix,
        ]
    }

    pub async fn post_update_atomic_ix(
        &self,
        price_update: Pubkey,
        message: &String,
    ) -> Vec<Instruction> {
        let authority = self.authority.pubkey();

        let payload_bytes = base64::decode(message).expect("decode");
        let (vaa, merkle_price_updates) = deserialize_accumulator_update_data(payload_bytes)
            .expect("deserialize_accumulator_update_data");
        let (mut header, body): (Header, Body<&RawMessage>) =
            serde_wormhole::from_slice(&vaa).unwrap();

        // Partially checks the Wormhole guardian signatures.
        // 5 signatures seems like the best it can currently do.
        let num_signatures = 5;
        trim_signatures(&mut header, num_signatures);

        let vaa = serde_wormhole::to_vec(&(header.clone(), body)).unwrap();
        let merkle_price_update = merkle_price_updates.first().unwrap();
        let guardian_set = get_guardian_set_address(header.guardian_set_index);
        let config = get_config_address();
        let treasury = get_treasury_address(0);

        // Create list of hashes that form a proof for membership in a tree.
        let hashes = merkle_price_update
            .clone()
            .proof
            .to_bytes()
            .chunks(20)
            .map(|x| x.to_vec())
            .collect::<Vec<_>>();
        let mut proof = vec![];
        for hash in hashes.into_iter() {
            let mut buf = [0u8; 20];
            buf.copy_from_slice(&hash.as_slice()[0..20]);
            proof.push(buf);
        }
        let merkle_price_update = MerklePriceUpdate {
            message: Vec::from(merkle_price_update.clone().message),
            proof,
        };

        // Create PostUpdateAtomic ix
        let params = PostUpdateAtomicParams {
            vaa,
            merkle_price_update,
            treasury_id: 0,
        };
        let post_update_ix = PostUpdateAtomic {
            program_id: pyth_solana_receiver_sdk::id(),
            payer: authority,
            guardian_set,
            config,
            treasury,
            price_update_account: price_update,
            system_program: system_program::id(),
            write_authority: authority,
            trailing_accounts: vec![],
            params,
        }
        .into_instruction();

        vec![post_update_ix]
    }

    pub async fn post_update(
        &self,
        message: &String,
        verification_level: WormholeVerificationLevel,
    ) -> Result<(
        /*tmp Pyth price account to be updated*/ Pubkey,
        /* tx signature */ Vec<Signature>,
    )> {
        let price_update_keypair = Keypair::new();
        let price_update = price_update_keypair.pubkey();

        let mut signatures = Vec::new();

        match verification_level {
            WormholeVerificationLevel::Full => {
                let encoded_vaa_keypair = Keypair::new();
                let encoded_vaa = encoded_vaa_keypair.pubkey();

                let ixs = self.write_encoded_vaa_ix(message, encoded_vaa).await;

                // 1st transaction with WriteEncodedVaa ix
                let sig = self
                    .send_transaction_by(ixs, &[&self.authority, &encoded_vaa_keypair])
                    .await?;

                signatures.push(sig);

                let ixs = self
                    .post_update_ix(price_update, message, encoded_vaa)
                    .await;

                // 2nd transaction with PostUpdate ix
                let sig = self
                    .send_transaction_by(ixs, &[&self.authority, &price_update_keypair])
                    .await?;
                signatures.push(sig);
            }
            WormholeVerificationLevel::Partial => {
                let ixs = self.post_update_atomic_ix(price_update, message).await;

                let sig = self
                    .send_transaction_by(ixs, &[&self.authority, &price_update_keypair])
                    .await?;
                signatures.push(sig);
            }
        }

        Ok((price_update, signatures))
    }

    pub async fn update_price_ix(
        &self,
        price_feed: Pubkey,
        source_address: Pubkey,
        transform_source_address: Pubkey,
        maximum_age_sec: u64,
    ) -> Vec<Instruction> {
        let update_price_ix = UpdatePrice {
            price_feed,
            source_address,
            transform_source_address,
            maximum_age_sec,
        }
        .into_instruction();
        vec![update_price_ix]
    }

    pub async fn update_price(
        &self,
        price_feed: Pubkey,
        source_address: Pubkey,
        transform_source_address: Pubkey,
        maximum_age_sec: u64,
    ) -> Result<SignatureView> {
        let ixs = self
            .update_price_ix(
                price_feed,
                source_address,
                transform_source_address,
                maximum_age_sec,
            )
            .await;

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(signature.into())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn alter_price_feed(
        &self,
        price_feed: Pubkey,
        feed_type: Option<FeedType>,
        symbol: Option<String>,
        quote_symbol: Option<QuoteSymbol>,
        verification_level: Option<WormholeVerificationLevel>,
        logo_url: Option<String>,
        source: Option<PriceFeedSource>,
        transform_source: Option<PriceFeedSource>,
        source_address: Option<Pubkey>,
        transform_source_address: Option<Pubkey>,
    ) -> Result<SignatureView> {
        let authority = self.authority.pubkey();

        let price_feed_view = self.price_feed(&price_feed).await.expect("get price-feed");
        let price_feed_acc = price_feed_view.price_feed;

        let mut params = PriceFeedParams {
            feed_type: price_feed_acc.feed_type(),
            symbol: price_feed_acc.symbol,
            quote_symbol: price_feed_acc.quote_symbol(),
            verification_level: price_feed_acc.verification_level(),
            logo_url: price_feed_acc.logo_url,
            source: price_feed_acc.source(),
            transform_source: price_feed_acc.transform_source(),
        };

        if let Some(symbol) = symbol {
            params.symbol = str_to_array(symbol.as_str());
        }

        if let Some(feed_type) = feed_type {
            params.feed_type = feed_type;
        }

        if let Some(quote_symbol) = quote_symbol {
            params.quote_symbol = quote_symbol;
        }

        if let Some(verification_level) = verification_level {
            params.verification_level = verification_level;
        }

        if let Some(logo_url) = logo_url {
            params.logo_url = str_to_array(logo_url.as_str());
        }

        if let Some(source) = source {
            params.source = source;
        }

        if let Some(transform_source) = transform_source {
            params.transform_source = transform_source;
        }

        let mut ixs = vec![];

        ixs.push(
            AlterPriceFeed {
                price_feed,
                authority,
                source_address: if let Some(new_source_address) = source_address {
                    new_source_address
                } else {
                    price_feed_acc.source_address
                },
                transform_source_address: if let Some(new_transform_source_address) =
                    transform_source_address
                {
                    new_transform_source_address
                } else {
                    price_feed_acc.transform_source_address
                },
                params,
            }
            .into_instruction(),
        );

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(signature.into())
    }

    pub async fn delete_price_feed(&self, price_feed: Pubkey) -> Result<SignatureView> {
        let authority = self.authority.pubkey();

        let ixs = vec![DeletePriceFeed {
            price_feed,
            authority,
        }
        .into_instruction()];

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(signature.into())
    }

    pub async fn close_price_update_ix(&self, price_update: Pubkey) -> Vec<Instruction> {
        let authority = self.authority.pubkey();
        let reclaim_rent_ix = ReclaimRent {
            program_id: pyth_solana_receiver_sdk::id(),
            payer: authority,
            price_update_account: price_update,
            trailing_accounts: vec![],
        }
        .into_instruction();
        vec![reclaim_rent_ix]
    }

    pub async fn close_price_update(&self, price_update: Pubkey) -> Result<SignatureView> {
        let ixs = self.close_price_update_ix(price_update).await;

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(signature.into())
    }

    pub async fn get_message_by_hex(
        &self,
        hex: &str,
        pyth_api_url: Option<String>,
    ) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        let query = vec![("ids[]", hex)];

        let result = client
            .get(pyth_api_url.unwrap_or(PYTH_PRICE_MESSAGE_API_URL.to_string()))
            .query(&query)
            .send()
            .await
            .expect("get pyth prices")
            .json::<Vec<String>>()
            .await
            .expect("parse json schema");
        Ok(result)
    }

    pub async fn price_feed(&self, key: &Pubkey) -> Result<PriceFeedView> {
        self.get_pod_account::<PriceFeed>(key)
            .await
            .map(|(price_feed, slot)| (*key, price_feed, slot))
            .map(Into::into)
    }

    pub async fn price_feeds(&self) -> Result<PriceFeedsView> {
        load_price_feeds(&self.rpc).await.map(Into::into)
    }

    pub async fn price_feeds_by_key(&self, keys: &[Pubkey]) -> Result<PriceFeedsView> {
        load_price_feeds_by_key(&self.rpc, keys)
            .await
            .map(Into::into)
    }

    pub async fn write_price_ix(
        &self,
        price_feed: Pubkey,
        price: impl Into<Decimal>,
        price_timestamp: i64,
    ) -> Vec<Instruction> {
        let authority = self.authority.pubkey();
        vec![WritePrice {
            price_feed,
            authority,
            price: price.into(),
            price_timestamp,
        }
        .into_instruction()]
    }

    pub async fn write_price(
        &self,
        price_feed: Pubkey,
        price: impl Into<Decimal>,
        price_timestamp: i64,
    ) -> Result<SignatureView> {
        let ixs = self
            .write_price_ix(price_feed, price, price_timestamp)
            .await;

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(signature.into())
    }

    pub async fn force_price_feed_timestamps(
        &self,
        keys: &[Pubkey],
    ) -> Result<Vec<PriceFeedSignatureView>> {
        let PriceFeedsView {
            price_feeds,
            slot: _,
        } = self
            .price_feeds_by_key(keys)
            .await
            .map_err(|err| anyhow!("get price feeds: {err}"))?;

        let writes = price_feeds.into_iter().map(move |(key, feed)| async move {
            if feed.source() != PriceFeedSource::OffChain {
                return PriceFeedSignatureView::failure(
                    key,
                    format!(
                        "price feed source must be 'OffChain' (current '{}')",
                        feed.source()
                    ),
                );
            }
            self.write_price(
                key,
                feed.try_price().unwrap(),
                chrono::Utc::now().timestamp(),
            )
            .map_ok_or_else(
                move |err| PriceFeedSignatureView::failure(key, err),
                move |view| PriceFeedSignatureView::success(key, view.signature),
            )
            .await
        });
        Ok(join_all(writes).await)
    }

    pub async fn contract_version(&self) {
        let ix = Version {}.into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Read logs below to see the contract version");
    }

    /// Constructs and send TXes to update given `price_feed_key`.
    /// This function does NOT process off-chain type feeds.
    /// `maximum_age_sec` maximum price age in seconds which is acceptable to put in to PriceFeed. If
    /// actual price age is greater than TX will fail and no update will happen.
    /// `pyth_api_url` - caller provided override for Pyth off chain API.
    ///
    /// TODO: Only Transform feeds with Transform source = Pyth are supported!
    pub async fn holistic_update_price(
        &self,
        price_feed_key: &Pubkey,
        maximum_age_sec: u64,
        pyth_api_url: Option<String>,
    ) -> Result<Vec<SignatureView>> {
        let mut signatures = Vec::new();
        let price_feed = self.price_feed(price_feed_key).await?;
        let transform_price_update = if price_feed.price_feed.feed_type() == FeedType::Transform
            && price_feed.price_feed.transform_source() == PriceFeedSource::Pyth
        {
            // 0. Convert feed pubkey to hex
            let hex: String = price_feed.price_feed.transform_source_address.encode_hex();

            // 1. Get a Hermes update from Hermes stable
            let message = self.get_message_by_hex(&hex, pyth_api_url.clone()).await?;

            // 2. Post a Pyth price update onto Solana
            let (price_update, local_signatures) = self
                .post_update(
                    message.first().unwrap(),
                    price_feed.price_feed.verification_level(),
                )
                .await?;
            let sig_views: Vec<SignatureView> = local_signatures
                .into_iter()
                .map(SignatureView::from)
                .collect();
            signatures.extend(sig_views);

            Some(price_update)
        } else {
            None
        };

        let transform_source_address = if let Some(transform_price_update) = transform_price_update
        {
            transform_price_update
        } else {
            price_feed.price_feed.transform_source_address
        };

        let source = price_feed.price_feed.source();

        match source {
            PriceFeedSource::Pyth => {
                // 0. Convert feed pubkey to hex
                let hex: String = price_feed.price_feed.source_address.encode_hex();

                // 1. Get a Hermes update from Hermes stable
                let message = self.get_message_by_hex(&hex, pyth_api_url).await?;

                // 2. Post a Pyth price update onto Solana
                let (price_update, local_signatures) = self
                    .post_update(
                        message.first().unwrap(),
                        price_feed.price_feed.verification_level(),
                    )
                    .await?;

                let sig_views: Vec<SignatureView> = local_signatures
                    .into_iter()
                    .map(SignatureView::from)
                    .collect();
                signatures.extend(sig_views);

                // 3. Update Price-feed
                let signature = self
                    .update_price(
                        *price_feed_key,
                        price_update,
                        transform_source_address,
                        maximum_age_sec,
                    )
                    .await?;
                signatures.push(signature);

                // 4. Close a price update account, recovering the rent.
                let signature = self.close_price_update(price_update).await?;

                signatures.push(signature);
            }

            PriceFeedSource::SuperLendy
            | PriceFeedSource::Switchboard
            | PriceFeedSource::StakePool => {
                if source == PriceFeedSource::SuperLendy {
                    // The algorithm:
                    // 1. Refresh Reserve0 which is the source of LP tokens (used as liquidity in Reserve1 -
                    //    reserve of our final interest), also refresh its marke price feed before refreshing the Reserve
                    // 2. Call PriceProxy::UpdatePrice for the price feed used in Reserve1. This action will use
                    //    LP token price, calculated by SuperLendy on step 1.
                    let reserve_data = self
                        .rpc
                        .get_account_data(&price_feed.price_feed.source_address)
                        .await
                        .map_err(|err| anyhow!("getting Reserve account: {}", err))?;
                    let unpacked_reserve = Reserve::try_from_bytes(&reserve_data)
                        .map_err(|err| anyhow!("unpacking Reserve: {}", err))?;

                    let second_level_price_feed = self
                        .price_feed(&unpacked_reserve.config.market_price_feed)
                        .await?;

                    if second_level_price_feed.price_feed.source() != PriceFeedSource::OffChain {
                        let _ = self
                            .update_price(
                                unpacked_reserve.config.market_price_feed,
                                second_level_price_feed.price_feed.source_address,
                                second_level_price_feed.price_feed.transform_source_address,
                                maximum_age_sec,
                            )
                            .await?;
                        debug!(
                            "Updated second level price feed {}",
                            unpacked_reserve.config.market_price_feed
                        );
                    }

                    let ixs = vec![RefreshReserve {
                        reserve: price_feed.price_feed.source_address,
                        market_price_feed: unpacked_reserve.config.market_price_feed,
                        irm: unpacked_reserve.config.irm,
                    }
                    .into_instruction()];

                    let _ = self
                        .send_transaction_by(ixs, &[&self.authority])
                        .await
                        .map_err(|err| anyhow!("Sending TX: {}", err))?;
                    debug!(
                        "Updated second level reserve {}",
                        price_feed.price_feed.source_address
                    );
                }

                let signature = self
                    .update_price(
                        *price_feed_key,
                        price_feed.price_feed.source_address,
                        transform_source_address,
                        maximum_age_sec,
                    )
                    .await?;

                signatures.push(signature);
            }
            _ => {
                bail!("unsupported source {}", price_feed.price_feed.source());
            }
        }

        if let Some(transform_price_update) = transform_price_update {
            // Close a price update account, recovering the rent.
            let signature = self.close_price_update(transform_price_update).await?;

            signatures.push(signature);
        }

        Ok(signatures)
    }
}

pub async fn get_account_with_retries(rpc: &RpcClient, pubkey: &Pubkey) -> Result<Account> {
    let consecutive_errors = 0;
    loop {
        match rpc.get_account(pubkey).await {
            Ok(acc) => return Ok(acc),
            Err(err) => {
                println!("{}'nd attempt to send tx. {:#?} ", consecutive_errors, err);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

pub fn deserialize_accumulator_update_data(
    accumulator_message: Vec<u8>,
) -> Result<(Vec<u8>, Vec<pythnet_sdk::wire::v1::MerklePriceUpdate>)> {
    let accumulator_update_data =
        AccumulatorUpdateData::try_from_slice(accumulator_message.as_slice()).unwrap();

    match accumulator_update_data.proof {
        Proof::WormholeMerkle { vaa, updates } => return Ok((vaa.as_ref().to_vec(), updates)),
    }
}

fn trim_signatures(header: &mut Header, n_signatures: usize) {
    header.signatures = header.signatures[..(n_signatures)].to_vec();
}

pub const SEED_PREFIX: &[u8] = b"GuardianSet";

pub fn get_guardian_set_address(guardian_set_index: u32) -> Pubkey {
    Pubkey::find_program_address(
        &[SEED_PREFIX, guardian_set_index.to_be_bytes().as_ref()],
        &wormhole_core_bridge_solana_interface::ID,
    )
    .0
}

struct Logs(Vec<String>);

impl std::fmt::Display for Logs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nLogs:")?;

        for (i, log) in self.0.iter().enumerate() {
            writeln!(f, "    {:>3}: {}", i + 1, log)?;
        }
        Ok(())
    }
}
pub fn with_logs(mut error: ClientError) -> anyhow::Error {
    let logs = match error.kind {
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data:
                RpcResponseErrorData::SendTransactionPreflightFailure(RpcSimulateTransactionResult {
                    ref mut logs,
                    ..
                }),
            ..
        }) => logs.take().map(Logs),
        _ => None,
    };

    if let Some(logs) = logs {
        anyhow::Error::from(error).context(logs)
    } else {
        error.into()
    }
}
