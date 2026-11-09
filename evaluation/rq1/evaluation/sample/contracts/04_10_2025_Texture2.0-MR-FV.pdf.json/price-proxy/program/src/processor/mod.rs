mod utils;

use anchor_lang::AccountDeserialize;
use borsh::BorshDeserialize;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2, VerificationLevel};
use solana_program::account_info::AccountInfo;
use solana_program::borsh1::try_from_slice_unchecked;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use super_lendy::state::reserve::Reserve;
use switchboard_solana::{AggregatorAccountData, Key};

use texture_common::account::PodAccount;
use texture_common::math::{CheckedDiv, CheckedMul, Decimal};
use texture_common::remote::system::SystemProgram;
use texture_common::utils::verify_key;

use crate::error::PriceProxyError;
use crate::error::PriceProxyError::OperationCanNotBePerformed;
use crate::instruction::{
    AlterPriceFeedAccounts, CreatePriceFeedAccounts, DeletePriceFeedAccounts,
    PriceProxyInstruction, UpdatePriceAccounts, WritePriceAccounts,
};
use crate::processor::utils::transfer_lamports;
use crate::state::price_feed::{
    FeedType, PriceFeed, PriceFeedParams, PriceFeedSource, WormholeVerificationLevel,
};
use crate::state::stake_pool::StakePool;
use crate::PriceProxyResult;

pub struct Processor<'a, 'b> {
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'b>],
}

