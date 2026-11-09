use anchor_lang::{
    context,
    prelude::{
        require, Account, AccountInfo, Clock, CpiContext, Result, SolanaSysvar, ToAccountInfo,
    },
    solana_program::program::invoke_signed,
};
use anchor_spl::token::{self, Mint, MintTo, TokenAccount, Transfer};

use mpl_token_metadata::instruction::create_metadata_accounts_v3;

use crate::{
    account::BlocksState, context as SallarContext, error::SallarError,
    token_math::calculate_max_bp, token_math::DUSTS_PER_BLOCK, MINT_SEED,
};
use context::*;
use SallarContext::InitializeContext;

const MIN_BLOCKS_SOLUTION_INTERVAL_SECONDS: i64 = 180;
const MIN_FINAL_STAKING_SOLUTION_INTERVAL_SECONDS: i64 = 72_000;

/// Transfers tokens between two accounts.
///
/// ### Arguments
///
/// * `authority` - the authority that is going to transfer the tokens, it also the source account,
/// * `to` - the destination account,
/// * `program_account_seed` - the seed of the program account,
/// * `program_account` - the program account,
/// * `program_account_nonce` - the nonce of the program account,
/// * `amount` - the amount of tokens to transfer.
///
/// ### Returns
/// The result of the transfer
pub fn transfer_tokens<'a>(
    authority: &Box<Account<'a, TokenAccount>>,
    to: AccountInfo<'a>,
    program_account_seed: &'a str,
    program_account: AccountInfo<'a>,
    program_account_nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[program_account_seed.as_bytes(), &[program_account_nonce]];
    let signer_seeds = &[&seeds[..]];

    let from = authority.to_account_info();
    let authority = authority.to_account_info();

    let cpi_accounts = Transfer {
        from,
        to,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(
        program_account.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    token::transfer(cpi_ctx, amount)
}

/// Mints tokens to given account.
///
/// ### Arguments
///
/// * `mint` - the mint account,
/// * `to` - the destination account,
/// * `authority` - the authority that is used to mint the tokens,
/// * `program_account` - the program account,
/// * `mint_nonce` - the nonce of the mint account,
/// * `amount` - the amount of tokens to transfer.
///
/// ### Returns
/// The result of the minting
pub fn mint_tokens<'a>(
    mint: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    program_account: AccountInfo<'a>,
    mint_nonce: u8,
    amount: u64,
) -> Result<()> {
    let seeds = &[MINT_SEED.as_bytes(), &[mint_nonce]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = MintTo {
        mint,
        to,
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(program_account, cpi_accounts, signer_seeds);

    token::mint_to(cpi_ctx, amount)
}

/// Asserts that the signer is authorized to perform the action, i.e. if the signer is contract's owner.
///
/// ### Arguments
///
/// * `state` - the current state of the contract,
/// * `signer` - the account which is the signer of the current transaction.
///
/// ### Returns
/// An error if the signer is not an owner of the contract, otherwise a successful result.
pub fn valid_owner(state: &BlocksState, signer: &AccountInfo) -> Result<()> {
    require!(signer.key.eq(&state.authority), SallarError::Unauthorized);

    Ok(())
}

/// Asserts that the given account is a signer.
///
/// ### Arguments
///
/// * `signer` - the account which is supposed to be a signer.
///
/// ### Returns
/// An error if the account is not a signer, otherwise a successful result.
pub fn valid_signer(signer: &AccountInfo) -> Result<()> {
    require!(signer.is_signer, SallarError::Unauthorized);

    Ok(())
}

/// Asserts that required time (3 minutes) passed since last block solution.
/// It supports both: top and bottom blocks as both of them have require the same time interval between solved blocks.
///
/// ### Arguments
///
/// * `last_solved_block_timestamp` - timestamp of the moment when last block was solved (either top or bottom).
///
/// ### Returns
/// An error if less than 3 minutes passed since last block solution, otherwise a successful result.
pub fn blocks_solution_required_interval_elapsed(last_solved_block_timestamp: &i64) -> Result<()> {
    require!(
        Clock::get()?.unix_timestamp - last_solved_block_timestamp
            >= MIN_BLOCKS_SOLUTION_INTERVAL_SECONDS,
        SallarError::BlockSolutionAheadOfTime
    );

    Ok(())
}

/// Asserts that required time (20 hours) passed since last completed final staking.
///
/// ### Arguments
///
/// * `last_completed_final_staking_timestamp` - timestamp of the moment when last block final staking was completed.
///
/// ### Returns
/// An error if less than 20 hours passed since last completed final staking, otherwise a successful result.
pub fn final_staking_required_interval_elapsed(
    last_completed_final_staking_timestamp: &i64,
) -> Result<()> {
    require!(
        Clock::get()?.unix_timestamp - last_completed_final_staking_timestamp
            >= MIN_FINAL_STAKING_SOLUTION_INTERVAL_SECONDS,
        SallarError::FinalStakingAheadOfTime
    );

    Ok(())
}

/// Asserts that blocks have collided, i.e. that `blocks_collided` attribute of the current `BlocksState` is set to true
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// An error if blocks have not collided yet, otherwise a successful result.
pub fn blocks_collided(state: &BlocksState) -> Result<()> {
    require!(state.blocks_collided, SallarError::BlocksNotCollidedYet);

    Ok(())
}

/// Asserts that the current top block is not solved yet, i.e. it has some available BPs.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// An error if top block is solved (has no available BPs), otherwise a successful result.
pub fn top_block_not_solved(state: &BlocksState) -> Result<()> {
    require!(
        state.top_block_available_bp > 0,
        SallarError::BlockAlreadySolved
    );

    Ok(())
}

/// Asserts that the current bottom block is not solved yet, i.e. it has some available BPs.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// An error if bottom block is solved (has no available BPs), otherwise a successful result.
pub fn bottom_block_not_solved(state: &BlocksState) -> Result<()> {
    require!(
        state.bottom_block_available_bp > 0,
        SallarError::BlockAlreadySolved
    );

    Ok(())
}

/// Asserts that the both top block and bottom block are solved, i.e. they have no available BPs.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// An error if either top block or bottom is not solved (any of them has available BPs), otherwise a successful result.
pub fn blocks_solved(state: &BlocksState) -> Result<()> {
    require!(
        state.top_block_available_bp == 0,
        SallarError::TopBlockNotSolvedYet
    );
    require!(
        state.bottom_block_available_bp == 0,
        SallarError::BottomBlockNotSolvedYet
    );

    Ok(())
}

/// Sets `blocks_collided` attribute of `BlocksState` to true to mark blocks as collided.
/// It happens only if blocks really collided, i.e. bottom block's number is great by 1 than top block's number.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// A successful result.
pub fn update_blocks_collided(state: &mut BlocksState) -> Result<()> {
    if !can_block_be_switched(state) {
        state.blocks_collided = true;
    }

    Ok(())
}

/// Asserts that initial_token_distribution function has not yet been successfully executed.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// An error if the initial_token_distribution function has been already successfully executed, otherwise a successful result.
pub fn initial_token_distribution_not_performed_yet(state: &BlocksState) -> Result<()> {
    require!(
        !state.initial_token_distribution_already_performed,
        SallarError::InitialTokenDistributionAlreadyPerformed
    );

    Ok(())
}

/// Specifies if any block can be switched to the next one.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state).
///
/// ### Returns
/// True if current bottom block number is greater than by current top block number by at least 2, false otherwise.
pub fn can_block_be_switched(state: &BlocksState) -> bool {
    state.bottom_block_number - 1 > state.top_block_number
}

/// Switches top block to the next one if the current one is already solved.
/// It updates top block related attributes of `BlocksState`:
/// - `top_block_solution_timestamp` to update timestamp of recently solved block to the current one,
/// - `top_block_number` - sets next block's number (current block's number + 1),
/// - `top_block_available_bp` - sets current block's BP to the max BP for the new current block (after switching its number),
/// - `top_block_balance` - sets current block's balance to the max block's balance (an initial one).
/// It also mints tokens to top block distribution account for the new block.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state),
/// * `mint_nonce` - the nonce of mint account,
/// * `mint` - reference to mint account,
/// * `distribution_top_block_account` - reference to top block distribution account where new tokens will be minted,
/// * `token_program` - the program account for the token being used.
///
/// ### Errors
/// This function can return a `MismatchBetweenAvailableBlockBPAndBalance` error if the balance and the available block's BP of the bottom block do not match.
///
/// ### Returns
/// A successful result.
pub fn switch_top_block_to_next_one_if_applicable<'a>(
    state: &mut BlocksState,
    mint_nonce: u8,
    mint: &Box<Account<'a, Mint>>,
    distribution_top_block_account: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
) -> Result<()> {
    require!(
        (state.top_block_balance == 0 && state.top_block_available_bp == 0)
            || (state.top_block_balance > 0 && state.top_block_available_bp > 0),
        SallarError::MismatchBetweenAvailableBlockBPAndBalance
    );

    if state.top_block_available_bp == 0 && can_block_be_switched(state) {
        state.top_block_solution_timestamp = Clock::get()?.unix_timestamp;
        state.top_block_number += 1;

        let authority = mint.to_account_info();
        let mint_token_account = mint.to_account_info();

        mint_tokens(
            authority,
            distribution_top_block_account,
            mint_token_account,
            token_program,
            mint_nonce,
            DUSTS_PER_BLOCK,
        )?;

        state.top_block_available_bp =
            convert_f64_to_u64(calculate_max_bp(state.top_block_number)?)?;
        state.top_block_balance = DUSTS_PER_BLOCK;
    }

    Ok(())
}

/// Switches bottom block to the next one if the current one is already solved.
/// It updates bottom block related attributes of `BlocksState`:
/// - `bottom_block_solution_timestamp` to update timestamp of recently solved block to the current one,
/// - `bottom_block_number` - sets next block's number (current block's number + 1),
/// - `bottom_block_available_bp` - sets current block's BP to the max BP for the new current block (after switching its number),
/// - `bottom_block_balance` - sets current block's balance to the max block's balance (an initial one).
/// It also mints tokens to bottom block distribution account for the new block.
///
/// ### Arguments
///
/// * `state` - contract's state (blocks state),
/// * `mint_nonce` - the nonce of mint account,
/// * `mint` - reference to mint account,
/// * `distribution_bottom_block_account` - reference to bottom block distribution account where new tokens will be minted,
/// * `token_program` - the program account for the token being used.
///
/// ### Errors
/// This function can return a `MismatchBetweenAvailableBlockBPAndBalance` error if the balance and the available block's BP of the bottom block do not match.
///
/// ### Returns
/// A successful result.
pub fn switch_bottom_block_to_next_one_if_applicable<'a>(
    state: &mut BlocksState,
    mint_nonce: u8,
    mint: &Box<Account<'a, Mint>>,
    distribution_bottom_block_account: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
) -> Result<()> {
    require!(
        (state.bottom_block_balance == 0 && state.bottom_block_available_bp == 0)
            || (state.bottom_block_balance > 0 && state.bottom_block_available_bp > 0),
        SallarError::MismatchBetweenAvailableBlockBPAndBalance
    );

    if state.bottom_block_available_bp == 0 && can_block_be_switched(state) {
        state.bottom_block_solution_timestamp = Clock::get()?.unix_timestamp;
        state.bottom_block_number -= 1;

        let authority = mint.to_account_info();
        let mint_token_account = mint.to_account_info();

        mint_tokens(
            authority,
            distribution_bottom_block_account,
            mint_token_account,
            token_program,
            mint_nonce,
            DUSTS_PER_BLOCK,
        )?;

        state.bottom_block_available_bp =
            convert_f64_to_u64(calculate_max_bp(state.bottom_block_number)?)?;
        state.bottom_block_balance = DUSTS_PER_BLOCK;
    }

    Ok(())
}

