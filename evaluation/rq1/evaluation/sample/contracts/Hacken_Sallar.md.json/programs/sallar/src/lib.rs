//! Sallar program

use anchor_lang::{
    err,
    prelude::*,
    program,
    solana_program::{pubkey::Pubkey, sysvar::Sysvar},
};
use anchor_spl::token;

use context::*;

pub mod account;
pub mod context;
pub mod error;
pub mod token_math;
pub mod utils;

const FINAL_STAKING_ACCOUNT_BALANCE_PART_FOR_STAKING_DIVISION_FACTOR: u64 = 1000;

/// set seeds for pda accounts
const MINT_SEED: &str = "sallar";
const BLOCKS_STATE_SEED: &str = "blocks_state";
const DISTRIBUTION_TOP_BLOCK_SEED: &str = "distribution_top_block";
const DISTRIBUTION_BOTTOM_BLOCK_SEED: &str = "distribution_bottom_block";
const FINAL_STAKING_ACCOUNT_SEED: &str = "final_staking";
const FINAL_MINING_ACCOUNT_SEED: &str = "final_mining";

declare_id!("nYcjEnRy1ubJtjoGUhT2RVW6o5H5vkiDaHfCTZqzds6");

/// This program is used to mint and distribute Sallar tokens.
#[program]
pub mod sallar {
    use error::SallarError;
    use token_math::{
        calculate_bottom_bp_with_boost, calculate_bottom_bp_without_boost, calculate_dust_per_bp,
        calculate_max_bp, calculate_single_reward, calculate_top_bp_with_boost,
        calculate_user_reward_bottom_block, calculate_user_reward_top_block, DUSTS_PER_BLOCK,
        TOKEN_AMOUNT_SCALING_FACTOR,
    };
    use utils::{
        blocks_collided, blocks_solution_required_interval_elapsed, blocks_solved,
        bottom_block_not_solved, convert_f64_to_u64, convert_u64_to_f64,
        final_staking_required_interval_elapsed, initial_token_distribution_not_performed_yet,
        mint_tokens, set_token_metadata, switch_bottom_block_to_next_one_if_applicable,
        switch_top_block_to_next_one_if_applicable, top_block_not_solved, transfer_tokens,
        update_blocks_collided, valid_owner, valid_signer,
    };

    use super::*;

    /// Initializes accounts and set their states. It also mints initial tokens for top and bottom distribution block accounts.
    /// It is the first function that must be called and it can be called only once.
    ///
    /// ### Arguments
    ///
    /// * `ctx` - the initialization context where all the accounts are provided,
    /// * `token_metadata_name` - token's name to set in metadata,
    /// * `token_metadata_symbol` - token's symbol to set in metadata,
    /// * `token_metadata_uri` - token's uri to set in metadata,
    #[access_control(valid_signer(&ctx.accounts.signer))]
    pub fn initialize(
        ctx: Context<InitializeContext>,
        token_metadata_name: String,
        token_metadata_symbol: String,
        token_metadata_uri: String,
    ) -> Result<()> {
        let program_id = id();
        let (_, mint_nonce) = Pubkey::find_program_address(&[MINT_SEED.as_bytes()], &program_id);
        let (_, blocks_state_nonce) =
            Pubkey::find_program_address(&[BLOCKS_STATE_SEED.as_bytes()], &program_id);
        let (_, top_block_nonce) =
            Pubkey::find_program_address(&[DISTRIBUTION_TOP_BLOCK_SEED.as_bytes()], &program_id);
        let (_, bottom_block_nonce) =
            Pubkey::find_program_address(&[DISTRIBUTION_BOTTOM_BLOCK_SEED.as_bytes()], &program_id);
        let (_, final_staking_account_nonce) =
            Pubkey::find_program_address(&[FINAL_STAKING_ACCOUNT_SEED.as_bytes()], &program_id);
        let (_, final_mining_account_nonce) =
            Pubkey::find_program_address(&[FINAL_MINING_ACCOUNT_SEED.as_bytes()], &program_id);

        let blocks_state = &mut ctx.accounts.blocks_state_account;
        blocks_state.authority = ctx.accounts.signer.key();
        blocks_state.mint_nonce = mint_nonce;
        blocks_state.block_state_nonce = blocks_state_nonce;

        blocks_state.top_block_distribution_address =
            ctx.accounts.distribution_top_block_account.key();
        blocks_state.top_block_distribution_nonce = top_block_nonce;
        blocks_state.top_block_solution_timestamp = 0;
        blocks_state.top_block_number = 1_u64;
        blocks_state.top_block_last_account_address = None;
        blocks_state.top_block_last_account_rest_bp = 0;

        blocks_state.top_block_available_bp =
            convert_f64_to_u64(calculate_max_bp(blocks_state.top_block_number)?)?;
        blocks_state.top_block_balance = DUSTS_PER_BLOCK;

        mint_tokens(
            ctx.accounts.mint.to_account_info(),
            ctx.accounts
                .distribution_top_block_account
                .to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            mint_nonce,
            DUSTS_PER_BLOCK,
        )?;

        blocks_state.bottom_block_distribution_address =
            ctx.accounts.distribution_bottom_block_account.key();
        blocks_state.bottom_block_distribution_nonce = bottom_block_nonce;
        blocks_state.bottom_block_solution_timestamp = 0;
        blocks_state.bottom_block_number = 2_600_000_u64;
        blocks_state.bottom_block_last_account_address = None;
        blocks_state.bottom_block_last_account_rest_bp = 0;

        blocks_state.bottom_block_available_bp =
            convert_f64_to_u64(calculate_max_bp(blocks_state.bottom_block_number)?)?;
        blocks_state.bottom_block_balance = DUSTS_PER_BLOCK;

        mint_tokens(
            ctx.accounts.mint.to_account_info(),
            ctx.accounts
                .distribution_bottom_block_account
                .to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            mint_nonce,
            DUSTS_PER_BLOCK,
        )?;

        blocks_state.initial_token_distribution_already_performed = false;
        blocks_state.blocks_collided = false;

        blocks_state.final_staking_account_nonce = final_staking_account_nonce;
        blocks_state.final_staking_pool_in_round = 0;
        blocks_state.final_staking_last_staking_timestamp = 0;
        blocks_state.final_staking_left_reward_parts_in_round = 1.0;
        blocks_state.final_staking_left_balance_in_round = 0;

        blocks_state.final_mining_account_nonce = final_mining_account_nonce;

        set_token_metadata(
            ctx,
            token_metadata_name,
            token_metadata_symbol,
            token_metadata_uri,
        )
    }

