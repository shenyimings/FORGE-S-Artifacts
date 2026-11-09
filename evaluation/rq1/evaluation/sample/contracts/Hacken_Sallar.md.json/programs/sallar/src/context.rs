use anchor_lang::{
    error,
    prelude::{
        account, borsh, require_keys_neq, Account, AccountInfo, Accounts, AnchorDeserialize, Key,
        Program, Pubkey, Rent, Signer, SolanaSysvar, System, ToAccountInfo,
    },
    solana_program::system_program,
    Id, Space,
};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata;

use crate::{
    account::BlocksState, BLOCKS_STATE_SEED, DISTRIBUTION_BOTTOM_BLOCK_SEED,
    DISTRIBUTION_TOP_BLOCK_SEED, FINAL_MINING_ACCOUNT_SEED, FINAL_STAKING_ACCOUNT_SEED, MINT_SEED,
};

/// The discriminator is defined by the first 8 bytes of the SHA256 hash of the account's Rust identifier.
/// It includes the name of struct type and lets Anchor know what type of account it should deserialize the data as.
const DISCRIMINATOR_LENGTH: usize = 8;

/// Context for the initialize instruction.
///
/// This context is used to initialize the contract state.
///
/// The contract state is initialized with the following accounts:
///
/// - `blocks_state_account` - the account that contains the contract state,
/// - `mint` - the mint account,
/// - `distribution_top_block_account` - the top block distribution account,
/// - `distribution_bottom_block_account` - the bottom block distribution account,
/// - `final_staking_account` - the final staking account,
/// - `final_mining_account` - the final mining account,
///
/// The context includes also:
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes current contract's owner,
/// - `system_program` - the Solana system program account.
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeContext<'info> {
    #[account(init, payer = signer, space = DISCRIMINATOR_LENGTH + BlocksState::INIT_SPACE, seeds = [BLOCKS_STATE_SEED.as_bytes()], bump)]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,

    /// Decimals are set to 8 because it is the highest possible precision,
    /// considering the desired total supply of Sallar which is 54_600_000_000.
    ///
    /// Adding 8 digits there results in a number that still fits u64 range:
    /// 54_600_000_000_000_000_00 (total supply in base units - dusts - with 8 digits)
    /// 18_446_744_073_709_551_615 (max number in u64)
    ///
    /// Increasing decimals to 9 would result in a number exceeding u64 range.
    #[account(
        init,
        payer = signer,
        seeds = [MINT_SEED.as_bytes()],
        bump,
        mint::decimals = 8,
        mint::authority = mint
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = distribution_top_block_account,
        seeds = [DISTRIBUTION_TOP_BLOCK_SEED.as_bytes()],
        bump,
    )]
    pub distribution_top_block_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = distribution_bottom_block_account,
        seeds = [DISTRIBUTION_BOTTOM_BLOCK_SEED.as_bytes()],
        bump,
    )]
    pub distribution_bottom_block_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = final_staking_account,
        seeds = [FINAL_STAKING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub final_staking_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = final_mining_account,
        seeds = [FINAL_MINING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub final_mining_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: The metadata program account. It is considered safe because it is checked by the inner instruction, ensuring it is the correct account.
    #[account(mut, address = Pubkey::find_program_address(&[b"metadata", &mpl_token_metadata::id().to_bytes(), &mint.key().to_bytes()], &mpl_token_metadata::id()).0)]
    pub metadata_pda: AccountInfo<'info>,

    /// CHECK: The metadata program account. It is considered safe because it is checked by the inner instruction, ensuring it is the correct account.
    #[account(address = mpl_token_metadata::id())]
    pub metadata_program: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

/// Context for the initial_token_distribution instruction.
///
/// This context is used to mint some tokens to organization account provided in the context.
///
/// Attributes:
/// - `blocks_state_account` - the blocks state account defining current contract's state,
/// - `mint` - the mint account,
/// - `organization_account` - the account that receives the tokens minted by initial_token_distribution function,
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes contract's owner.
#[derive(Accounts)]
pub struct InitialTokenDistributionContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = blocks_state_account.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub organization_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state_account.authority)]
    pub signer: Signer<'info>,
}

/// Context for the solve_top_block instruction.
///
/// This context is used to solve top blocks and distribute tokens from top block distribution account to users solving current top block.
///
/// Attributes:
/// - `blocks_state_account` - the blocks state account defining current contract's state,
/// - `distribution_top_block_account` - the top block distribution account,
/// - `mint` - the mint account,
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes contract's owner.
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SolveTopBlockContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [DISTRIBUTION_TOP_BLOCK_SEED.as_bytes()],
        bump = blocks_state_account.top_block_distribution_nonce
    )]
    pub distribution_top_block_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = blocks_state_account.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state_account.authority)]
    pub signer: Signer<'info>,
}

/// Context for the solve_bottom_block instruction.
///
/// This context is used to solve bottom blocks and distribute tokens from bottom block distribution account to users solving current bottom block.
///
/// Attributes:
/// - `blocks_state_account` - the blocks state account defining current contract's state,
/// - `distribution_bottom_block_account` - the bottom block distribution account,
/// - `mint` - the mint account,
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes contract's owner.
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SolveBottomBlockContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [DISTRIBUTION_BOTTOM_BLOCK_SEED.as_bytes()],
        bump = blocks_state_account.bottom_block_distribution_nonce,
    )]
    pub distribution_bottom_block_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = blocks_state_account.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state_account.authority)]
    pub signer: Signer<'info>,
}

/// Context for the final_staking instruction.
///
/// This context is used to execute final staking process and distribute tokens to accounts participating in the process.
///
/// Attributes:
/// - `blocks_state_account` - the blocks state account defining current contract's state,
/// - `final_staking_account` - the final staking account,
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes contract's owner.
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct FinalStakingContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [FINAL_STAKING_ACCOUNT_SEED.as_bytes()],
        bump = blocks_state_account.final_staking_account_nonce,
    )]
    pub final_staking_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state_account.authority)]
    pub signer: Signer<'info>,
}

/// Context for the final_mining instruction.
///
/// This context is used to execute final mining process and distribute tokens to accounts participating in the process.
///
/// Attributes:
/// - `blocks_state_account` - the blocks state account defining current contract's state,
/// - `final_mining_account` - the final mining account,
/// - `token_program` - the Solana token program account,
/// - `signer` - the signer of the transaction which executes initialize instruction, the signer becomes contract's owner.
#[derive(Accounts)]
pub struct FinalMiningContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [FINAL_MINING_ACCOUNT_SEED.as_bytes()],
        bump = blocks_state_account.final_mining_account_nonce,
    )]
    pub final_mining_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state_account.authority)]
    pub signer: Signer<'info>,
}

/// Context for the change_authority instruction.
///
/// This context is used to set new authority on contract state.
///
/// The context includes:
/// - `blocks_state_account` - the blocks state account defining current contract's state,
/// - `signer` - the signer of the transaction which must be the contract's owner.
#[derive(Accounts)]
pub struct ChangeAuthorityContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetBlocksCollidedContext<'info> {
    #[account(
        mut,
        seeds = [BLOCKS_STATE_SEED.as_bytes()],
        bump = blocks_state_account.block_state_nonce,
    )]
    pub blocks_state_account: Box<Account<'info, BlocksState>>,
    pub signer: Signer<'info>,
}