impl<'a, 'b> Processor<'a, 'b> {
    pub fn new(program_id: &'a Pubkey, accounts: &'a [AccountInfo<'b>]) -> Self {
        Self {
            program_id,
            accounts,
        }
    }

    pub fn process_instruction(self, input: &[u8]) -> PriceProxyResult<()> {
        match PriceProxyInstruction::try_from_slice(input).map_err(PriceProxyError::from)? {
            PriceProxyInstruction::CreatePriceFeed { params } => self.create_price_feed(params),
            PriceProxyInstruction::WritePrice {
                price,
                price_timestamp,
            } => self.write_price(price, price_timestamp),
            PriceProxyInstruction::UpdatePrice { maximum_age_sec } => {
                self.update_price(maximum_age_sec)
            }
            PriceProxyInstruction::AlterPriceFeed { params } => self.alter_price_feed(params),
            PriceProxyInstruction::DeletePriceFeed {} => self.delete_price_feed(),
            PriceProxyInstruction::Version => self.version(),
        }
    }

    #[inline(never)]
    pub(super) fn create_price_feed(self, params: PriceFeedParams) -> PriceProxyResult<()> {
        msg!("create_price_feed ix: {:?}", params);

        let CreatePriceFeedAccounts {
            price_feed,
            authority,
            source_address,
            transform_source_address,
            system_program,
        } = CreatePriceFeedAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let rent = Rent::get().expect("No Rent");

        SystemProgram::new(system_program)
            .create_account(
                authority,
                price_feed,
                PriceFeed::SIZE as u64,
                rent.minimum_balance(PriceFeed::SIZE),
                self.program_id,
            )
            .call()?;

        let mut price_feed_data = price_feed.data.borrow_mut();
        PriceFeed::init_bytes(
            price_feed_data.as_mut(),
            (
                params,
                *authority.key,
                *source_address.key,
                *transform_source_address.key,
            ),
        )?;

        Ok(())
    }

    fn write_price(&self, price: Decimal, price_timestamp: i64) -> Result<(), PriceProxyError> {
        msg!("write_price ix: {}", price);

        let WritePriceAccounts {
            price_feed: price_feed_info,
            authority: authority_info,
        } = WritePriceAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut price_feed_data = price_feed_info.data.borrow_mut();
        let price_feed = PriceFeed::try_from_bytes_mut(&mut price_feed_data)?;

        verify_source(price_feed.source(), PriceFeedSource::OffChain)?;

        verify_key(
            authority_info.key,
            &price_feed.source_address,
            "source authority",
        )?;

        // NOTE: Strictly less so that we can update the timestamp
        // for the current price by simply setting the price and timestamp from account data
        if price_timestamp < price_feed.update_timestamp {
            return Err(PriceProxyError::TimestampIsNotRecent);
        }

        let clock = Clock::get().expect("clock");
        price_feed.try_set_price(price, price_timestamp, clock.slot)?;

        Ok(())
    }

    fn update_price(&self, maximum_age_sec: u64) -> Result<(), PriceProxyError> {
        msg!("update_price ix");

        let UpdatePriceAccounts {
            price_feed,
            source_address,
            transform_source_address,
        } = UpdatePriceAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut price_feed_data = price_feed.data.borrow_mut();
        let unpacked_price_feed = PriceFeed::try_from_bytes_mut(&mut price_feed_data)?;

        if unpacked_price_feed.feed_type() == FeedType::Transform
            && source_address.key == transform_source_address.key
        {
            msg!("Transform source address must be different from the source address");
            return Err(PriceProxyError::OperationCanNotBePerformed);
        }

        // Update
        let (price, update_ts) = {
            let (price, update_ts) = get_price_from_source_no_older_than(
                unpacked_price_feed,
                source_address,
                maximum_age_sec,
                FeedType::Direct,
            )?;
            if unpacked_price_feed.feed_type() == FeedType::Transform {
                let (second_price, second_update_ts) = get_price_from_source_no_older_than(
                    unpacked_price_feed,
                    transform_source_address,
                    maximum_age_sec,
                    FeedType::Transform,
                )?;
                (
                    price.checked_mul(second_price)?,
                    update_ts.min(second_update_ts),
                )
            } else {
                (price, update_ts)
            }
        };
        let clock = Clock::get().expect("clock");
        unpacked_price_feed.try_set_price(price, update_ts, clock.slot)?;

        Ok(())
    }

    fn alter_price_feed(&self, params: PriceFeedParams) -> Result<(), PriceProxyError> {
        msg!("alter_price_feed ix");

        let AlterPriceFeedAccounts {
            price_feed: price_feed_info,
            authority: authority_info,
            source_address,
            transform_source_address,
        } = AlterPriceFeedAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut price_feed_data = price_feed_info.data.borrow_mut();
        let price_feed = PriceFeed::try_from_bytes_mut(&mut price_feed_data)?;

        verify_key(
            authority_info.key,
            &price_feed.update_authority,
            "authority",
        )?;

        let PriceFeedParams {
            feed_type,
            symbol,
            quote_symbol,
            logo_url,
            source,
            verification_level,
            transform_source,
        } = params;

        price_feed.feed_type = feed_type as u8;
        price_feed.symbol = symbol;
        price_feed.quote_symbol = quote_symbol as u8;
        price_feed.logo_url = logo_url;
        price_feed.source_raw = source as u8;
        price_feed.transform_source_raw = transform_source as u8;
        price_feed.source_address = source_address.key();
        price_feed.transform_source_address = transform_source_address.key();
        price_feed.verification_level = verification_level as u8;

        Ok(())
    }

    fn delete_price_feed(&self) -> Result<(), PriceProxyError> {
        msg!("delete_price_feed ix");

        let DeletePriceFeedAccounts {
            price_feed: price_feed_info,
            authority: authority_info,
        } = DeletePriceFeedAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut price_feed_data = price_feed_info.data.borrow_mut();
        let price_feed = PriceFeed::try_from_bytes_mut(&mut price_feed_data)?;

        verify_key(
            authority_info.key,
            &price_feed.update_authority,
            "authority",
        )?;

        let balance = {
            let lamports_data = price_feed_info.lamports.borrow();
            **lamports_data
        };

        transfer_lamports(price_feed_info, authority_info, balance)?;

        Ok(())
    }

    #[inline(never)]
    pub(super) fn version(&self) -> Result<(), PriceProxyError> {
        msg!(
            "PriceProxy contract {}",
            env!("CARGO_PKG_VERSION").to_string()
        );
        Err(OperationCanNotBePerformed)
    }
}

pub(crate) fn get_price_from_source_no_older_than(
    price_feed: &mut PriceFeed,
    source_address: &AccountInfo<'_>,
    maximum_age_sec: u64,
    feed_type: FeedType,
) -> Result<(Decimal, i64), PriceProxyError> {
    let (source, expected_source_address) = if feed_type == FeedType::Direct {
        (price_feed.source(), price_feed.source_address)
    } else {
        (
            price_feed.transform_source(),
            price_feed.transform_source_address,
        )
    };

    let mut source_data: &[u8] = &source_address.data.try_borrow_mut().expect("borrow mut");

    // Update
    let (price, update_ts) = match source {
        PriceFeedSource::Pyth => {
            let price_update = PriceUpdateV2::try_deserialize_unchecked(&mut source_data)
                .expect("deserialize Pyth source address");

            // Check that PriceFeed's source_address and PriceUpdateV2's feed_id are equal
            verify_key(
                &Pubkey::try_from_slice(price_update.price_message.feed_id.as_slice())
                    .expect("pubkey from feedId"),
                &expected_source_address,
                "source address",
            )?;
            let pyth_price = match price_feed.verification_level() {
                WormholeVerificationLevel::Full => price_update
                    .get_price_no_older_than(
                        &Clock::get().expect("clock"),
                        maximum_age_sec,
                        &price_update.price_message.feed_id,
                    )
                    .expect("get Pyth price"),
                WormholeVerificationLevel::Partial => {
                    // Partially checks the Wormhole guardian signatures.
                    // 5 signatures seems like the best it can currently do.
                    let num_signatures = 5;

                    price_update
                        .get_price_no_older_than_with_custom_verification_level(
                            &Clock::get().expect("clock"),
                            maximum_age_sec,
                            &price_update.price_message.feed_id,
                            VerificationLevel::Partial { num_signatures },
                        )
                        .expect("get Pyth price")
                }
            };

            if pyth_price.price < 0 || pyth_price.exponent > 0 || pyth_price.exponent < -255 {
                msg!("Invalid Pyth price or exponent");
                return Err(PriceProxyError::InvalidPriceOrExpo);
            }
            (
                Decimal::from_i128_with_scale(
                    pyth_price.price as i128,
                    pyth_price.exponent.unsigned_abs(),
                )?,
                pyth_price.publish_time,
            )
        }
        PriceFeedSource::Switchboard => {
            // Check that PriceFeed's source_address and provided source_address are equal
            verify_key(
                source_address.key,
                &expected_source_address,
                "source address",
            )?;
            let data_feed = AggregatorAccountData::new_from_bytes(source_data)
                .expect("deserialize Switchboard feed");

            data_feed
                .check_staleness(Clock::get().unwrap().unix_timestamp, maximum_age_sec as i64)
                .expect("price is stale");

            let feed_result = data_feed.get_result().expect("get Switchboard price");
            (
                Decimal::from_i128_with_scale(feed_result.mantissa, feed_result.scale)?,
                data_feed.latest_confirmed_round.round_open_timestamp,
            )
        }
        PriceFeedSource::SuperLendy => {
            // Check that PriceFeed's source_address and provided source_address are equal
            verify_key(
                source_address.key,
                &expected_source_address,
                "source address",
            )?;
            let reserve_data = Reserve::try_from_bytes(source_data)?;

            let unix_tx = Clock::get().unwrap().unix_timestamp;
            let staleness = (unix_tx - reserve_data.last_update.timestamp) as u64;
            if staleness > maximum_age_sec {
                msg!("Feed has not been updated in {} seconds!", staleness);
                return Err(PriceProxyError::StaleFeed(staleness));
            }
            if reserve_data.last_update.stale > 0 {
                msg!("Reserve is stale and needs to be refreshed prior to get LP market price from it");
                return Err(PriceProxyError::StaleFeed(0));
            }

            (
                reserve_data.lp_market_price().expect("get lp market price"),
                reserve_data.last_update.timestamp,
            )
        }
        PriceFeedSource::StakePool => {
            // Check that PriceFeed's source_address and provided source_address are equal
            verify_key(
                source_address.key,
                &expected_source_address,
                "source address",
            )?;

            let data_feed = try_from_slice_unchecked::<StakePool>(source_data)?;
            let feed_epoch = data_feed.last_update_epoch;
            let clock = Clock::get().unwrap();
            let curr_epoch = clock.epoch;

            if curr_epoch > feed_epoch {
                let staleness = curr_epoch - feed_epoch;
                if staleness > 1 {
                    msg!("Feed has not been updated in {} epoch!", staleness);
                    return Err(PriceProxyError::StaleFeed(staleness));
                }
            }

            let lst_price =
                Decimal::from_i128_with_scale(data_feed.total_lamports as i128, 0)?.checked_div(
                    Decimal::from_i128_with_scale(data_feed.pool_token_supply as i128, 0)?,
                )?;

            (lst_price, clock.unix_timestamp)
        }
        other => {
            msg!(
                "Price source must be Pyth, Switchboard, StakePool or SuperLendy only, given {}",
                other.to_string()
            );
            return Err(PriceProxyError::InvalidSource {
                current: other,
                expected: PriceFeedSource::Unknown,
            });
        }
    };
    Ok((price, update_ts))
}

pub(crate) fn verify_source(
    current: PriceFeedSource,
    expected: PriceFeedSource,
) -> Result<(), PriceProxyError> {
    if current != expected {
        return Err(PriceProxyError::InvalidSource { current, expected });
    }
    Ok(())
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
) -> ProgramResult {
    if program_id != &crate::ID {
        msg!("IX in not for PriceProxy but for {}", program_id);
        return Err(ProgramError::IncorrectProgramId);
    }

    Processor::new(program_id, accounts)
        .process_instruction(instruction_data)
        .map_err(|err| {
            msg!("Error: {}", err);
            err.into()
        })
}