    /// Distributes 2 600 000 000 tokens to the organization account provided in the context by minting tokens to the account.
    /// This function can be called only once and it can be called at any time after the initialization.
    ///
    /// ### Arguments
    ///
    /// * `ctx` - the initial token distribution context where the organization account is provided.
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) initial_token_distribution_not_performed_yet(&ctx.accounts.blocks_state_account))]
    pub fn initial_token_distribution(ctx: Context<InitialTokenDistributionContext>) -> Result<()> {
        let blocks_state = &mut ctx.accounts.blocks_state_account;
        let mint_nonce = blocks_state.mint_nonce;

        mint_tokens(
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.organization_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            mint_nonce,
            260_000_000_000_000_u64 * TOKEN_AMOUNT_SCALING_FACTOR,
        )?;

        blocks_state.initial_token_distribution_already_performed = true;

        Ok(())
    }

    /// Solves current top block.
    /// Distributes tokens from top block distribution account to the accounts solving the block, i.e. to the accounts passed in the context and in the `users_info` parameter.
    /// Once the block is solved and all tokens from top block distribution account are distributed, the block is switched to the next one and the distribution account is refilled.
    /// This function can be called multiple times, until all blocks are solved or the blocks would collide after the switch to the next block (i.e. the next block number is already used as the current bottom block number).
    /// The function cannot be invoked for 3 minutes after the block has been solved.
    ///
    /// ### Arguments
    ///
    /// * `ctx` - the solve top block context where all required accounts are provided,
    /// * `users_info` - a vector of accounts solving the current top block, containing the information for each of the accounts needed to calculate the number of tokens to distribute to the accounts.
    ///
    /// ### Returns
    /// Number of current top block after processing all input accounts
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) top_block_not_solved(&ctx.accounts.blocks_state_account) blocks_solution_required_interval_elapsed(&ctx.accounts.blocks_state_account.top_block_solution_timestamp))]
    pub fn solve_top_block<'info>(
        ctx: Context<'_, '_, '_, 'info, SolveTopBlockContext<'info>>,
        users_info: Vec<UserInfoTopBlock>,
    ) -> Result<u64> {
        require!(!&users_info.is_empty(), SallarError::MissingUserInfo);
        let first_user_info_key = users_info.first().unwrap().user_public_key;
        let blocks_state = &mut ctx.accounts.blocks_state_account;
        let block_number = blocks_state.top_block_number;
        let mint_nonce = blocks_state.mint_nonce;

        let top_bp_with_boost = calculate_top_bp_with_boost(block_number)?;
        let dust_per_bp = calculate_dust_per_bp(block_number)?;

        let has_unprocessed_rest_from_last_block = blocks_state.top_block_last_account_rest_bp > 0;
        if has_unprocessed_rest_from_last_block {
            require!(
                blocks_state.top_block_balance == DUSTS_PER_BLOCK,
                SallarError::UserRestExistsButBlockIsNotNew
            );
            require!(
                first_user_info_key == blocks_state.top_block_last_account_address.unwrap(),
                SallarError::UserRestExistsButFirstRequestForNewBlockIsNotForThisAccount
            );

            let account = ctx.remaining_accounts.iter().find(|account| {
                account.key() == blocks_state.top_block_last_account_address.unwrap()
            });
            let account_info = match account {
                Some(acc) => acc.to_account_info(),
                None => {
                    return err!(
                        SallarError::UserRestExistsButFirstRequestForNewBlockMissedTheAccount
                    )
                }
            };

            let user_rest_bp = blocks_state
                .top_block_last_account_rest_bp
                .min(blocks_state.top_block_available_bp);
            let user_rest_transfer_amount: u64;
            if user_rest_bp < blocks_state.top_block_available_bp {
                user_rest_transfer_amount = calculate_single_reward(user_rest_bp, dust_per_bp)?;
            } else {
                user_rest_transfer_amount = blocks_state.top_block_balance;
            }

            transfer_tokens(
                &ctx.accounts.distribution_top_block_account,
                account_info,
                DISTRIBUTION_TOP_BLOCK_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.top_block_distribution_nonce,
                user_rest_transfer_amount,
            )?;

            blocks_state.top_block_available_bp =
                blocks_state.top_block_available_bp - user_rest_bp;
            blocks_state.top_block_last_account_rest_bp =
                blocks_state.top_block_last_account_rest_bp - user_rest_bp;
            blocks_state.top_block_balance =
                blocks_state.top_block_balance - user_rest_transfer_amount;
        }
        let users_info_without_info_for_user_rest = match has_unprocessed_rest_from_last_block {
            true => users_info
                .into_iter()
                .skip(1)
                .collect::<Vec<UserInfoTopBlock>>(),
            false => users_info,
        };

        for user_info in &users_info_without_info_for_user_rest {
            require!(
                blocks_state.top_block_available_bp > 0,
                SallarError::UserRequestForSolvedBlock
            );

            let account = ctx
                .remaining_accounts
                .iter()
                .find(|account| account.key() == user_info.user_public_key);
            let account_info = match account {
                Some(acc) => acc.to_account_info(),
                None => return err!(SallarError::MismatchBetweenRemainingAccountsAndUserInfo),
            };

            let (current_user_reward_bp, mut current_user_transfer_amount) =
                calculate_user_reward_top_block(
                    user_info.user_request_without_boost,
                    user_info.user_request_with_boost,
                    top_bp_with_boost,
                    dust_per_bp,
                )?;

            if current_user_reward_bp <= blocks_state.top_block_available_bp {
                blocks_state.top_block_last_account_rest_bp = 0;
                blocks_state.top_block_available_bp -= current_user_reward_bp;
            } else {
                blocks_state.top_block_last_account_rest_bp =
                    current_user_reward_bp - blocks_state.top_block_available_bp;
                blocks_state.top_block_available_bp = 0;
            }

            if blocks_state.top_block_available_bp == 0 {
                current_user_transfer_amount = blocks_state.top_block_balance;
            }

            transfer_tokens(
                &ctx.accounts.distribution_top_block_account,
                account_info,
                DISTRIBUTION_TOP_BLOCK_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.top_block_distribution_nonce,
                current_user_transfer_amount,
            )?;

            blocks_state.top_block_balance -= current_user_transfer_amount;
            blocks_state.top_block_last_account_address = Some(user_info.user_public_key);
        }

        switch_top_block_to_next_one_if_applicable(
            blocks_state,
            mint_nonce,
            &ctx.accounts.mint,
            ctx.accounts
                .distribution_top_block_account
                .to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        )?;
        update_blocks_collided(blocks_state)?;

        Ok(blocks_state.top_block_number)
    }

    /// Solves current bottom block.
    /// Distributes tokens from bottom block distribution account to the accounts solving the block, i.e. to the accounts passed in the context and in the `users_info` parameter.
    /// Once the block is solved and all tokens are from bottom block distribution account are distributed, the block is switched to the next one and the distribution account is refilled.
    /// This function can be called multiple times, until all blocks are solved or the blocks would collide after the switch to the next block (i.e. the next block number is already used as the current top block number).
    /// The function cannot be invoked for 3 minutes after the block has been solved.
    ///
    /// ### Arguments
    ///
    /// * `ctx` - the solve bottom block context where all required accounts are provided,
    /// * `users_info` - a vector of accounts solving the current bottom block, containing the information for each of the accounts needed to calculate the number of tokens to distribute to the accounts.
    ///
    /// ### Returns
    /// Number of current bottom block after processing all input accounts
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) bottom_block_not_solved(&ctx.accounts.blocks_state_account) blocks_solution_required_interval_elapsed(&ctx.accounts.blocks_state_account.bottom_block_solution_timestamp))]
    pub fn solve_bottom_block<'info>(
        ctx: Context<'_, '_, '_, 'info, SolveBottomBlockContext<'info>>,
        users_info: Vec<UserInfoBottomBlock>,
    ) -> Result<u64> {
        require!(!&users_info.is_empty(), SallarError::MissingUserInfo);
        let first_user_info_key = users_info.first().unwrap().user_public_key;
        let blocks_state = &mut ctx.accounts.blocks_state_account;
        let block_number = blocks_state.bottom_block_number;
        let mint_nonce = blocks_state.mint_nonce;

        let mut current_user_reward_bp;
        let mut current_user_transfer_amount;

        let dust_per_bp = calculate_dust_per_bp(block_number)?;

        let has_unprocessed_rest_from_last_block =
            blocks_state.bottom_block_last_account_rest_bp > 0;
        if has_unprocessed_rest_from_last_block {
            require!(
                blocks_state.bottom_block_balance == DUSTS_PER_BLOCK,
                SallarError::UserRestExistsButBlockIsNotNew
            );
            require!(
                first_user_info_key == blocks_state.bottom_block_last_account_address.unwrap(),
                SallarError::UserRestExistsButFirstRequestForNewBlockIsNotForThisAccount
            );

            let account = ctx.remaining_accounts.iter().find(|account| {
                account.key() == blocks_state.bottom_block_last_account_address.unwrap()
            });
            let account_info = match account {
                Some(acc) => acc.to_account_info(),
                None => {
                    return err!(
                        SallarError::UserRestExistsButFirstRequestForNewBlockMissedTheAccount
                    )
                }
            };

            let user_rest_bp = blocks_state
                .bottom_block_last_account_rest_bp
                .min(blocks_state.bottom_block_available_bp);
            let user_rest_transfer_amount: u64;
            if user_rest_bp < blocks_state.bottom_block_available_bp {
                user_rest_transfer_amount = calculate_single_reward(user_rest_bp, dust_per_bp)?;
            } else {
                user_rest_transfer_amount = blocks_state.bottom_block_balance;
            }

            transfer_tokens(
                &ctx.accounts.distribution_bottom_block_account,
                account_info,
                DISTRIBUTION_BOTTOM_BLOCK_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.bottom_block_distribution_nonce,
                user_rest_transfer_amount,
            )?;

            blocks_state.bottom_block_available_bp =
                blocks_state.bottom_block_available_bp - user_rest_bp;
            blocks_state.bottom_block_last_account_rest_bp =
                blocks_state.bottom_block_last_account_rest_bp - user_rest_bp;
            blocks_state.bottom_block_balance =
                blocks_state.bottom_block_balance - user_rest_transfer_amount;
        }
        let users_info_without_info_for_user_rest = match has_unprocessed_rest_from_last_block {
            true => users_info
                .into_iter()
                .skip(1)
                .collect::<Vec<UserInfoBottomBlock>>(),
            false => users_info,
        };

        for user_info in &users_info_without_info_for_user_rest {
            require!(
                blocks_state.bottom_block_available_bp > 0,
                SallarError::UserRequestForSolvedBlock
            );

            let account = ctx
                .remaining_accounts
                .iter()
                .find(|account| account.key() == user_info.user_public_key);
            let account_info = match account {
                Some(acc) => acc.to_account_info(),
                None => return err!(SallarError::MismatchBetweenRemainingAccountsAndUserInfo),
            };

            let bottom_bp_with_boost =
                calculate_bottom_bp_with_boost(block_number, user_info.user_balance)?;
            let bottom_bp_without_boost = calculate_bottom_bp_without_boost(user_info.user_balance);

            (current_user_reward_bp, current_user_transfer_amount) =
                calculate_user_reward_bottom_block(
                    user_info.user_request_without_boost,
                    user_info.user_request_with_boost,
                    bottom_bp_without_boost,
                    bottom_bp_with_boost,
                    dust_per_bp,
                    user_info.user_balance,
                )?;

            if current_user_reward_bp <= blocks_state.bottom_block_available_bp {
                blocks_state.bottom_block_last_account_rest_bp = 0;
                blocks_state.bottom_block_available_bp -= current_user_reward_bp;
            } else {
                blocks_state.bottom_block_last_account_rest_bp =
                    current_user_reward_bp - blocks_state.bottom_block_available_bp;
                blocks_state.bottom_block_available_bp = 0;
            }

            if blocks_state.bottom_block_available_bp == 0 {
                current_user_transfer_amount = blocks_state.bottom_block_balance;
            }

            transfer_tokens(
                &ctx.accounts.distribution_bottom_block_account,
                account_info,
                DISTRIBUTION_BOTTOM_BLOCK_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.bottom_block_distribution_nonce,
                current_user_transfer_amount,
            )?;

            blocks_state.bottom_block_balance -= current_user_transfer_amount;
            blocks_state.bottom_block_last_account_address = Some(user_info.user_public_key);
        }

        switch_bottom_block_to_next_one_if_applicable(
            blocks_state,
            mint_nonce,
            &ctx.accounts.mint,
            ctx.accounts
                .distribution_bottom_block_account
                .to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        )?;
        update_blocks_collided(blocks_state)?;

        Ok(blocks_state.bottom_block_number)
    }

    /// Distributes tokens from final mining account to accounts passed in the input to this function.
    /// The amount of tokens transferred to particular account depends on the final mining account's balance in the moment when user requested participation in final mining on the client side so the balance is passed in the input.
    /// This function can be called unlimited number of times but only after all top and bottom blocks are solved.
    ///
    /// ### Arguments
    ///
    /// * `ctx` - the final mining context where all required accounts are provided,
    /// * `users_info` - a vector of accounts participating in the final mining process, containing the information for each of the accounts needed to calculate the number of tokens to distribute to the accounts.
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) blocks_collided(&ctx.accounts.blocks_state_account) blocks_solved(&ctx.accounts.blocks_state_account))]
    pub fn final_mining<'info>(
        ctx: Context<'_, '_, '_, 'info, FinalMiningContext<'info>>,
        users_info: Vec<UserInfoFinalMining>,
    ) -> Result<()> {
        require!(!users_info.is_empty(), SallarError::MissingUserInfo);
        let blocks_state = &mut ctx.accounts.blocks_state_account;

        for account in ctx.remaining_accounts.iter() {
            let user_find_result = users_info
                .iter()
                .filter(|user_info| user_info.user_public_key == account.key())
                .collect::<Vec<&UserInfoFinalMining>>();

            require!(
                user_find_result.len() > 0,
                SallarError::MismatchBetweenRemainingAccountsAndUserInfo
            );

            let mut total_amount = 0;
            for user_sub_info in &user_find_result {
                let transfer_amount = match user_sub_info.final_mining_balance {
                    0...12_499_999_999_999_999 => 2_500_000_000,
                    12_500_000_000_000_000...24_999_999_999_999_999 => 5_000_000_000,
                    25_000_000_000_000_000...49_999_999_999_999_999 => 10_000_000_000,
                    50_000_000_000_000_000...99_999_999_999_999_999 => 25_000_000_000,
                    _ => 50_000_000_000,
                };
                total_amount += transfer_amount;
            }
            transfer_tokens(
                &ctx.accounts.final_mining_account,
                account.to_account_info(),
                FINAL_MINING_ACCOUNT_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.final_mining_account_nonce,
                total_amount,
            )?;
        }

        Ok(())
    }

    /// Distributes tokens from final staking account to accounts passed in the input to this function.
    /// Final staking processed is organized as rounds. At the beginning of each round 0.1% of the current final staking account balance is reserved as the prize pool for the round.
    /// The amount of tokens transferred to particular account depends on the account's balance and the prize pool of the current round.
    /// This function can be called unlimited number of times but only after all top and bottom blocks are solved.
    /// The function cannot be invoked for 20 hours after the final staking round has been completed.
    ///
    /// ### Arguments
    ///
    /// * `ctx` - the final staking context where all required accounts are provided,
    /// * `users_info` - a vector of accounts participating in the final staking process, containing the information for each of the accounts needed to calculate the number of tokens to distribute to the accounts.
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) blocks_collided(&ctx.accounts.blocks_state_account) blocks_solved(&ctx.accounts.blocks_state_account) final_staking_required_interval_elapsed(&ctx.accounts.blocks_state_account.final_staking_last_staking_timestamp))]
    pub fn final_staking<'info>(
        ctx: Context<'_, '_, '_, 'info, FinalStakingContext<'info>>,
        users_info: Vec<UserInfoFinalStaking>,
    ) -> Result<()> {
        let blocks_state = &mut ctx.accounts.blocks_state_account;
        let mut total_users_reward_part = 0.0;

        if blocks_state.final_staking_left_balance_in_round == 0 {
            let final_staking_account_balance =
                token::accessor::amount(&ctx.accounts.final_staking_account.to_account_info())?;
            blocks_state.final_staking_pool_in_round = final_staking_account_balance
                / FINAL_STAKING_ACCOUNT_BALANCE_PART_FOR_STAKING_DIVISION_FACTOR;

            require!(
                blocks_state.final_staking_pool_in_round > 0,
                SallarError::FinalStakingPoolInRoundIsEmpty
            );

            blocks_state.final_staking_left_balance_in_round =
                blocks_state.final_staking_pool_in_round;
            blocks_state.final_staking_left_reward_parts_in_round = 1.0;
        }

        users_info
            .iter()
            .for_each(|user_info| total_users_reward_part += user_info.reward_part);

        require!(
            total_users_reward_part <= 1.0,
            SallarError::UserRewardPartsSumTooHigh
        );

        let mut current_user_transfer_amount;

        for account in ctx.remaining_accounts.iter() {
            let user_find_result = users_info
                .iter()
                .filter(|user_info| user_info.user_public_key == account.key())
                .collect::<Vec<&UserInfoFinalStaking>>();

            require!(
                user_find_result.len() > 0,
                SallarError::MismatchBetweenRemainingAccountsAndUserInfo
            );

            for user_sub_info in &user_find_result {
                require!(
                    user_sub_info.reward_part <= 1.0 && user_sub_info.reward_part > 0.0,
                    SallarError::UserRequestExceedsAvailableRewardParts
                );

                let reward_parts_pool_after_user = blocks_state
                    .final_staking_left_reward_parts_in_round
                    - user_sub_info.reward_part;
                require!(
                    reward_parts_pool_after_user >= 0.0,
                    SallarError::UserRequestExceedsAvailableRewardParts
                );

                if reward_parts_pool_after_user == 0.0 {
                    current_user_transfer_amount = blocks_state.final_staking_left_balance_in_round;
                } else {
                    current_user_transfer_amount = convert_f64_to_u64(
                        user_sub_info.reward_part
                            * convert_u64_to_f64(blocks_state.final_staking_pool_in_round)?,
                    )?;
                }

                require!(
                    current_user_transfer_amount
                        <= blocks_state.final_staking_left_balance_in_round,
                    SallarError::LackOfFundsToPayTheReward
                );

                transfer_tokens(
                    &ctx.accounts.final_staking_account,
                    account.to_account_info(),
                    FINAL_STAKING_ACCOUNT_SEED,
                    ctx.accounts.token_program.to_account_info(),
                    blocks_state.final_staking_account_nonce,
                    current_user_transfer_amount,
                )?;

                blocks_state.final_staking_left_reward_parts_in_round =
                    reward_parts_pool_after_user;
                blocks_state.final_staking_left_balance_in_round -= current_user_transfer_amount;
            }
        }

        if blocks_state.final_staking_left_balance_in_round == 0 {
            blocks_state.final_staking_last_staking_timestamp = Clock::get()?.unix_timestamp;
        }

        Ok(())
    }

    /// Sets new authority
    ///
    /// ### Arguments
    ///
    /// * `new_authority` - new authority
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn change_authority<'info>(
        ctx: Context<'_, '_, '_, 'info, ChangeAuthorityContext<'info>>,
        new_authority: Pubkey,
    ) -> Result<()> {
        let blocks_state_account = &mut ctx.accounts.blocks_state_account;
        blocks_state_account.authority = new_authority;

        Ok(())
    }

    /// Set blocks collided flag
    /// This function is only available in tests
    ///
    /// ### Arguments
    ///
    /// * `collided` - new value of blocks collided flag
    #[access_control(valid_owner(&ctx.accounts.blocks_state_account, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer))]
    pub fn set_blocks_collided<'info>(
        ctx: Context<'_, '_, '_, 'info, SetBlocksCollidedContext<'info>>,
        collided: bool,
    ) -> Result<()> {
        require!(
            cfg!(feature = "bpf-tests"),
            SallarError::ExecutionOfSetBlocksCollidedFunctionOutsideTests
        );

        let blocks_state_account = &mut ctx.accounts.blocks_state_account;
        blocks_state_account.blocks_collided = collided;
        blocks_state_account.top_block_available_bp = 0;
        blocks_state_account.bottom_block_available_bp = 0;

        Ok(())
    }
}

