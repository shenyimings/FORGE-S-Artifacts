use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::{RpcError, RpcResponseErrorData};
use solana_client::rpc_response::RpcSimulateTransactionResult;
use solana_sdk::account::Account;
use solana_sdk::clock::Slot;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;

use texture_common::account::loaders::load_accounts;
use texture_common::account::PodAccount;

use curvy::instruction::{AlterCurve, CreateCurve, DeleteCurve};
use curvy::state::curve::{Curve, CurveParams, CurveX, CurveY, MAX_Y_CNT};

pub async fn load_curves(rpc: &RpcClient) -> Result<(HashMap<Pubkey, Curve>, Slot)> {
    Ok(load_accounts(rpc, &curvy::ID).await?)
}

#[derive(Debug)]
pub struct SignatureView {
    pub signature: Signature,
}

#[derive(Debug)]
#[serde_with::serde_as]
#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize, display_json::DisplayAsJsonPretty)]
pub struct CurveSignatureView {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub curve: Pubkey,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub signature: Option<Signature>,
    pub error: Option<String>,
}
impl CurveSignatureView {
    pub fn success(curve: Pubkey, signature: Signature) -> Self {
        Self {
            curve,
            signature: Some(signature),
            error: None,
        }
    }

    pub fn failure(curve: Pubkey, error: impl ToString) -> Self {
        Self {
            curve,
            signature: None,
            error: Some(error.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct CurveView {
    pub key: Pubkey,
    pub curve: Curve,
}
impl From<(Pubkey, Curve)> for CurveView {
    fn from((key, curve): (Pubkey, Curve)) -> Self {
        Self { key, curve }
    }
}

impl Display for CurveView {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Address : {}", self.key)?;
        writeln!(f, "Name    : {}", String::from_utf8_lossy(&self.curve.name))?;
        writeln!(
            f,
            "Formula : {}",
            String::from_utf8_lossy(&self.curve.formula)
        )?;
        writeln!(f, "decimals: {}", self.curve.decimals)?;
        writeln!(f, "x0      : {}", self.curve.x0)?;
        writeln!(f, "x_step  : {}", self.curve.x_step)?;
        writeln!(f, "y_count : {}", self.curve.y_count)?;
        write!(f, "y[]     : \n          ")?;

        let mut cnt = 0;

        for y_value in self.curve.y.iter().take(self.curve.y_count as usize) {
            write!(f, "{}, ", y_value)?;

            cnt += 1;

            if cnt == 11 {
                write!(f, "\n          ")?;
                cnt = 0;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CurvesView {
    pub curves: Vec<CurveView>,
}

pub struct CurvyClient {
    pub rpc: RpcClient,
    pub authority: Keypair,
    pub priority_fee: Option<u64>,
}

impl CurvyClient {
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

        let signature = self
            .rpc
            .send_and_confirm_transaction_with_spinner(&tx)
            .await
            .map_err(with_logs)?;

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

    pub async fn create_curve(
        &self,
        params: CurveParams,
        priority_rate: Option<u64>,
    ) -> Result<CurveSignatureView> {
        let owner = self.authority.pubkey();

        let curve_keypair = Keypair::new();
        let curve = curve_keypair.pubkey();

        let mut ixs = vec![];

        if let Some(priority_rate) = priority_rate {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_rate);
            ixs.push(priority_fee_ix);
        }

        ixs.push(
            CreateCurve {
                curve,
                owner,
                params,
            }
            .into_instruction(),
        );

        let signature = self
            .send_transaction_by(ixs, &[&self.authority, &curve_keypair])
            .await?;

        Ok(CurveSignatureView::success(curve, signature))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn alter_curve(
        &self,
        curve_key: Pubkey,
        name: Option<String>,
        formula: Option<String>,
        decimals: Option<u8>,
        x0: Option<CurveX>,
        x_step: Option<CurveX>,
        y_count: Option<u8>,
        y: Option<[CurveY; MAX_Y_CNT]>,
        priority_rate: Option<u64>,
    ) -> Result<SignatureView> {
        let owner = self.authority.pubkey();

        let curve_view = self.curve(&curve_key).await.expect("get curve");
        let curve = curve_view.curve;

        let mut params = CurveParams {
            name: curve.name,
            formula: curve.formula,
            x0: curve.x0,
            x_step: curve.x_step,
            y_count: curve.y_count,
            decimals: curve.decimals,
            y: curve.y,
        };

        if let Some(name) = name {
            params.name = curvy::state::utils::str_to_array(&name);
        }

        if let Some(formula) = formula {
            params.formula = curvy::state::utils::str_to_array(&formula);
        }

        if let Some(decimals) = decimals {
            params.decimals = decimals;
        }

        if let Some(x0) = x0 {
            params.x0 = x0;
        }
        if let Some(x_step) = x_step {
            params.x_step = x_step;
        }

        if let Some(y_count) = y_count {
            params.y_count = y_count;
        }

        if let Some(y) = y {
            params.y = y;
        }

        let mut ixs = vec![];

        if let Some(priority_rate) = priority_rate {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_rate);
            ixs.push(priority_fee_ix);
        }

        ixs.push(
            AlterCurve {
                curve: curve_key,
                owner,
                params,
            }
            .into_instruction(),
        );

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(SignatureView { signature })
    }

    pub async fn delete_curve(
        &self,
        curve: Pubkey,
        priority_rate: Option<u64>,
    ) -> Result<SignatureView> {
        let owner = self.authority.pubkey();

        let mut ixs = vec![];

        if let Some(priority_rate) = priority_rate {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_rate);
            ixs.push(priority_fee_ix);
        }

        ixs.push(DeleteCurve { curve, owner }.into_instruction());

        let signature = self.send_transaction_by(ixs, &[&self.authority]).await?;

        Ok(SignatureView { signature })
    }

    pub async fn curve(&self, key: &Pubkey) -> Result<CurveView> {
        self.get_pod_account::<Curve>(key)
            .await
            .map(|(curve, _slot)| (*key, curve))
            .map(Into::into)
    }

    pub async fn curves(&self) -> Result<CurvesView> {
        let curves: Vec<CurveView> = load_curves(&self.rpc)
            .await?
            .0
            .iter()
            .map(|(key, curve)| CurveView::from((*key, *curve)))
            .collect();

        Ok(CurvesView { curves })
    }
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
