# Introduction

We express our gratitude to the C8ntinuum team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

Continuum is a multichain-powered protocol that thrives on the limitless possibilities created by connecting leading blockchain networks.

# Review Scope

Repository:	https://github.com/c8ntinuum/contracts-sol
Commit:	7ff01bf38219d3e40817d355855e99de2b2e7232

# Audit Findings

## F-2025-9438 Incorrect Pool Deserialization Causing Program Failure

Severity: Critical

Description
The program attempts to use the USDT-WSOL pool from Raydium Concentrated Liquidity, but it is being deserialized as if it were a Raydium CPMM (Constant Product Market Maker) pool. This mismatch leads to size mismatches in deserialization, causing a panic with the error message:

Program log: panicked at src/internal.rs:32:3: from_bytes>SizeMismatch

The issue makes the program unusable.

    // Load pool WSOL/C8NT state to calculate prices - WSOL is always token 0 because is the smallest key ever
    let pool_state_data: Vec<u8> = ctx.accounts.pool_state.try_borrow_data()?.to_vec();
    let pool_state_bytes: &[u8] = &pool_state_data[8..];
    let pool_state: &PoolState = bytemuck::from_bytes(pool_state_bytes);
    let (price_1_wsol_in_c8nt, price_1_c8nt_in_wsol) = utils::token_price(
        ctx.accounts.pool_wsol_vault.amount,
        pool_state.protocol_fees_token_0,
        pool_state.fund_fees_token_0,
        ctx.accounts.pool_c8nt_vault.amount,
        pool_state.protocol_fees_token_1,
        pool_state.fund_fees_token_1,
    );

    // Load pool WSOL/USDT state to calculate prices - WSOL is always token 0 because is the smallest key ever
    let pool2_state_data: Vec<u8> = ctx.accounts.pool_usdt_wsol.try_borrow_data()?.to_vec();
    let pool2_state_bytes: &[u8] = &pool2_state_data[8..];
    let pool2_state: &PoolState2 = bytemuck::from_bytes(pool2_state_bytes);
    let (price_1_wsol_in_usdt, _) = utils::token_price(
        ctx.accounts.pool_usdt_wsol_token_wsol_vault.amount,
        pool2_state.protocol_fees_token_0,
        pool2_state.fund_fees_token_0,
        ctx.accounts.pool_usdt_wsol_token_usdt_vault.amount,
        pool2_state.protocol_fees_token_1,
        pool2_state.fund_fees_token_1,
    );

## F-2025-9391 Missing Nonce Allows Repeated Executions of the Same Signed Generation Request


Severity: Low

Description
The generate_handler verifies a signed message from an off-chain verifier, but the message does not include a nonce or unique identifier. This allows the same signed payload to be reused multiple times until the deadline is reached, effectively enabling repeated executions of the same generation request.

However, the intended behavior is not clearly specified in the documentation — it's unclear whether each signed message is meant to authorize a single-use action or be valid for multiple uses during the valid window. As such, the security and financial impact of this behavior cannot be fully determined without explicit clarification from the protocol's design documentation.

    let message = [
        &ctx.program_id.as_ref(),              //
        C8NT_MINT.as_ref(),                    //
        &ctnm_usd_price_oracle.to_le_bytes(),  //
        &deadline.to_le_bytes(),               //
        ctx.accounts.referral1.key().as_ref(), //
        ctx.accounts.referral2.key().as_ref(), //
        ctx.accounts.signer.key().as_ref(),    //
    ]
    .concat();

## F-2025-9390 WSOL Price Derived Solely from Raydium Pool May Be Manipulable

Severity: Low

Description
The protocol currently derives the WSOL/USDT price from a Raydium pool to calculate the SOL value in USD. While WSOL is generally a highly liquid asset, relying solely on a DEX pool introduces potential vulnerabilities to short-term price manipulation, especially during periods of low activity, sandwich attacks, or MEV. Since this price feeds into off-chain processes, a manipulated value could have downstream financial consequences or be used to exploit the system.

    let (price_1_wsol_in_c8nt, price_1_c8nt_in_wsol) = utils::token_price(
        ctx.accounts.pool_wsol_vault.amount,
        pool_state.protocol_fees_token_0,
        pool_state.fund_fees_token_0,
        ctx.accounts.pool_c8nt_vault.amount,
        pool_state.protocol_fees_token_1,
        pool_state.fund_fees_token_1,
    );