/// Struct defining single account participating in the top block solution process.
/// Consists of the account address and data required to calculate the number of tokens to transfer to the account (number of requests to participate in the current top block solution on the client side).
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoTopBlock {
    pub user_public_key: Pubkey,
    pub user_request_without_boost: u8,
    pub user_request_with_boost: u8,
}

/// Struct defining single account participating in the bottom block solution process.
/// Consists of the account address and data required to calculate the number of tokens to transfer to the account (account's balance and number of requests to participate in the current bottom block solution on the client side).
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoBottomBlock {
    pub user_public_key: Pubkey,
    pub user_balance: u64,
    pub user_request_without_boost: u8,
    pub user_request_with_boost: u8,
}

/// Struct defining single account participating in the final mining process.
/// Consists of the account address and data required to calculate the number of tokens to be transferred to the account (final mining account balance at the time the account requested participation in the final mining process on the client side).
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoFinalMining {
    pub user_public_key: Pubkey,
    pub final_mining_balance: u64,
}

/// Struct defining single account participating in the final staking process.
/// Consists of the account address and data required to calculate the number of tokens to be transferred to the account (part of the total prize pool declared for the current final staking round).
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoFinalStaking {
    pub user_public_key: Pubkey,
    pub reward_part: f64,
}

#[cfg(test)]
mod test {
    use super::*;

