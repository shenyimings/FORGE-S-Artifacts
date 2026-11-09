use anchor_lang::{
    prelude::{account, borsh, AnchorDeserialize, AnchorSerialize, InitSpace},
    solana_program::pubkey::Pubkey,
};

/// Struct defining the current blocks state in the program.
/// Consists of the following attributes:
/// * `authority` - the authority that initialized the contract, an owner of the contract,
/// * `block_state_nonce` - the nonce of the block state account,
/// * `mint_nonce` - the nonce of the mint account,
///
/// * `initial_token_distribution_already_performed` - true if initial_token_distribution function was already invoked and completed successfully, false otherwise,
/// * `blocks_collided` - true if blocks cannot be switched to the next ones, i.e. the current top block number is less than the current bottom block number by 1,
///
/// * `top_block_number` - current top block number,
/// * `top_block_available_bp` - the number of left bp for the current top block number (when bp is decreased to 0, then the current block is solved),
/// * `top_block_solution_timestamp` - the timestamp of recently solved top block,
/// * `top_block_balance` - amount of tokens left on the current top block to be distributed as the part of the block solution process,
/// * `top_block_distribution_address` - the address of the top block distribution account,
/// * `top_block_distribution_nonce` - the nonce of the top block distribution account,
/// * `top_block_last_account_address` - address of the last account that participated in top block solving,
/// * `top_block_last_account_rest_bp` - the number of BP that the last account - that participated in top block solving - did not receive due to too low amount of remaining BP on the block,
///
/// * `bottom_block_number` - current bottom block number,
/// * `bottom_block_available_bp` - the number of left bp for the current bottom block number (when bp is decreased to 0, then the current block is solved),
/// * `bottom_block_solution_timestamp` - the timestamp of recently solved bottom block,
/// * `bottom_block_balance` - amount of tokens left on the current bottom block to be distributed as the part of the block solution process,
/// * `bottom_block_distribution_address` - the address of the bottom block distribution account,
/// * `bottom_block_distribution_nonce` - the nonce of the bottom block distribution account,
/// * `bottom_block_last_account_address` - address of the last account that participated in bottom block solving,
/// * `bottom_block_last_account_rest_bp` - the number of BP that the last account - that participated in bottom block solving - did not receive due to too low amount of remaining BP on the block,
///
/// * `final_staking_account_nonce` - the nonce of the final staking account,
/// * `final_staking_pool_in_round` - prize pool (amount of tokens) to be distributed in the current final staking round,
/// * `final_staking_last_staking_timestamp` - the timestamp of the recently completed final staking round,
/// * `final_staking_left_reward_parts_in_round` - the number of left reward parts for the current final staking round (the number starts at 1.0 and is decreased by reward parts of the input accounts participating in the final staking process) - final staking round is completed when this number is decreased to 0,
/// * `final_staking_left_balance_in_round` - left amount of tokens to be distributed in the current final staking round,
///
/// * `final_mining_account_nonce` - the nonce of the final mining account.
#[account]
#[derive(InitSpace)]
pub struct BlocksState {
    pub authority: Pubkey,
    pub block_state_nonce: u8,
    pub mint_nonce: u8,

    pub initial_token_distribution_already_performed: bool,
    pub blocks_collided: bool,

    pub top_block_number: u64,
    pub top_block_available_bp: u64,
    pub top_block_solution_timestamp: i64,
    pub top_block_balance: u64,
    pub top_block_distribution_address: Pubkey,
    pub top_block_distribution_nonce: u8,
    pub top_block_last_account_address: Option<Pubkey>,
    pub top_block_last_account_rest_bp: u64,

    pub bottom_block_number: u64,
    pub bottom_block_available_bp: u64,
    pub bottom_block_solution_timestamp: i64,
    pub bottom_block_balance: u64,
    pub bottom_block_distribution_address: Pubkey,
    pub bottom_block_distribution_nonce: u8,
    pub bottom_block_last_account_address: Option<Pubkey>,
    pub bottom_block_last_account_rest_bp: u64,

    pub final_staking_account_nonce: u8,
    pub final_staking_pool_in_round: u64,
    pub final_staking_last_staking_timestamp: i64,
    pub final_staking_left_reward_parts_in_round: f64,
    pub final_staking_left_balance_in_round: u64,

    pub final_mining_account_nonce: u8,
}