/// Converts a given `f64` value to an `u64` value and returns it as a result.
/// Performs various checks to ensure that the conversion can be performed,
/// i.e. provided `f64` number is in the range of `u64`.
///
/// The conversion is not safe in the context of precision
/// so there is no guarantee that provided `f64` number will be exactly the same number
/// as in `u64` after the conversion.
///
/// ### Arguments
///
/// * value - the f64 value to be converted to u64
///
/// ### Returns
///
/// The result of the conversion if the input value is in the scope of `u64`, or an error otherwise.
pub fn convert_f64_to_u64(value: f64) -> Result<u64> {
    require!(value <= u64::MAX as f64, SallarError::U64ConversionError);
    require!(value >= u64::MIN as f64, SallarError::U64ConversionError);

    Ok(value as u64)
}

/// Converts a given `u64` value to an `f64` value and returns it as a result.
///
/// ### Arguments
///
/// * value - the u64 value to be converted to f64
///
/// ### Returns
///
/// The result of the conversion.
///
pub fn convert_u64_to_f64(value: u64) -> Result<f64> {
    Ok(value as f64)
}

/// Sets token metadata
///
/// ### Arguments
///
/// * `name` - token name
/// * `symbol` - token symbol
/// * `uri` - token uri
pub fn set_token_metadata(
    ctx: Context<InitializeContext>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let program_id = ctx.accounts.metadata_program.to_account_info();
    let metadata_pda = ctx.accounts.metadata_pda.to_account_info();
    let mint = ctx.accounts.mint.to_account_info();
    let mint_authority = ctx.accounts.mint.to_account_info();
    let payer = ctx.accounts.signer.to_account_info();
    let update_authority = ctx.accounts.mint.to_account_info();
    let system_program = ctx.accounts.system_program.to_account_info();

    let seeds = &[
        MINT_SEED.as_bytes(),
        &[ctx.accounts.blocks_state_account.mint_nonce],
    ];

    let account_infos = &[
        program_id.clone(),
        metadata_pda.clone(),
        mint.clone(),
        mint_authority.clone(),
        payer.clone(),
        update_authority.clone(),
        system_program.clone(),
    ];

    let create_metadata_accounts_instruction = create_metadata_accounts_v3(
        *program_id.key,
        *metadata_pda.key,
        *mint.key,
        *mint_authority.key,
        *payer.key,
        *update_authority.key,
        name.clone(),
        symbol.clone(),
        uri.clone(),
        None,
        0u16,
        false,
        true,
        None,
        None,
        None,
    );

    invoke_signed(
        &create_metadata_accounts_instruction,
        account_infos,
        &[seeds],
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use anchor_lang::err;
    use anchor_lang::prelude::AccountInfo;
    use anchor_lang::prelude::Pubkey;

    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    impl PartialEq for BlocksState {
        fn eq(&self, other: &Self) -> bool {
            self.top_block_number == other.top_block_number
                && self.block_state_nonce == other.block_state_nonce
                && self.top_block_balance == other.top_block_balance
                && self.top_block_available_bp == other.top_block_available_bp
                && self.top_block_solution_timestamp == other.top_block_solution_timestamp
                && self.bottom_block_number == other.bottom_block_number
                && self.bottom_block_balance == other.bottom_block_balance
                && self.bottom_block_available_bp == other.bottom_block_available_bp
                && self.bottom_block_solution_timestamp == other.bottom_block_solution_timestamp
                && self.blocks_collided == other.blocks_collided
                && self.initial_token_distribution_already_performed
                    == other.initial_token_distribution_already_performed
        }
    }

    impl std::fmt::Debug for BlocksState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BlocksState")
                .field("block_state_nonce", &self.block_state_nonce)
                .field("top_block_number", &self.top_block_number)
                .field("top_block_balance", &self.top_block_balance)
                .field("top_block_available_bp", &self.top_block_available_bp)
                .field(
                    "top_block_solution_timestamp",
                    &self.top_block_solution_timestamp,
                )
                .field(
                    "top_block_last_account_address",
                    &self.top_block_last_account_address,
                )
                .field(
                    "top_block_last_account_rest_bp",
                    &self.top_block_last_account_rest_bp,
                )
                .field("bottom_block_number", &self.bottom_block_number)
                .field("bottom_block_balance", &self.bottom_block_balance)
                .field("bottom_block_available_bp", &self.bottom_block_available_bp)
                .field(
                    "bottom_block_solution_timestamp",
                    &self.bottom_block_solution_timestamp,
                )
                .field(
                    "bottom_block_last_account_address",
                    &self.bottom_block_last_account_address,
                )
                .field(
                    "bottom_block_last_account_rest_bp",
                    &self.bottom_block_last_account_rest_bp,
                )
                .field("blocks_collided", &self.blocks_collided)
                .field(
                    "initial_token_distribution_already_performed",
                    &self.initial_token_distribution_already_performed,
                )
                .field("authority", &self.authority)
                .finish()
        }
    }

    impl BlocksState {
        pub fn default() -> Self {
            BlocksState {
                block_state_nonce: 0,
                top_block_number: 0,
                top_block_balance: 0,
                top_block_available_bp: 0,
                top_block_solution_timestamp: 0,
                top_block_last_account_address: Some(Pubkey::new_unique()),
                top_block_last_account_rest_bp: 0,
                bottom_block_number: 0,
                bottom_block_balance: 0,
                bottom_block_available_bp: 0,
                bottom_block_solution_timestamp: 0,
                bottom_block_last_account_address: Some(Pubkey::new_unique()),
                bottom_block_last_account_rest_bp: 0,
                blocks_collided: false,
                initial_token_distribution_already_performed: false,
                authority: Pubkey::new_unique(),
                mint_nonce: 0,
                top_block_distribution_address: Pubkey::new_unique(),
                top_block_distribution_nonce: 0,
                bottom_block_distribution_address: Pubkey::new_unique(),
                bottom_block_distribution_nonce: 0,
                final_staking_account_nonce: 0,
                final_staking_pool_in_round: 0,
                final_staking_last_staking_timestamp: 0,
                final_staking_left_reward_parts_in_round: 0.0,
                final_staking_left_balance_in_round: 0,
                final_mining_account_nonce: 0,
            }
        }
    }

    #[test]
    fn test_valid_signer() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let mut binding = 0u64;
        let deps = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: true,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        valid_signer(&deps).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_valid_signer() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let mut binding = 0u64;
        let deps = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        valid_signer(&deps).unwrap();
    }

    #[test]
    fn test_valid_owner() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let authority = Pubkey::new_unique();
        let mut binding = 0u64;

        let signer = AccountInfo {
            key: &authority,
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let state = BlocksState {
            authority,
            ..BlocksState::default()
        };

        valid_owner(&state, &signer).unwrap()
    }

    #[test]
    fn test_blocks_solved() {
        let mut state = BlocksState::default();
        state.top_block_available_bp = 0;
        state.bottom_block_available_bp = 0;
        blocks_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_blocks_solved_top_block_unlock() {
        let mut state = BlocksState::default();
        state.top_block_available_bp = 1;
        state.bottom_block_available_bp = 0;
        blocks_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_blocks_solved_bottom_block_unlock() {
        let mut state = BlocksState::default();
        state.top_block_available_bp = 0;
        state.bottom_block_available_bp = 1;
        blocks_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_blocks_solved_all_blocks_unlock() {
        let mut state = BlocksState::default();
        state.top_block_available_bp = 1;
        state.bottom_block_available_bp = 1;
        blocks_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_valid_owner() {
        let data: Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut [0u8; 0]));
        let authority = Pubkey::new_unique();
        let mut binding = 0u64;

        let signer = AccountInfo {
            key: &authority,
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding)),
            data,
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let state = BlocksState {
            authority: Pubkey::new_unique(),
            ..BlocksState::default()
        };

        valid_owner(&state, &signer).unwrap()
    }

    #[test]
    fn test_blocks_collided() {
        let mut state = BlocksState::default();
        state.blocks_collided = true;

        blocks_collided(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_blocks_collided() {
        let mut state = BlocksState::default();
        state.blocks_collided = false;

        blocks_collided(&state).unwrap();
    }

    #[test]
    fn test_update_blocks_collided() {
        let mut state = BlocksState::default();
        state.bottom_block_number = 1;
        state.blocks_collided = true;

        update_blocks_collided(&mut state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_update_blocks_collided() {
        let mut state = BlocksState::default();
        state.blocks_collided = false;

        update_blocks_collided(&mut state).unwrap();
    }

    #[test]
    fn test_initial_token_distribution_not_performed_yet() {
        let mut state = BlocksState::default();
        state.initial_token_distribution_already_performed = false;

        initial_token_distribution_not_performed_yet(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_initial_token_distribution_not_performed_yet() {
        let mut state = BlocksState::default();
        state.initial_token_distribution_already_performed = true;

        initial_token_distribution_not_performed_yet(&state).unwrap();
    }

    #[test]
    fn test_can_block_be_switched() {
        let mut state = BlocksState::default();
        state.top_block_number = 1;
        state.bottom_block_number = 2;

        assert_eq!(can_block_be_switched(&state), false);
    }

    #[test]
    #[should_panic]
    fn test_fail_can_block_be_switched() {
        let mut state = BlocksState::default();
        state.top_block_number = 1;
        state.bottom_block_number = 3;

        assert_eq!(can_block_be_switched(&state), false);
    }

    #[test]
    fn test_top_block_not_solved() {
        let mut state = BlocksState::default();
        state.top_block_available_bp = 1;

        top_block_not_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_top_block_not_solved() {
        let mut state = BlocksState::default();
        state.top_block_available_bp = 0;

        top_block_not_solved(&state).unwrap();
    }

    #[test]
    fn test_bottom_block_not_solved() {
        let mut state = BlocksState::default();
        state.bottom_block_available_bp = 1;

        bottom_block_not_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_fail_bottom_block_not_solved() {
        let mut state = BlocksState::default();
        state.bottom_block_available_bp = 0;

        bottom_block_not_solved(&state).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_final_staking_required_interval_elapsed() {
        final_staking_required_interval_elapsed(&0).unwrap();
    }

    #[test]
    fn test_convert_f64_to_u64_valid() {
        assert_eq!(convert_f64_to_u64((u64::MIN) as f64), Ok(0));
        assert_eq!(convert_f64_to_u64(123.0), Ok(123));
        assert_eq!(convert_f64_to_u64((u64::MAX) as f64), Ok(u64::MAX));

        // This is the maximum value that can be represented by f64
        // without losing precision
        assert_eq!(
            convert_f64_to_u64((1u64 << 53) as f64),
            Ok(9007199254740992)
        );

        // This value cannot be represented by f64
        // without losing precision but it is acceptable
        assert_eq!(
            convert_f64_to_u64((1u64 << 53) as f64 + 1.0),
            Ok(9007199254740992)
        );

        // This value cannot be represented by f64
        // without losing precision but it is acceptable
        assert_eq!(
            convert_f64_to_u64((1u64 << 53) as f64 + 2.0),
            Ok(9007199254740994)
        );
    }

    #[test]
    fn test_convert_f64_to_u64_invalid() {
        assert_eq!(
            convert_f64_to_u64(f64::MIN),
            err!(SallarError::U64ConversionError)
        );
        assert_eq!(
            convert_f64_to_u64(-1.0),
            err!(SallarError::U64ConversionError)
        );
        assert_eq!(
            convert_f64_to_u64(-0.1),
            err!(SallarError::U64ConversionError)
        );
        assert_eq!(
            convert_f64_to_u64(18446744073709553665.0),
            err!(SallarError::U64ConversionError)
        );
        assert_eq!(
            convert_f64_to_u64(f64::MAX),
            err!(SallarError::U64ConversionError)
        );
    }

    #[test]
    fn test_convert_u64_to_f64_valid() {
        assert_eq!(convert_u64_to_f64(u64::MIN), Ok(0.0));
        assert_eq!(convert_u64_to_f64(u64::MAX), Ok(18446744073709551615.0));
        // The same value as for u64::MAX as the precision is lost in this case
        // but it is acceptable
        assert_eq!(convert_u64_to_f64(u64::MAX - 1), Ok(18446744073709551615.0));

        // This is the maximum value that can be represented by f64
        // without losing precision
        assert_eq!(convert_u64_to_f64(9007199254740992), Ok(9007199254740992.0));

        // This value cannot be represented by f64
        // without losing precision but it is acceptable
        assert_eq!(convert_u64_to_f64(9007199254740993), Ok(9007199254740992.0));

        // This value cannot be represented by f64
        // without losing precision but it is acceptable
        assert_eq!(convert_u64_to_f64(9007199254740994), Ok(9007199254740994.0));
    }

    #[test]
    fn test_can_block_be_switched_true() {
        let mut state = BlocksState::default();
        state.bottom_block_number = 3;
        state.top_block_number = 1;

        assert!(can_block_be_switched(&state));
    }

    #[test]
    fn test_can_block_be_switched_false_equal() {
        let mut state = BlocksState::default();
        state.bottom_block_number = 2;
        state.top_block_number = 2;

        assert!(!can_block_be_switched(&state));
    }

    #[test]
    fn test_can_block_be_switched_false_less() {
        let mut state = BlocksState::default();
        state.bottom_block_number = 1;
        state.top_block_number = 2;

        assert!(!can_block_be_switched(&state));
    }

    #[test]
    fn test_can_block_be_switched_false_difference_one() {
        let mut state = BlocksState::default();
        state.bottom_block_number = 2;
        state.top_block_number = 1;

        assert!(!can_block_be_switched(&state));
    }

    #[cfg(feature = "bpf-tests")]
    #[test]
    #[should_panic]
    fn test_mint_tokens() {
        let mut binding_mint = 0u64;
        let mut binding_to = 0u64;
        let mut binding_authority = 0u64;
        let mut binding_program_account = 0u64;

        let mint: AccountInfo = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding_mint)),
            data: Rc::new(RefCell::new(&mut [0u8; 0])),
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };

        let to: AccountInfo = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding_to)),
            data: Rc::new(RefCell::new(&mut [0u8; 0])),
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let authority: AccountInfo = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding_authority)),
            data: Rc::new(RefCell::new(&mut [0u8; 0])),
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let program_account: AccountInfo = AccountInfo {
            key: &Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut binding_program_account)),
            data: Rc::new(RefCell::new(&mut [0u8; 0])),
            owner: &Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        };
        let mint_nonce = 0;
        let amount = 0;

        mint_tokens(mint, to, authority, program_account, mint_nonce, amount).unwrap();
    }
}