    use anchor_lang::{prelude::AccountMeta, system_program, InstructionData, ToAccountMetas};
    use anchor_spl::token::spl_token;
    use solana_program_test::*;
    use spl_token::state::Account;

    use solana_sdk::{
        commitment_config::CommitmentLevel, signature::Keypair, signer::Signer,
        transaction::Transaction,
    };

    use solana_program::{
        hash::Hash, instruction::Instruction, program_pack::Pack, system_instruction,
    };
    use utils::final_staking_required_interval_elapsed;

    #[cfg(feature = "bpf-tests")]
    use solana_program::{instruction::InstructionError, sysvar::clock::Clock};

    #[cfg(feature = "bpf-tests")]
    use std::collections::HashMap;

    #[cfg(feature = "bpf-tests")]
    use solana_sdk::transaction::TransactionError;

    impl Clone for UserInfoBottomBlock {
        fn clone(&self) -> Self {
            Self {
                user_public_key: self.user_public_key.clone(),
                user_balance: self.user_balance.clone(),
                user_request_without_boost: self.user_request_without_boost.clone(),
                user_request_with_boost: self.user_request_with_boost.clone(),
            }
        }
    }

    impl Clone for UserInfoTopBlock {
        fn clone(&self) -> Self {
            Self {
                user_public_key: self.user_public_key.clone(),
                user_request_without_boost: self.user_request_without_boost.clone(),
                user_request_with_boost: self.user_request_with_boost.clone(),
            }
        }
    }

