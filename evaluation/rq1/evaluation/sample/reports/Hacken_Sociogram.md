# Introduction

We express our gratitude to the Sociogram team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

Sociogram is a token launch protocol on Solana that implements a bonding curve mechanism for predictable token distribution and price discovery. It provides a fully automated market maker (AMM) with virtual reserves for price calculation and real reserves for actual assets, allowing projects to launch tokens with guaranteed liquidity and controlled price movement. The protocol features a complete token lifecycle management system, from initial token creation and distribution through a bonding curve to final withdrawal phase when all tokens are sold.

# Review Scope

Repository	https://github.com/biswap-org/curve-launchpad
Commit	05f59a60823a6b60ce706ee9dffd94e2837ca1a2

# Audit Findings

## F-2025-8332 Lack of validation for set_fee edge values

Severity: Low

Description
The program does not validate the fee_amount value set during the set_fee call This allows authority to supply an arbitrary high values, what might discourage usage of the protocol due to the too big fee for the potential users. Additionally, authority can potentially front-run user operations, collecting high fees.

pub fn set_fee(
    ctx: Context<SetFee>,
    fee_amount: u64
) -> Result<()> {
    let global = &mut ctx.accounts.global;

    //confirm program is initialized
    require!(
        global.initialized,
        CurveLaunchpadError::NotInitialized
    );

    //confirm user is the authority
    require!(
        global.authority == *ctx.accounts.user.to_account_info().key,
        CurveLaunchpadError::InvalidAuthority
    );

    global.fee_basis_points = fee_amount;

    Ok(())
}

## F-2025-8331 Missing functionality to pause program operations

Severity: Low

Description
The program lacks a mechanism to pause its operations. This is considered a bad practice as it limits the authority's ability to respond to critical situations, such as:

Security Incidents - in the event of a vulnerability or exploit, the inability to pause operations could lead to significant financial losses.

Operational Failures - Pausing functionality allows the authority to temporarily halt operations while addressing bugs, misconfigurations, or other unforeseen issues.

## F-2025-8330 Lack of functionality for admin rotation

Severity: Low

Description
The program does not provide a mechanism to rotate or update the authority. This is considered a deviation from leading security practices as it limits flexibility and poses operational risks, such as the inability to recover from admin key compromise.

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    msg!("Calling initialize");
    let global = &mut ctx.accounts.global;

    require!(
        !global.initialized,
        CurveLaunchpadError::AlreadyInitialized,
    );

    global.authority = *ctx.accounts.authority.to_account_info().key;