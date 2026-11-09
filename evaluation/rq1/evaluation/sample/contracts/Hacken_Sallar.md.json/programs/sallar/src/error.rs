use anchor_lang::prelude::error_code;

/// The enum defining all errors used by the contract.
#[error_code]
pub enum SallarError {
    #[msg("You are not an owner")]
    Unauthorized = 0,
    #[msg("Required time interval between solved blocks not passed (3 minutes).")]
    BlockSolutionAheadOfTime = 1,
    #[msg("Required time interval between final staking not passed (3 minutes).")]
    FinalStakingAheadOfTime = 2,
    #[msg("Block already solved")]
    BlockAlreadySolved = 3,
    #[msg("Top block not solved yet")]
    TopBlockNotSolvedYet = 4,
    #[msg("Bottom block not solved yet")]
    BottomBlockNotSolvedYet = 5,
    #[msg("Initial token distribution already performed")]
    InitialTokenDistributionAlreadyPerformed = 6,
    #[msg("Blocks not collided yet")]
    BlocksNotCollidedYet = 7,
    #[msg("Final staking pool in round is empty")]
    FinalStakingPoolInRoundIsEmpty = 8,
    #[msg("Missing user info")]
    MissingUserInfo = 9,
    #[msg("User request received for solved block")]
    UserRequestForSolvedBlock = 10,
    #[msg("Last account did not receive all BPs but the current block is not a new one")]
    UserRestExistsButBlockIsNotNew = 11,
    #[msg("Last account did not receive all BPs but the first user info for a new block is not this account")]
    UserRestExistsButFirstRequestForNewBlockIsNotForThisAccount = 12,
    #[msg("Last account did not receive all BPs but the first call to solve block for a new block does not contain this account")]
    UserRestExistsButFirstRequestForNewBlockMissedTheAccount = 13,
    #[msg("User request exceeds available reward parts")]
    UserRequestExceedsAvailableRewardParts = 14,
    #[msg("Account from remaining accounts not found in user info")]
    MismatchBetweenRemainingAccountsAndUserInfo = 15,
    #[msg("Sum of user reward parts exceeds 1")]
    UserRewardPartsSumTooHigh = 16,
    #[msg("Lack of funds to pay the reward")]
    LackOfFundsToPayTheReward = 17,
    #[msg("Mismatch between available block BP and balance")]
    MismatchBetweenAvailableBlockBPAndBalance = 18,
    #[msg("F64 conversion error occurred")]
    F64ConversionError = 19,
    #[msg("U64 conversion error occurred")]
    U64ConversionError = 20,
    #[msg("Illegal execution of set_blocks_collided function outside tests")]
    ExecutionOfSetBlocksCollidedFunctionOutsideTests = 21,
}