    async fn initialize_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
    ) -> Result<()> {
        let program_id = id();
        let (
            mint_pda,
            _,
            blocks_state_pda,
            _,
            distribution_top_block_pda,
            _,
            distribution_bottom_block_pda,
            _,
            final_staking_account_pda,
            _,
            final_mining_account_pda,
            _,
        ) = get_pda_accounts();
        let metadata_seed1 = "metadata".as_bytes();
        let metadata_seed2 = &mpl_token_metadata::id().to_bytes();
        let metadata_seed3 = &mint_pda.to_bytes();
        let (metadata_pda, _) = Pubkey::find_program_address(
            &[metadata_seed1, metadata_seed2, metadata_seed3],
            &mpl_token_metadata::id(),
        );

        let token_program = spl_token::id();
        let signer = payer.pubkey();
        let token_metadata_name = "Sallar".to_string();
        let token_metadata_symbol = "ALL".to_string();
        let token_metadata_uri = "http://sallar.io".to_string();

        let data = instruction::Initialize {
            token_metadata_name,
            token_metadata_symbol,
            token_metadata_uri,
        }
        .data();

        let accs = accounts::InitializeContext {
            blocks_state_account: blocks_state_pda,
            token_program,
            signer,
            system_program: system_program::ID,
            mint: mint_pda,
            distribution_top_block_account: distribution_top_block_pda,
            distribution_bottom_block_account: distribution_bottom_block_pda,
            final_staking_account: final_staking_account_pda,
            final_mining_account: final_mining_account_pda,
            metadata_pda,
            metadata_program: mpl_token_metadata::id(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Confirmed)
            .await
            .unwrap();

        Ok(())
    }

    #[cfg(feature = "bpf-tests")]
    async fn initial_token_distribution_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
        organization_account: Pubkey,
    ) -> Result<()> {
        let program_id = id();
        let (mint_pda, _, blocks_state_pda, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let data = instruction::InitialTokenDistribution {}.data();

        let accs = accounts::InitialTokenDistributionContext {
            blocks_state_account: blocks_state_pda,
            mint: mint_pda,
            organization_account,
            token_program,
            signer,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap();

        Ok(())
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_initialize() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);

        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_initial_token_distribution() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (mint_pda, _) = Pubkey::find_program_address(&[MINT_SEED.as_bytes()], &program_id);
        let organization_account =
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap();
        initial_token_distribution_instruction(
            &mut banks_client,
            &payer,
            recent_blockhash,
            organization_account,
        )
        .await
        .unwrap();
    }

    async fn solve_top_block_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
        key_list: &Vec<Pubkey>,
        users_info: &Vec<UserInfoTopBlock>,
    ) -> Result<()> {
        let program_id = id();

        let (mint_pda, _, blocks_state_pda, _, distribution_top_block_pda, _, _, _, _, _, _, _) =
            get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let data = instruction::SolveTopBlock {
            users_info: users_info.clone(),
        }
        .data();

        let accs = accounts::SolveTopBlockContext {
            blocks_state_account: blocks_state_pda,
            mint: mint_pda,
            distribution_top_block_account: distribution_top_block_pda,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        for key in key_list.iter() {
            accounts.push(AccountMeta::new(*key, false));
        }

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap();

        Ok(())
    }

    async fn solve_bottom_block_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
        key_list: &Vec<Pubkey>,
        users_info: &Vec<UserInfoBottomBlock>,
    ) -> Result<()> {
        let program_id = id();

        let (mint_pda, _, blocks_state_pda, _, _, _, distribution_bottom_block_pda, _, _, _, _, _) =
            get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let data = instruction::SolveBottomBlock {
            users_info: users_info.clone(),
        }
        .data();

        let accs = accounts::SolveBottomBlockContext {
            blocks_state_account: blocks_state_pda,
            mint: mint_pda,
            distribution_bottom_block_account: distribution_bottom_block_pda,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        for key in key_list.into_iter() {
            accounts.push(AccountMeta::new(*key, false));
        }

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction, CommitmentLevel::Finalized)
            .await
            .unwrap();

        Ok(())
    }

    #[cfg(feature = "bpf-tests")]
    async fn set_blocks_collided_instruction(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
        collided: bool,
    ) -> Result<()> {
        let program_id = id();
        let (_, _, blocks_state_pda, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let signer = payer.pubkey();

        let data = instruction::SetBlocksCollided { collided }.data();

        let accs = accounts::SetBlocksCollidedContext {
            blocks_state_account: blocks_state_pda,
            signer,
        };

        let accounts = accs.to_account_metas(Some(false));

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[payer], recent_blockhash);

        banks_client
            .process_transaction_with_commitment(transaction, CommitmentLevel::Confirmed)
            .await
            .unwrap();

        Ok(())
    }

    async fn default_top_block_setup(
        banks_client: &mut BanksClient,
        payer: &Keypair,
    ) -> (Vec<Pubkey>, Vec<UserInfoTopBlock>) {
        let (mint_pda, _, _, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let mut key_list = vec![];

        for _ in 0..5 {
            let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
            key_list.push(
                create_token_account(banks_client, payer, recent_blockhash, mint_pda)
                    .await
                    .unwrap(),
            );
        }

        let mut users_info: Vec<UserInfoTopBlock> = vec![];

        for key in key_list.iter() {
            let user_info = UserInfoTopBlock {
                user_public_key: *key,
                user_request_with_boost: 1,
                user_request_without_boost: 1,
            };
            users_info.push(user_info);
        }

        (key_list, users_info)
    }

    async fn default_bottom_block_setup(
        banks_client: &mut BanksClient,
        payer: &Keypair,
    ) -> (Vec<Pubkey>, Vec<UserInfoBottomBlock>) {
        let (mint_pda, _, _, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let mut key_list: Vec<Pubkey> = vec![];
        for _ in 0..1 {
            let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
            key_list.push(
                create_token_account(banks_client, payer, recent_blockhash, mint_pda)
                    .await
                    .unwrap(),
            );
        }

        let mut users_info: Vec<UserInfoBottomBlock> = vec![];
        for key in key_list.iter() {
            users_info.push(UserInfoBottomBlock {
                user_public_key: key.clone(),
                user_balance: 107_753_703_900_000_000,
                user_request_without_boost: 25,
                user_request_with_boost: 0,
            });
        }

        (key_list, users_info)
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_solve_top_block_full_block() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (key_list, users_info) = default_top_block_setup(&mut banks_client, &payer).await;

        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        solve_top_block_instruction(
            &mut banks_client,
            &payer,
            recent_blockhash,
            &key_list,
            &users_info,
        )
        .await
        .unwrap();
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        solve_top_block_instruction(
            &mut banks_client,
            &payer,
            recent_blockhash,
            &key_list,
            &users_info,
        )
        .await
        .unwrap();

        for key in key_list.iter() {
            let account = banks_client.get_account(*key).await.unwrap().unwrap();
            let account_data = Account::unpack(&account.data).unwrap();
            assert_eq!(account_data.amount, 400000000000);
        }
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_solve_top_two_blocks_with_user_rest() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));
        program_test.set_compute_max_units(5000000);

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let mut program_test_context = program_test.start_with_context().await;
        let mut banks_client = program_test_context.banks_client.clone();
        let recent_blockhash = program_test_context.last_blockhash;

        let mut time_in_timestamp = 1677978061;
        set_time(&mut program_test_context, time_in_timestamp).await;

        initialize_instruction(
            &mut banks_client,
            &program_test_context.payer,
            recent_blockhash,
        )
        .await
        .unwrap();

        let (mint_pda, _, _, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let key_list = vec![create_token_account(
            &mut banks_client,
            &program_test_context.payer,
            recent_blockhash,
            mint_pda,
        )
        .await
        .unwrap()];
        let users_info: Vec<UserInfoTopBlock> = vec![UserInfoTopBlock {
            user_public_key: key_list[0].clone(),
            user_request_without_boost: 50,
            user_request_with_boost: 0,
        }];

        for _ in 0..2 {
            let recent_blockhash = program_test_context
                .banks_client
                .get_latest_blockhash()
                .await
                .unwrap();
            solve_top_block_instruction(
                &mut banks_client,
                &program_test_context.payer,
                recent_blockhash,
                &key_list,
                &users_info,
            )
            .await
            .unwrap();

            // move time forward for 3 minutes to pass the required time between solved blocks
            time_in_timestamp = time_in_timestamp + 180;
            set_time(&mut program_test_context, time_in_timestamp).await;
        }

        let key_list = vec![
            key_list[0],
            create_token_account(
                &mut banks_client,
                &program_test_context.payer,
                recent_blockhash,
                mint_pda,
            )
            .await
            .unwrap(),
        ];
        let users_info: Vec<UserInfoTopBlock> = vec![
            UserInfoTopBlock {
                user_public_key: key_list[0].clone(),
                user_request_without_boost: 0,
                user_request_with_boost: 0,
            },
            UserInfoTopBlock {
                user_public_key: key_list[1].clone(),
                user_request_without_boost: 7,
                user_request_with_boost: 0,
            },
        ];

        let recent_blockhash = program_test_context
            .banks_client
            .get_latest_blockhash()
            .await
            .unwrap();
        solve_top_block_instruction(
            &mut banks_client,
            &program_test_context.payer,
            recent_blockhash,
            &key_list,
            &users_info,
        )
        .await
        .unwrap();

        let expected_user_balances: HashMap<Pubkey, u64> =
            HashMap::from([(key_list[0], 5000000000000), (key_list[1], 700000000000)]);
        for key in key_list.iter() {
            let user_account = (&mut banks_client).get_account(*key).await.unwrap();
            let user_account_data = Account::unpack(&user_account.unwrap().data).unwrap();
            assert_eq!(user_account_data.amount, expected_user_balances[key]);
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn test_fail_solve_top_block() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (key_list, users_info) = default_top_block_setup(&mut banks_client, &payer).await;

        for _ in 0..3 {
            let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
            solve_top_block_instruction(
                &mut banks_client,
                &payer,
                recent_blockhash,
                &key_list,
                &users_info,
            )
            .await
            .unwrap();
        }
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_solve_bottom_block() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (key_list, users_info) = default_bottom_block_setup(&mut banks_client, &payer).await;

        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        solve_bottom_block_instruction(
            &mut banks_client,
            &payer,
            recent_blockhash,
            &key_list,
            &users_info,
        )
        .await
        .unwrap();

        for key in key_list.iter() {
            let account = banks_client.get_account(*key).await.unwrap().unwrap();
            let account_data = Account::unpack(&account.data).unwrap();
            assert_eq!(account_data.amount, 1000000000000);
        }
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_solve_bottom_block_full_block() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (key_list, users_info) = default_bottom_block_setup(&mut banks_client, &payer).await;

        for _ in 0..2 {
            let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
            solve_bottom_block_instruction(
                &mut banks_client,
                &payer,
                recent_blockhash,
                &key_list,
                &users_info,
            )
            .await
            .unwrap();
        }

        for key in key_list.iter() {
            let user_account = (&mut banks_client).get_account(*key).await.unwrap();
            let user_account_data = Account::unpack(&user_account.unwrap().data).unwrap();
            assert_eq!(user_account_data.amount, 2000000000000);
        }
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_solve_bottom_two_blocks_with_user_rest() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));
        program_test.set_compute_max_units(5000000);

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let mut program_test_context = program_test.start_with_context().await;
        let mut banks_client = program_test_context.banks_client.clone();
        let recent_blockhash = program_test_context.last_blockhash;

        let time_in_timestamp = 1677978061;
        set_time(&mut program_test_context, time_in_timestamp).await;

        initialize_instruction(
            &mut banks_client,
            &program_test_context.payer,
            recent_blockhash,
        )
        .await
        .unwrap();

        let payer = &program_test_context.payer;

        let (mint_pda, _, _, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let mut key_list = vec![
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
        ];

        let mut users_info: Vec<UserInfoBottomBlock> = vec![];
        for key in key_list.iter() {
            users_info.push(UserInfoBottomBlock {
                user_public_key: key.clone(),
                user_balance: 200_000_000_000_000,
                user_request_without_boost: 255,
                user_request_with_boost: 255,
            });
        }

        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        solve_bottom_block_instruction(
            &mut banks_client,
            &program_test_context.payer,
            recent_blockhash,
            &key_list,
            &users_info,
        )
        .await
        .unwrap();

        // move time forward for 3 minutes to pass the required time between solved blocks
        let time_in_timestamp = time_in_timestamp + 180;
        set_time(&mut program_test_context, time_in_timestamp).await;

        // the user that solved the previous block must be provided as the first one in the request to solve next block
        // so one of ways to do this is to reuse the users provided in the first request but in the reversed order
        key_list.reverse();
        users_info.reverse();

        let recent_blockhash = program_test_context
            .banks_client
            .get_latest_blockhash()
            .await
            .unwrap();
        solve_bottom_block_instruction(
            &mut banks_client,
            &program_test_context.payer,
            recent_blockhash,
            &key_list,
            &users_info,
        )
        .await
        .unwrap();

        let expected_user_balances: HashMap<Pubkey, u64> =
            HashMap::from([(key_list[0], 1173789936729), (key_list[1], 2347582599105)]);
        for key in key_list.iter() {
            let user_account = (&mut banks_client).get_account(*key).await.unwrap();
            let user_account_data = Account::unpack(&user_account.unwrap().data).unwrap();
            assert_eq!(user_account_data.amount, expected_user_balances[key]);
        }
    }

    #[tokio::test]
    #[should_panic]
    async fn test_fail_solve_bottom_block_block() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (key_list, users_info) = default_bottom_block_setup(&mut banks_client, &payer).await;

        for _ in 0..3 {
            let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
            solve_bottom_block_instruction(
                &mut banks_client,
                &payer,
                recent_blockhash,
                &key_list,
                &users_info,
            )
            .await
            .unwrap();
        }
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_final_mining_fail_blocks_not_collided() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (mint_pda, _, blocks_state_pda, _, _, _, _, _, _, _, final_mining_account_pda, _) =
            get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let key_list = vec![
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
        ];

        let users_info: Vec<UserInfoFinalMining> = vec![
            UserInfoFinalMining {
                user_public_key: key_list[0],
                final_mining_balance: 1,
            },
            UserInfoFinalMining {
                user_public_key: key_list[1],
                final_mining_balance: 1,
            },
            UserInfoFinalMining {
                user_public_key: key_list[2],
                final_mining_balance: 1,
            },
            UserInfoFinalMining {
                user_public_key: key_list[3],
                final_mining_balance: 1,
            },
        ];

        let data = instruction::FinalMining { users_info }.data();

        let accs = accounts::FinalMiningContext {
            blocks_state_account: blocks_state_pda,
            final_mining_account: final_mining_account_pda,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        accounts.push(AccountMeta::new(key_list[0], false));
        accounts.push(AccountMeta::new(key_list[1], false));
        accounts.push(AccountMeta::new(key_list[2], false));
        accounts.push(AccountMeta::new(key_list[3], false));

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        let error = banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap_err()
            .unwrap();
        assert_eq!(get_custom_error_code(error).unwrap(), 6007);
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_final_mining() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (mint_pda, _, blocks_state_pda, _, _, _, _, _, _, _, final_mining_account_pda, _) =
            get_pda_accounts();

        initial_token_distribution_instruction(
            &mut banks_client,
            &payer,
            recent_blockhash,
            final_mining_account_pda,
        )
        .await
        .unwrap();

        set_blocks_collided_instruction(&mut banks_client, &payer, recent_blockhash, true)
            .await
            .unwrap();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let key_list =
            vec![
                create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                    .await
                    .unwrap(),
            ];

        let users_info: Vec<UserInfoFinalMining> = vec![UserInfoFinalMining {
            user_public_key: key_list[0],
            final_mining_balance: 1,
        }];

        let data = instruction::FinalMining { users_info }.data();

        let accs = accounts::FinalMiningContext {
            blocks_state_account: blocks_state_pda,
            final_mining_account: final_mining_account_pda,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        accounts.push(AccountMeta::new(key_list[0], false));

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap();
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_final_staking_fail_blocks_not_collided() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let program_id = id();

        let (mint_pda, _, blocks_state_pda, _, _, _, _, _, final_staking_account_pda, _, _, _) =
            get_pda_accounts();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let key_list = vec![
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
            create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                .await
                .unwrap(),
        ];

        let users_info: Vec<UserInfoFinalStaking> = vec![
            UserInfoFinalStaking {
                user_public_key: key_list[0],
                reward_part: 0.1,
            },
            UserInfoFinalStaking {
                user_public_key: key_list[1],
                reward_part: 0.1,
            },
            UserInfoFinalStaking {
                user_public_key: key_list[2],
                reward_part: 0.1,
            },
            UserInfoFinalStaking {
                user_public_key: key_list[3],
                reward_part: 0.1,
            },
        ];

        let data = instruction::FinalStaking { users_info }.data();

        let accs = accounts::FinalStakingContext {
            blocks_state_account: blocks_state_pda,
            final_staking_account: final_staking_account_pda,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        accounts.push(AccountMeta::new(key_list[0], false));
        accounts.push(AccountMeta::new(key_list[1], false));
        accounts.push(AccountMeta::new(key_list[2], false));
        accounts.push(AccountMeta::new(key_list[3], false));

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        let error = banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Finalized)
            .await
            .unwrap_err()
            .unwrap();
        assert_eq!(get_custom_error_code(error).unwrap(), 6007);
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_final_staking() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (mint_pda, _, blocks_state_pda, _, _, _, _, _, final_staking_account_pda, _, _, _) =
            get_pda_accounts();

        initial_token_distribution_instruction(
            &mut banks_client,
            &payer,
            recent_blockhash,
            final_staking_account_pda,
        )
        .await
        .unwrap();

        set_blocks_collided_instruction(&mut banks_client, &payer, recent_blockhash, true)
            .await
            .unwrap();

        let program_id = id();

        let token_program = spl_token::id();
        let signer = payer.pubkey();

        let key_list =
            vec![
                create_token_account(&mut banks_client, &payer, recent_blockhash, mint_pda)
                    .await
                    .unwrap(),
            ];

        let users_info: Vec<UserInfoFinalStaking> = vec![UserInfoFinalStaking {
            user_public_key: key_list[0],
            reward_part: 0.1,
        }];

        let data = instruction::FinalStaking { users_info }.data();

        let accs = accounts::FinalStakingContext {
            blocks_state_account: blocks_state_pda,
            final_staking_account: final_staking_account_pda,
            token_program,
            signer,
        };

        let mut accounts = accs.to_account_metas(Some(false));
        accounts.push(AccountMeta::new(key_list[0], false));

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(program_id, &data, accounts)],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client
            .process_transaction_with_commitment(transaction.clone(), CommitmentLevel::Confirmed)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_fail_final_staking_required_interval_elapsed_without_context() {
        final_staking_required_interval_elapsed(&1).unwrap();
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_new_authority() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (_, _, blocks_state_pda, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let data = instruction::ChangeAuthority {
            new_authority: signer,
        }
        .data();

        let accs = accounts::ChangeAuthorityContext {
            blocks_state_account: blocks_state_pda,
            signer,
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[cfg(feature = "bpf-tests")]
    #[tokio::test]
    async fn test_new_authority_with_wrong_signer() {
        let program_id = id();
        let mut program_test = ProgramTest::new("sallar", program_id, processor!(entry));
        program_test.set_compute_max_units(500000);

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
        program_test.prefer_bpf(true);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let signer = payer.pubkey();

        initialize_instruction(&mut banks_client, &payer, recent_blockhash)
            .await
            .unwrap();

        let (_, _, blocks_state_pda, _, _, _, _, _, _, _, _, _) = get_pda_accounts();

        let data = instruction::ChangeAuthority {
            new_authority: signer,
        }
        .data();

        let sub_signer = Keypair::new();
        let accs = accounts::ChangeAuthorityContext {
            blocks_state_account: blocks_state_pda,
            signer: sub_signer.pubkey(),
        };

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &data,
                accs.to_account_metas(Some(false)),
            )],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer, &sub_signer], recent_blockhash);
        let error = banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap();
        assert_eq!(get_custom_error_code(error).unwrap(), 6000);
    }

    async fn create_token_account(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: Hash,
        mint: Pubkey,
    ) -> Result<Pubkey> {
        let rent = Rent::default();
        let new_keypair = Keypair::new();
        let transaction = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &payer.pubkey(),
                    &new_keypair.pubkey(),
                    rent.minimum_balance(Account::LEN),
                    Account::LEN.try_into().unwrap(),
                    &spl_token::id(),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &new_keypair.pubkey(),
                    &mint,
                    &payer.pubkey(),
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer, &new_keypair],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        Ok(new_keypair.pubkey())
    }

    fn get_pda_accounts() -> (
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
        Pubkey,
        u8,
    ) {
        let program_id = id();

        let (mint_pda, mint_bump) =
            Pubkey::find_program_address(&[MINT_SEED.as_bytes()], &program_id);
        let (blocks_state_pda, blocks_state_bump) =
            Pubkey::find_program_address(&[BLOCKS_STATE_SEED.as_bytes()], &program_id);
        let (distribution_top_block_pda, distribution_top_block_bump) =
            Pubkey::find_program_address(&[DISTRIBUTION_TOP_BLOCK_SEED.as_bytes()], &program_id);
        let (distribution_bottom_block_pda, distribution_bottom_block_bump) =
            Pubkey::find_program_address(&[DISTRIBUTION_BOTTOM_BLOCK_SEED.as_bytes()], &program_id);
        let (final_staking_account_pda, final_staking_account_bump) =
            Pubkey::find_program_address(&[FINAL_STAKING_ACCOUNT_SEED.as_bytes()], &program_id);
        let (final_mining_account_pda, final_mining_account_bump) =
            Pubkey::find_program_address(&[FINAL_MINING_ACCOUNT_SEED.as_bytes()], &program_id);

        (
            mint_pda,
            mint_bump,
            blocks_state_pda,
            blocks_state_bump,
            distribution_top_block_pda,
            distribution_top_block_bump,
            distribution_bottom_block_pda,
            distribution_bottom_block_bump,
            final_staking_account_pda,
            final_staking_account_bump,
            final_mining_account_pda,
            final_mining_account_bump,
        )
    }

    #[cfg(feature = "bpf-tests")]
    async fn set_time(ctx: &mut ProgramTestContext, time: i64) {
        let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
        let mut new_clock = clock_sysvar.clone();
        new_clock.epoch = new_clock.epoch + 30;
        new_clock.unix_timestamp = time;

        ctx.set_sysvar(&new_clock);
    }

    #[cfg(feature = "bpf-tests")]
    fn get_custom_error_code(error: TransactionError) -> Option<u32> {
        if let TransactionError::InstructionError(_, InstructionError::Custom(error_code)) = error {
            Some(error_code)
        } else {
            None
        }
    }
}
