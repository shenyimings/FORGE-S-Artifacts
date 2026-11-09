# 

# Pump Science  Findings & Analysis Report



#### 2025-02-26



## Table of Contents

- 
[Overview](#overview)

- [About C4](#about-c4)

- [Summary](#summary)
- [Scope](#scope)
- [Severity Criteria](#severity-criteria)
- 
[High Risk Findings (2)](#high-risk-findings-2)

- [[H-01] The lock_pool operation can be DoS](#h-01-the-lock_pool-operation-can-be-dos)
- [[H-02] Missing Update of migration_token_allocation on Global Struct](#h-02-missing-update-of-migration_token_allocation-on-global-struct)

- 
[Medium Risk Findings (3)](#medium-risk-findings-3)

- [[M-01] Last buy might charge the wrong fee](#m-01-last-buy-might-charge-the-wrong-fee)
- [[M-02] Bonding Curve Invariant Check Incorrectly Validates SOL Balance Due to Rent Inclusion](#m-02-bonding-curve-invariant-check-incorrectly-validates-sol-balance-due-to-rent-inclusion)
- [[M-03] Abrupt fee transition from 8.76% to 1% at slot 250 due to incorrect linear decrease formula](#m-03-abrupt-fee-transition-from-876-to-1-at-slot-250-due-to-incorrect-linear-decrease-formula)

- 
[Low Risk and Non-Critical Issues](#low-risk-and-non-critical-issues)

- [](#)
- [Pump Science  Findings \& Analysis Report](#pump-science--findings--analysis-report)
      - [2025-02-26](#2025-02-26)
  - [Table of Contents](#table-of-contents)
- [Overview](#overview)
  - [About C4](#about-c4)
- [Summary](#summary)
- [Scope](#scope)
- [Severity Criteria](#severity-criteria)
- [High Risk Findings (2)](#high-risk-findings-2)
  - [\[H-01\] The lock\_pool operation can be DoS](#h-01-the-lock_pool-operation-can-be-dos)
    - [Proof of Concept](#proof-of-concept)
    - [Recommended mitigation steps](#recommended-mitigation-steps)
  - [\[H-02\] Missing Update of migration\_token\_allocation on Global Struct](#h-02-missing-update-of-migration_token_allocation-on-global-struct)
    - [Proof of Concept](#proof-of-concept-1)
    - [Recommended mitigation steps](#recommended-mitigation-steps-1)
- [Medium Risk Findings (3)](#medium-risk-findings-3)
  - [\[M-01\] Last buy might charge the wrong fee](#m-01-last-buy-might-charge-the-wrong-fee)
    - [Proof of Concept](#proof-of-concept-2)
    - [Recommended mitigation steps](#recommended-mitigation-steps-2)
  - [\[M-02\] Bonding Curve Invariant Check Incorrectly Validates SOL Balance Due to Rent Inclusion](#m-02-bonding-curve-invariant-check-incorrectly-validates-sol-balance-due-to-rent-inclusion)
    - [Proof of Concept](#proof-of-concept-3)
    - [Recommended Mitigation Steps](#recommended-mitigation-steps-3)
  - [\[M-03\] Abrupt fee transition from 8.76% to 1% at slot 250 due to incorrect linear decrease formula](#m-03-abrupt-fee-transition-from-876-to-1-at-slot-250-due-to-incorrect-linear-decrease-formula)
    - [Proof of Concept](#proof-of-concept-4)
    - [Proof of Concept](#proof-of-concept-5)
    - [Recommended Mitigation Steps](#recommended-mitigation-steps-4)
- [Low Risk and Non-Critical Issues](#low-risk-and-non-critical-issues)
  - [\[01\] Incorrect Fund Distribution During Migration Due to Unvalidated Escrow Balance Transfer](#01-incorrect-fund-distribution-during-migration-due-to-unvalidated-escrow-balance-transfer)
    - [Impact](#impact)
    - [Recommendation](#recommendation)
  - [\[02\] Fee Calculation in Phase 2 May Lead to Rounding Errors Due to Integer Division](#02-fee-calculation-in-phase-2-may-lead-to-rounding-errors-due-to-integer-division)
    - [Impact](#impact-1)
    - [Recommendation](#recommendation-1)
  - [\[03\] Lack of Maximum Input Amount Validation in Swap Function Could Lead to Unnecessary Transaction Failures](#03-lack-of-maximum-input-amount-validation-in-swap-function-could-lead-to-unnecessary-transaction-failures)
    - [Impact](#impact-2)
    - [Recommendation](#recommendation-2)
  - [\[04\] Bonding Curve Creation Lacks URI Validation in Metadata Leading to Potential Malformed Token Information](#04-bonding-curve-creation-lacks-uri-validation-in-metadata-leading-to-potential-malformed-token-information)
    - [Impact](#impact-3)
    - [Recommendation](#recommendation-3)
  - [\[05\] Hardcoded Gas Fee in Pool Migration Could Lead to Failed Transactions in Network Congestion](#05-hardcoded-gas-fee-in-pool-migration-could-lead-to-failed-transactions-in-network-congestion)
    - [Impact](#impact-4)
    - [Recommendation](#recommendation-4)
  - [\[06\] Basis Points Multiplication Function Lacks Input Validation Leading to Silent Failures](#06-basis-points-multiplication-function-lacks-input-validation-leading-to-silent-failures)
    - [Impact](#impact-5)
    - [Recommendation](#recommendation-5)
  - [\[07\] Trade Event Emission Uses Redundant Clock Calls and Lacks Block Time Validation](#07-trade-event-emission-uses-redundant-clock-calls-and-lacks-block-time-validation)
    - [Impact](#impact-6)
    - [Recommendation](#recommendation-6)
  - [\[08\] Bonding Curve Token Metadata Fields Lack Length Restrictions Leading to Excessive Storage Costs](#08-bonding-curve-token-metadata-fields-lack-length-restrictions-leading-to-excessive-storage-costs)
    - [Impact](#impact-7)
    - [Recommendation](#recommendation-7)
  - [\[09\] Bonding Curve Token Account Lock/Unlock Operations Lack Event Emission for Critical State Changes](#09-bonding-curve-token-account-lockunlock-operations-lack-event-emission-for-critical-state-changes)
    - [Impact](#impact-8)
    - [Recommendation](#recommendation-8)
  - [\[10\] Empty remove\_wl Function Implementation Creates Misleading Security Expectation](#10-empty-remove_wl-function-implementation-creates-misleading-security-expectation)
    - [Impact](#impact-9)
    - [Recommendation](#recommendation-9)
  - [\[11\] Hardcoded Program IDs in Constants Create Deployment Inflexibility and Testing Challenges](#11-hardcoded-program-ids-in-constants-create-deployment-inflexibility-and-testing-challenges)
    - [Impact](#impact-10)
    - [Recommendation](#recommendation-10)
  - [\[12\] Lock Pool Instruction Lacks Escrow Account Address Validation Leading to Potential Fund Lock](#12-lock-pool-instruction-lacks-escrow-account-address-validation-leading-to-potential-fund-lock)
    - [Impact](#impact-11)
    - [Recommendation](#recommendation-11)
- [Disclosures](#disclosures)


# Overview



## About C4


Code4rena (C4) is an open organization consisting of security researchers, auditors, developers, and individuals with domain expertise in smart contracts.

A C4 audit is an event in which community participants, referred to as Wardens, review, audit, or analyze smart contract logic in exchange for a bounty provided by sponsoring projects.

During the audit outlined in this document, C4 conducted an analysis of the Pump Science smart contract system. The audit took place from January 15 to January 23, 2025.

This audit was judged by [Koolex](https://code4rena.com/@Koolex).

Final report assembled by Code4rena.


# Summary


The C4 analysis yielded an aggregated total of 5 unique vulnerabilities. Of these vulnerabilities, 2 received a risk rating in the category of HIGH severity and 3 received a risk rating in the category of MEDIUM severity.

Additionally, C4 analysis included 8 reports detailing issues with a risk rating of LOW severity or non-critical. 

All of the issues presented here are linked back to their original finding.


# Scope


The code under review can be found within the [C4 Pump Science repository](https://github.com/code-423n4/2025-01-pump-science), and is composed of 25 smart contracts written in the Rust programming language and includes 2,030 lines of Rust code.


# Severity Criteria


C4 assesses the severity of disclosed vulnerabilities based on three primary risk categories: high, medium, and low/non-critical.

High-level considerations for vulnerabilities span the following key areas when conducting assessments:

- Malicious Input Handling
- Escalation of privileges
- Arithmetic
- Gas use

For more information regarding the severity criteria referenced throughout the submission review process, please refer to the documentation provided on [the C4 website](https://code4rena.com), specifically our section on [Severity Categorization](https://docs.code4rena.com/awarding/judging-criteria/severity-categorization).


# High Risk Findings (2)



## [H-01] The lock_pool operation can be DoS


*Submitted by shaflow2, also found by Spearmint*

[https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/instructions/migration/lock_pool.rs#L149](https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/instructions/migration/lock_pool.rs#L149)

The `lock_pool` operation requires the creation of a `lockEscrow` account. However, a malicious actor could preemptively create the `lockEscrow` account, causing the `create_lock_escrow` transaction to fail and resulting in a Denial of Service (DoS) for the `lock_pool` operation.


### Proof of Concept


During the `lock_pool` process, the `create_lock_escrow` function is called to create the `lock_escrow` account.

```

    // Create Lock Escrow
    let escrow_accounts = vec![
        AccountMeta::new(ctx.accounts.pool.key(), false),
        AccountMeta::new(ctx.accounts.lock_escrow.key(), false),
        AccountMeta::new_readonly(ctx.accounts.fee_receiver.key(), false),
        AccountMeta::new_readonly(ctx.accounts.lp_mint.key(), false),
        AccountMeta::new(ctx.accounts.bonding_curve_sol_escrow.key(), true), // Bonding Curve Sol Escrow is the payer/signer
        AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
    ];

    let escrow_instruction = Instruction {
        program_id: meteora_program_id,
        accounts: escrow_accounts,
        data: get_function_hash("global", "create_lock_escrow").into(),
    };

    invoke_signed(
        &escrow_instruction,
        &[
            ctx.accounts.pool.to_account_info(),
            ctx.accounts.lock_escrow.to_account_info(),
            ctx.accounts.fee_receiver.to_account_info(),
            ctx.accounts.lp_mint.to_account_info(),
            ctx.accounts.bonding_curve_sol_escrow.to_account_info(), // Bonding Curve Sol Escrow is the payer/signer
            ctx.accounts.system_program.to_account_info(),
        ],
        bonding_curve_sol_escrow_signer_seeds,
    )?;

```

However, the `lock_escrow` account is derived using the `pool` and `owner` as seeds, and its creation does not require the owner’s signature. This means that a malicious actor could preemptively create the `lock_escrow` account to perform a DoS attack on the `lock_pool` operation.

```

/// Accounts for create lock account instruction
#[derive(Accounts)]
pub struct CreateLockEscrow<'info> {
    /// CHECK:
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Lock account
    #[account(
        init,
        seeds = [
            "lock_escrow".as_ref(),
            pool.key().as_ref(),
            owner.key().as_ref(),
        ],
        space = 8 + std::mem::size_of::<LockEscrow>(),
        bump,
        payer = payer,
    )]
    pub lock_escrow: UncheckedAccount<'info>,

    /// CHECK: Owner account
@>  pub owner: UncheckedAccount<'info>,

    /// CHECK: LP token mint of the pool
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Payer account
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: System program.
    pub system_program: UncheckedAccount<'info>,
}

```

### Recommended mitigation steps


In the `lock_pool` process, check if the `lock_escrow` exists. If it exists, skip the creation process.

**Kulture (Pump Science) confirmed**


## [H-02] Missing Update of migration_token_allocation on Global Struct


*Submitted by D1r3Wolf, also found by 0x_kmr_ and Spearmint*

[https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/state/global.rs#L119-L132](https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/state/global.rs#L119-L132)

[https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/state/global.rs#L66](https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/state/global.rs#L66)

During the audit, it was identified that the `migration_token_allocation` variable on the `Global` struct is not updated in the `Global::update_settings` function. This creates a critical issue as the `migration_token_allocation` value, which is used during the migration process in the `create_pool` instruction, will remain uninitialized or stuck at its default value indefinitely.  

The `update_settings` function is executed within the `set_params` instruction, making it a central mechanism for modifying key global settings. However, due to the missing update logic for `migration_token_allocation`, any updates intended for this variable via `GlobalSettingsInput` are ignored. As a result, the `migration_token_allocation` on the `Global` struct is never updated, leading to a persistent and incorrect value that could disrupt the migration process.


### Proof of Concept


Notes: Execute the test case in src/state/bonding_curve/tests.rs

```

    #[test]
    fn test_global_update_settings() {
        use crate::GlobalSettingsInput;
        let mut global = Global::default();

        let new_mint_decimals = 8;
        let new_migration_token_allocation = 123_000_000;
        let mut params = GlobalSettingsInput {
            initial_virtual_token_reserves: 0,
            initial_virtual_sol_reserves: 0,
            initial_real_token_reserves: 0,
            token_total_supply: 0,
            mint_decimals: new_mint_decimals,
            migrate_fee_amount: 0,
            migration_token_allocation: new_migration_token_allocation,
            fee_receiver: Pubkey::default(),
            whitelist_enabled: false,
            meteora_config: Pubkey::default(),
        };

        global.update_settings(params, 0);

        assert_eq!(global.mint_decimals, new_mint_decimals); // Passes
        assert_eq!(global.migration_token_allocation, new_migration_token_allocation); // Fails as the variables not updated
    }

```

**Impact:** 

The `migration_token_allocation` will retain its default value indefinitely, regardless of any intended updates.


### Recommended mitigation steps


To resolve this issue, we recommend the following steps:  

1. 
Implement the logic to update `migration_token_allocation` in the `Global::update_settings` function.  

- This should retrieve the value from the `GlobalSettingsInput` parameter provided to the `update_settings` function.  

2. 
Test and Validate the Fix:  

- Ensure unit tests are added to confirm the successful update of Global struct with the values from the `GlobalSettingsInput`.

**Kulture (Pump Science) confirmed**


# Medium Risk Findings (3)



## [M-01] Last buy might charge the wrong fee


*Submitted by shaflow2, also found by 0xlookman, 13u9, Agontuk, ATH, GEEKS, and p0wd3r*

[https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/state/bonding_curve/curve.rs#L103](https://github.com/code-423n4/2025-01-pump-science/blob/768ef58478724bf6b464c9f0952e3e5a3b2a2613/programs/pump-science/src/state/bonding_curve/curve.rs#L103)

In the “last buy” process, the protocol automatically adjusts the price to fit the curve, ensuring precise SOL fundraising. This results in a change in the transaction price, which also alters the amount of SOL paid by the user. However, since the swap fee is calculated beforehand, this can lead to an incorrect fee calculation. And if the actual amount of SOL to be paid increases, we should recheck that `ctx.accounts.user.get_lamports() >= exact_in_amount.checked_add(min_rent).unwrap()` to ensure sufficient remaining funds and prevent the account from being closed.


### Proof of Concept


The transaction fee is calculated before entering the `apply_buy` function, based on the `exact_in_amount` provided as input.

```

            // Check if slot is start slot and buyer is bonding_curve creator
            if clock.slot == bonding_curve.start_slot
                && ctx.accounts.user.key() == bonding_curve.creator
            {
                msg!("Dev buy");
                fee_lamports = 0;
                buy_amount_applied = exact_in_amount;
            } else {
                fee_lamports = bonding_curve.calculate_fee(exact_in_amount, clock.slot)?;
                msg!("Fee: {} SOL", fee_lamports);
                buy_amount_applied = exact_in_amount - fee_lamports;
            }

            let buy_result = ctx
                .accounts
                .bonding_curve
                .apply_buy(buy_amount_applied)
                .ok_or(ContractError::BuyFailed)?;

```

However, if it is the last buy, the price adjustment will cause the actual amount of SOL the user needs to pay (as reflected in `buy_result`) to change, leading to incorrect protocol fee calculation.

```

        if token_amount >= self.real_token_reserves {
            // Last Buy
            token_amount = self.real_token_reserves;

            // Temporarily store the current state
            let current_virtual_token_reserves = self.virtual_token_reserves;
            let current_virtual_sol_reserves = self.virtual_sol_reserves;

            // Update self with the new token amount
            self.virtual_token_reserves = (current_virtual_token_reserves as u128)
                .checked_sub(token_amount as u128)?
                .try_into()
                .ok()?;
            self.virtual_sol_reserves = 115_005_359_056; // Total raise amount at end

            let recomputed_sol_amount = self.get_sol_for_sell_tokens(token_amount)?;
            msg!("ApplyBuy: recomputed_sol_amount: {}", recomputed_sol_amount);
            sol_amount = recomputed_sol_amount;

            // Restore the state with the recomputed sol_amount
            self.virtual_token_reserves = current_virtual_token_reserves;
            self.virtual_sol_reserves = current_virtual_sol_reserves;

            // Set complete to true
            self.complete = true;
        }

```

### Recommended mitigation steps


After the `apply_buy` call is completed, check whether the actual SOL paid by the user in `buy_result` matches `buy_amount_applied`. If they do not match, recalculate `fee_lamports`. Additionally, revalidate that `ctx.accounts.user.get_lamports() >= exact_in_amount.checked_add(min_rent).unwrap()`.

It is also recommended to add a new slippage parameter to control the maximum SOL input.

**Kulture (Pump Science) confirmed**


## [M-02] Bonding Curve Invariant Check Incorrectly Validates SOL Balance Due to Rent Inclusion


*Submitted by Evo*

[https://github.com/code-423n4/2025-01-pump-science/blob/main/programs/pump-science/src/state/bonding_curve/curve.rs#L306](https://github.com/code-423n4/2025-01-pump-science/blob/main/programs/pump-science/src/state/bonding_curve/curve.rs#L306)

The bonding curve invariant check fails to account for rent when comparing SOL balances, leading to incorrect validation of the protocol’s core invariant. Since `sol_escrow_lamports` includes rent while `real_sol_reserves` doesn’t, the invariant check could pass when it should fail. 


### Proof of Concept


The issue exists in the bonding curve invariant check in [curve.rs:L306](https://github.com/code-423n4/2025-01-pump-science/blob/main/programs/pump-science/src/state/bonding_curve/curve.rs#L306):

```

// Get raw lamports which includes rent
let sol_escrow_lamports = sol_escrow.lamports();

// Ensure real sol reserves are equal to bonding curve pool lamports 
if sol_escrow_lamports < bonding_curve.real_sol_reserves {
    msg!(
        "real_sol_r:{}, bonding_lamps:{}",
        bonding_curve.real_sol_reserves,
        sol_escrow_lamports
    );
    msg!("Invariant failed: real_sol_reserves != bonding_curve_pool_lamports");
    return Err(ContractError::BondingCurveInvariant.into());
}

```

The issue arises because:

1. `sol_escrow_lamports` is retrieved using `lamports()` which returns the total balance including rent
2. This is compared directly against `real_sol_reserves` which tracks only the actual SOL reserves without rent
3. The comparison `sol_escrow_lamports < bonding_curve.real_sol_reserves` will incorrectly pass when `sol_escrow_lamports` has insufficient SOL (excluding rent) but the rent amount makes up the difference

For example:

- If `real_sol_reserves` = 100 SOL (100,000,000,000 lamports)
- And actual available SOL = 99.99795072 SOL (99,997,960,720 lamports)
- And rent = 0.00204928 SOL (2,039,280 lamports)
- Then `sol_escrow_lamports` = 100 SOL (100,000,000,000 lamports)
- The check 100 < 100 is false, so the invariant passes
- But it should fail since the actual available SOL (99.99795072) is less than required (100)

Evidence of the original intent to handle rent can be seen in the commented out code:

```

// let rent_exemption_balance: u64 =
//     Rent::get()?.minimum_balance(8 + BondingCurve::INIT_SPACE as usize);
// let bonding_curve_pool_lamports: u64 = lamports - rent_exemption_balance;

```

Which will cause the issue.


### Recommended Mitigation Steps


Subtract the rent-exemption amount from `sol_escrow_lamports` before comparing to `real_sol_reserves` in the invariant check.

**Kulture (Pump Science) confirmed**


## [M-03] Abrupt fee transition from 8.76% to 1% at slot 250 due to incorrect linear decrease formula


*Submitted by Evo, also found by 0xcb90f054, Albort, Arjuna, debo, ETHworker, ETHworker, ETHworker, and Spearmint*

[https://github.com/code-423n4/2025-01-pump-science/blob/main/programs/pump-science/src/state/bonding_curve/curve.rs#L31-L41](https://github.com/code-423n4/2025-01-pump-science/blob/main/programs/pump-science/src/state/bonding_curve/curve.rs#L31-L41)

Fee transition creates a significant 7.76% economic discontinuity at slot 250-251 boundary, causing incorrect fees implementation as protocol intended.


### Proof of Concept


The issue occurs in the fee calculation logic in [curve.rs](https://github.com/code-423n4/2025-01-pump-science/blob/main/programs/pump-science/src/state/bonding_curve/curve.rs#L31-L41):

```

pub fn calculate_fee(&self, amount: u64, current_slot: u64) -> Result<u64> {
	///code
	
    if slots_passed < 150 {
        // Phase 1: 99% fees
        sol_fee = bps_mul(9900, amount, 10_000).unwrap();
    } else if slots_passed >= 150 && slots_passed <= 250 {
        // Phase 2: Linear decrease - Issue occurs here
        let fee_bps = (-8_300_000_i64)
            .checked_mul(slots_passed as i64)
            .ok_or(ContractError::ArithmeticError)?
            .checked_add(2_162_600_000)
            .ok_or(ContractError::ArithmeticError)?
            .checked_div(100_000)
            .ok_or(ContractError::ArithmeticError)?;
        sol_fee = bps_mul(fee_bps as u64, amount, 10_000).unwrap();
    } else if slots_passed > 250 {
        // Phase 3: 1% fees
        sol_fee = bps_mul(100, amount, 10_000).unwrap();
    }

```

The linear decrease formula during Phase 2 (slots 150-250) creates an incorrect transition:

- 
Phase 2 last slot (250):

- Formula: (-8*300*000 * 250 + 2*162*600*000) / 100*000
- Result: 876 basis points = 8.76% fee

- 
Phase 3 first slot (251):

- Fixed 100 basis points = 1% fee
- Sudden 7.76% drop from previous slot

Key fee percentages showing the discontinuity:

- Slot 248: 10.42%
- Slot 249: 9.59%
- Slot 250: 8.76%
- Slot 251: 1.00% (abrupt drop)
- Slot 252: 1.00%

The linear decrease formula coefficients (-8*300*000 and 2*162*600_000) were not properly calibrated to reach 1% at slot 250, causing this economic discontinuity at the phase transition.


### Proof of Concept


Typescript simulation:

```

interface FeeCalculation {
 slot: number;
 feeBps: number;
 feePercentage: number;
 phase: string;
 details?: string;
}

function calculateFee(slot: number): FeeCalculation {
 let feeBps: number;
 let phase: string;
 let details: string = '';

 if (slot < 150) {
   feeBps = 9900;
   phase = "Phase 1: Fixed 99%";
 } else if (slot >= 150 && slot <= 250) {
   const multiplier = -8300000;
   const additive = 2162600000;
   
   const step1 = multiplier * slot;
   const step2 = step1 + additive;
   feeBps = Math.floor(step2 / 100000);
   
   phase = "Phase 2: Linear Decrease";
   details = `
     Step 1 (multiply): ${multiplier} * ${slot} = ${step1}
     Step 2 (add): ${step1} + ${additive} = ${step2}
     Step 3 (divide): ${step2} / 100000 = ${feeBps}
   `;
 } else {
   feeBps = 100;
   phase = "Phase 3: Fixed 1%";
 }

 return {
   slot,
   feeBps,
   feePercentage: feeBps / 100,
   phase,
   details
 };
}

function printAllFees(): void {
 for (let slot = 0; slot <= 252; slot++) {
   const result = calculateFee(slot);
   console.log(`Slot ${slot.toString().padStart(3, ' ')}: ${result.feePercentage.toFixed(2)}% - ${result.phase}`);
   if (result.details) {
     console.log(result.details);
   }
   console.log('-'.repeat(50));
 }
}

// Call function to print all fees
printAllFees();

// Test specific slots
const testSlots = [149, 150, 200, 250, 251];
testSlots.forEach(slot => {
 const result = calculateFee(slot);
 console.log(`\nDetailed analysis for slot ${slot}:`);
 console.log(JSON.stringify(result, null, 2));
});

```

OUTPUT:

```

LOG]: "Slot 250: 8.76% - Phase 2: Linear Decrease" 
[LOG]: "
     Step 1 (multiply): -8300000 * 250 = -2075000000
     Step 2 (add): -2075000000 + 2162600000 = 87600000
     Step 3 (divide): 87600000 / 100000 = 876
   " 
[LOG]: "--------------------------------------------------" 
[LOG]: "Slot 251: 1.00% - Phase 3: Fixed 1%" 
[LOG]: "--------------------------------------------------" 
[LOG]: "Slot 252: 1.00% - Phase 3: Fixed 1%" 
[LOG]: "--------------------------------------------------" 
[LOG]: "
Detailed analysis for slot 149:" 
[LOG]: "{
  "slot": 149,
  "feeBps": 9900,
  "feePercentage": 99,
  "phase": "Phase 1: Fixed 99%",
  "details": ""
}" 
[LOG]: "
Detailed analysis for slot 150:" 
[LOG]: "{
  "slot": 150,
  "feeBps": 9176,
  "feePercentage": 91.76,
  "phase": "Phase 2: Linear Decrease",
  "details": "\n     Step 1 (multiply): -8300000 * 150 = -1245000000\n     Step 2 (add): -1245000000 + 2162600000 = 917600000\n     Step 3 (divide): 917600000 / 100000 = 9176\n   "
}" 
[LOG]: "
Detailed analysis for slot 200:" 
[LOG]: "{
  "slot": 200,
  "feeBps": 5026,
  "feePercentage": 50.26,
  "phase": "Phase 2: Linear Decrease",
  "details": "\n     Step 1 (multiply): -8300000 * 200 = -1660000000\n     Step 2 (add): -1660000000 + 2162600000 = 502600000\n     Step 3 (divide): 502600000 / 100000 = 5026\n   "
}" 
[LOG]: "
Detailed analysis for slot 250:" 
[LOG]: "{
  "slot": 250,
  "feeBps": 876,
  "feePercentage": 8.76,
  "phase": "Phase 2: Linear Decrease",
  "details": "\n     Step 1 (multiply): -8300000 * 250 = -2075000000\n     Step 2 (add): -2075000000 + 2162600000 = 87600000\n     Step 3 (divide): 87600000 / 100000 = 876\n   "
}" 
[LOG]: "
Detailed analysis for slot 251:" 
[LOG]: "{
  "slot": 251,
  "feeBps": 100,
  "feePercentage": 1,
  "phase": "Phase 3: Fixed 1%",
  "details": ""
}"

```

### Recommended Mitigation Steps


Recalibrate the linear decrease formula coefficients to ensure the fee percentage reaches exactly 1% at slot 250, maintaining a smooth transition between Phase 2 and Phase 3.

**Kulture (Pump Science) acknowledged**


# Low Risk and Non-Critical Issues


For this audit, 8 reports were submitted by wardens detailing low risk and non-critical issues. The [report highlighted below](https://code4rena.com/audits/2025-01-pump-science/submissions/S-162) by **Agontuk** received the top score from the judge.

*The following wardens also submitted reports: 0xcb90f054, ATH, chinepun, DoD4uFN, ElectronicCricket91, KupiaSec, and Sparrow.*


## [01] Incorrect Fund Distribution During Migration Due to Unvalidated Escrow Balance Transfer


The Pump Science protocol’s migration process involves a two-step operation where bonding curve assets are transferred to a Meteora pool. During this process, the `initialize_pool_with_config()` function first calculates and allocates the required SOL amount for the new pool, while `lock_pool()` completes the migration and handles remaining funds.

The vulnerability exists in the `lock_pool()` function’s handling of remaining escrow funds:

```

// In lock_pool()
let bonding_curve_remaining_lamports = ctx.accounts.bonding_curve_sol_escrow.get_lamports();
let sol_ix = system_instruction::transfer(
    &ctx.accounts.bonding_curve_sol_escrow.to_account_info().key,
    &ctx.accounts.fee_receiver.to_account_info().key,
    bonding_curve_remaining_lamports,
);

```

The function blindly transfers all remaining lamports to the fee receiver without validating against the expected `migrate_fee_amount`. Since the escrow is a PDA that can receive SOL between the two migration steps, any excess funds (from failed transactions, rounding errors, or external transfers) will be incorrectly sent to the fee receiver.

For example:

1. Initial state: `real_sol_reserves` = 1000 SOL, `migrate_fee_amount` = 0.5 SOL
2. Pool initialized with 999.46 SOL (1000 - 0.5 - 0.04)
3. 5 SOL sent to escrow between transactions
4. `lock_pool()` transfers entire 5.5 SOL to fee receiver instead of just 0.5 SOL fee


### Impact


Excess funds in the escrow during migration are incorrectly transferred to the fee receiver instead of being properly allocated.


### Recommendation


Modify `lock_pool()` to validate the remaining balance against `migrate_fee_amount`:

```

let remaining_lamports = ctx.accounts.bonding_curve_sol_escrow.get_lamports();
require!(
    remaining_lamports == ctx.accounts.global.migrate_fee_amount,
    ContractError::InvalidRemainingBalance
);

```

## [02] Fee Calculation in Phase 2 May Lead to Rounding Errors Due to Integer Division


The Pump Science protocol implements a dynamic fee structure with three phases, where Phase 2 (slots 150-250) uses a linear decrease formula. The fee calculation is implemented in the `calculate_fee()` function:

```

let fee_bps = (-8_300_000_i64)
    .checked_mul(slots_passed as i64)
    .ok_or(ContractError::ArithmeticError)?
    .checked_add(2_162_600_000)
    .ok_or(ContractError::ArithmeticError)?
    .checked_div(100_000)
    .ok_or(ContractError::ArithmeticError)?;

```

The issue lies in the integer division by 100_000 which happens before the fee calculation. This order of operations can lead to precision loss due to integer division rounding. For example:

1. 
At slot 200:

- (-8*300*000 * 200 + 2*162*600*000) / 100*000 = 4963
- Actual calculation should be: 49.63%
- But due to integer division, it becomes 49%

This means users could be charged slightly incorrect fees during Phase 2, though the impact is minimal due to the small rounding differences.


### Impact


Users may be charged slightly incorrect fees during Phase 2 due to precision loss in integer division.


### Recommendation


Consider reordering the operations to minimize precision loss:

```

let fee_bps = (-8_300_000_i64)
    .checked_mul(slots_passed as i64)
    .ok_or(ContractError::ArithmeticError)?
    .checked_add(2_162_600_000)
    .ok_or(ContractError::ArithmeticError)?;
let fee = amount
    .checked_mul((fee_bps as u64))
    .ok_or(ContractError::ArithmeticError)?
    .checked_div(1_000_000)
    .ok_or(ContractError::ArithmeticError)?;

```

## [03] Lack of Maximum Input Amount Validation in Swap Function Could Lead to Unnecessary Transaction Failures


The Pump Science protocol’s swap functionality, implemented in the `Swap` instruction, validates the minimum input amount but lacks validation for maximum input amounts. In the `validate()` function, we only see:

```

require!(exact_in_amount > &0, ContractError::MinSwap);

```

While there is a check for minimum input, there’s no upper bound validation. This could lead to unnecessary transaction failures in two scenarios:

1. 
For token purchases (base_in = false):

- If user inputs amount > bonding curve’s `real_sol_reserves`
- Transaction will fail later in `apply_buy()` but gas is already consumed

2. 
For token sales (base_in = true):

- If user inputs amount > their token balance
- Transaction will fail at token transfer but gas is already consumed

The issue is especially relevant because the bonding curve’s available liquidity changes over time, and users might not be aware of the current limits when submitting transactions.


### Impact


Users may experience unnecessary transaction failures and gas wastage when submitting swap transactions with amounts that exceed available liquidity or their balance.


### Recommendation


Add maximum amount validations in the `validate()` function:

```

pub fn validate(&self, params: &SwapParams) -> Result<()> {
    // ... existing validations ...
    
    if params.base_in {
        require!(
            params.exact_in_amount <= self.user_token_account.amount,
            ContractError::InsufficientBalance
        );
    } else {
        require!(
            params.exact_in_amount <= self.bonding_curve.real_sol_reserves,
            ContractError::InsufficientLiquidity
        );
    }
    
    Ok(())
}

```

## [04] Bonding Curve Creation Lacks URI Validation in Metadata Leading to Potential Malformed Token Information


The Pump Science protocol allows creators to create bonding curves with associated token metadata. The metadata creation is handled in the `initialize_meta()` function of the `CreateBondingCurve` instruction. 

However, the protocol does not validate the URI format or content in the metadata parameters:

```

pub fn intialize_meta(
    &mut self,
    mint_auth_signer_seeds: &[&[&[u8]]; 1],
    params: &CreateBondingCurveParams,
) -> Result<()> {
    let data_v2 = DataV2 {
        name: params.name.clone(),
        symbol: params.symbol.clone(),
        uri: params.uri.clone(),  // No validation on URI format or content
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };
    // ... metadata creation code ...
}

```

The issue is that the URI, which typically points to off-chain metadata (like JSON files containing token images and descriptions), is not validated for:

1. Basic URL format compliance
2. Maximum length restrictions
3. Allowed protocols (http/https/ipfs)
4. Character encoding

This could lead to:

1. Malformed metadata that breaks token explorers
2. URIs that are too long and waste on-chain storage
3. URIs pointing to invalid or malicious resources
4. Encoding issues causing display problems


### Impact


Token metadata may be malformed or contain invalid URIs, leading to poor user experience and potential display issues in token explorers or wallets.


### Recommendation


Add URI validation in the `validate()` function:

```

pub fn validate(&self, params: &CreateBondingCurveParams) -> Result<()> {
    // ... existing validations ...
    
    // Validate URI
    require!(
        params.uri.len() <= 200,  // Reasonable max length
        ContractError::InvalidMetadataUri
    );
    require!(
        params.uri.starts_with("http://") || 
        params.uri.starts_with("https://") || 
        params.uri.starts_with("ipfs://"),
        ContractError::InvalidMetadataUri
    );
    require!(
        params.uri.chars().all(|c| c.is_ascii()),
        ContractError::InvalidMetadataUri
    );
    
    Ok(())
}

```

## [05] Hardcoded Gas Fee in Pool Migration Could Lead to Failed Transactions in Network Congestion


The Pump Science protocol’s pool migration process includes a hardcoded gas fee deduction in the `initialize_pool_with_config()` function:

```

let token_a_amount = ctx
    .accounts
    .bonding_curve
    .real_sol_reserves
    .checked_sub(ctx.accounts.global.migrate_fee_amount)
    .ok_or(ContractError::ArithmeticError)?
    .checked_sub(40_000_000)  // Hardcoded 0.04 SOL for gas
    .ok_or(ContractError::ArithmeticError)?;

```

The function subtracts a hardcoded value of 0.04 SOL (40*000*000 lamports) for gas fees during pool migration. This presents several issues:

1. During network congestion, gas fees might exceed 0.04 SOL, causing transaction failures
2. During low network activity, 0.04 SOL might be excessive, leading to unnecessary fee costs
3. Future Solana network upgrades might change typical gas costs
4. The hardcoded value doesn’t account for potential changes in SOL’s value relative to transaction costs

For example, if network congestion pushes gas costs to 0.06 SOL:

1. Migration starts with 1 SOL in reserves
2. 0.04 SOL is reserved for gas
3. Actual gas cost is 0.06 SOL
4. Transaction fails due to insufficient gas


### Impact


Pool migrations may fail during network congestion or incur unnecessary costs during low network activity due to inflexible gas fee allocation.


### Recommendation


Make the gas fee configurable in the global state:

```

#[account]
#[derive(InitSpace, Debug)]
pub struct Global {
    // ... existing fields ...
    pub migration_gas_amount: u64,  // Add configurable gas amount
}

// In initialize_pool_with_config:
let token_a_amount = ctx
    .accounts
    .bonding_curve
    .real_sol_reserves
    .checked_sub(ctx.accounts.global.migrate_fee_amount)
    .ok_or(ContractError::ArithmeticError)?
    .checked_sub(ctx.accounts.global.migration_gas_amount)
    .ok_or(ContractError::ArithmeticError)?;

```

## [06] Basis Points Multiplication Function Lacks Input Validation Leading to Silent Failures


The Pump Science protocol uses a basis points multiplication utility function for fee calculations. 

The implementation in `util.rs` lacks input validation:

```

pub fn bps_mul(bps: u64, value: u64, divisor: u64) -> Option<u64> {
    bps_mul_raw(bps, value, divisor).unwrap().try_into().ok()
}

pub fn bps_mul_raw(bps: u64, value: u64, divisor: u64) -> Option<u128> {
    (value as u128)
        .checked_mul(bps as u128)?
        .checked_div(divisor as u128)
}

```

The issues are:

1. No validation that `divisor` is non-zero
2. No validation that `bps` is less than or equal to `divisor`
3. Silent failure through `Option` return type without specific error reasons
4. Potential for unexpected results when `bps > divisor`

For example:

```

// These cases silently return None:
bps_mul(10_000, 1000, 0);  // Division by zero
bps_mul(20_000, 1000, 10_000);  // bps > divisor

```

This is particularly problematic because the function is used for critical fee calculations in the bonding curve’s `calculate_fee()` function.


### Impact


Fee calculations may silently fail or return incorrect results without proper error handling, potentially leading to transaction failures without clear error messages.


### Recommendation


Add input validation and specific error handling


## [07] Trade Event Emission Uses Redundant Clock Calls and Lacks Block Time Validation


The Pump Science protocol emits trade events for indexing purposes in the `handler()` function of the swap instruction. The event emission has two issues:

1. 
Redundant Clock Calls:

```

emit_cpi!(TradeEvent {
// ... other fields ...
timestamp: Clock::get()?.unix_timestamp,  // First Clock::get() call
// ... other fields ...
});

```

if bonding*curve.complete {
emit*cpi!(CompleteEvent {
// … other fields …
timestamp: Clock::get()?.unix_timestamp,  // Second Clock::get() call
// … other fields …
});
}

```

2. No Validation of Block Time:
The timestamp is used directly from `Clock::get()` without any validation that the block time is reasonable or hasn't been manipulated by the validator.

This could lead to:
1. Unnecessary computational overhead from redundant syscalls
2. Inconsistent timestamps between `TradeEvent` and `CompleteEvent` if they're emitted in the same transaction
3. Potential for incorrect historical data if validators manipulate block times

For example:

```rust
// Current implementation might result in:
TradeEvent.timestamp = 1000
CompleteEvent.timestamp = 1001  // Different timestamp from same tx

```

### Impact


Inefficient resource usage and potential for inconsistent event timestamps affecting indexing and historical data accuracy.


### Recommendation


Cache the timestamp and add basic validation


## [08] Bonding Curve Token Metadata Fields Lack Length Restrictions Leading to Excessive Storage Costs


The Pump Science protocol’s bonding curve creation allows creators to specify token metadata through `CreateBondingCurveParams` :

`rust pub struct CreateBondingCurveParams { pub name: String, pub symbol: String, pub uri: String, pub start_slot: Option<u64>, }`

The issue is that these metadata fields (`name`, `symbol`, `uri`) lack length restrictions. This presents several problems:

1. Token names and symbols could be unreasonably long, making them impractical for display
2. Long URIs could waste on-chain storage
3. Malicious creators could create tokens with extremely long metadata to increase storage costs
4. No validation for minimum lengths, allowing empty strings

For example:

```

CreateBondingCurveParams {
    name: "A".repeat(1000),  // 1000-character name
    symbol: "B".repeat(500), // 500-character symbol
    uri: "C".repeat(2000),   // 2000-character URI
    start_slot: None,
}

```

This could lead to:

1. Excessive storage costs for the protocol
2. Poor UX in wallets and explorers
3. Potential for spam tokens with unnecessarily large metadata


### Impact


Excessive storage costs and poor UX due to unbounded metadata string lengths.


### Recommendation


Add length restrictions in the `validate()` function


## [09] Bonding Curve Token Account Lock/Unlock Operations Lack Event Emission for Critical State Changes


The Pump Science protocol uses a locker mechanism to control token transfers through freezing/thawing token accounts. This is implemented in `BondingCurveLockerCtx` with two critical functions:

```

pub fn lock_ata<'a>(&self) -> Result<()> {
    // ... freeze account logic ...
    msg!("BondingCurveLockerCtx::lock_ata complete");
    Ok(())
}

pub fn unlock_ata<'a>(&self) -> Result<()> {
    // ... thaw account logic ...
    msg!("BondingCurveLockerCtx::unlock_ata complete");
    Ok(())
}

```

The issue is that these critical state changes only log a message but don’t emit events. This presents several problems:

1. Off-chain indexers can’t reliably track token account freeze/thaw state
2. No permanent on-chain record of when these operations occurred
3. Difficult to audit the history of lock/unlock operations
4. No easy way to monitor for potential unauthorized operations

For example, if monitoring a bonding curve’s lifecycle:

```

// Current logs only show:
"BondingCurveLockerCtx::unlock_ata complete"
"BondingCurveLockerCtx::lock_ata complete"

```

// No structured event data for:

- Who initiated the operation
- When it occurred
- Which accounts were affected


### Impact


Reduced transparency and auditability of token account state changes, making it harder for indexers and monitoring tools to track protocol state.


### Recommendation


Add events for lock/unlock operations


## [10] Empty remove_wl Function Implementation Creates Misleading Security Expectation


The Pump Science protocol includes a whitelist removal function in its program module that is completely empty:

```

pub fn remove_wl(_ctx: Context<RemoveWl>) -> Result<()> {
    Ok(())  // Empty implementation
}

```

This empty implementation presents several issues:

1. 
Misleading Security Expectation:

- Function name suggests whitelist removal functionality
- Empty implementation silently succeeds without performing any action
- Could lead to false assumptions about whitelist management

2. 
Potential Integration Issues:

- External systems might assume whitelist removal works
- No error or warning indicating non-implementation
- Transaction fees charged for no-op operation

3. 
Documentation Mismatch:

- Function exists in public interface
- No indication in code or comments about why it’s empty
- Unclear if this is intentional or oversight

Example problematic scenario:

```

// Admin thinks they're removing a creator from whitelist
await program.remove_wl({
    creator: badActor,
    // ... other accounts
});
// Transaction succeeds but creator remains whitelisted
// Bad actor can still create bonding curves

```

### Impact


False sense of security and wasted transaction fees due to non-functional whitelist removal that appears to succeed.


### Recommendation


Either implement the function properly or make it explicitly unavailable


## [11] Hardcoded Program IDs in Constants Create Deployment Inflexibility and Testing Challenges


The Pump Science protocol uses hardcoded program IDs in its constants file:

```

pub const METEORA_PROGRAM_KEY: &str = "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB";
pub const METEORA_VAULT_PROGRAM_KEY: &str = "24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi";
pub const QUOTE_MINT: &str = "So11111111111111111111111111111111111111112";
pub const CREATION_AUTHORITY_PUBKEY: &str = "Hce3sP3t82MZFSt42ZmMQMF34sghycvjiQXsSEp6afui";

```

This presents several issues:

1. 
Testing Limitations:

- Cannot easily test with different program IDs
- Local development requires exact program deployment addresses
- Integration tests must match mainnet addresses

2. 
Deployment Inflexibility:

- Cannot deploy to different networks without code changes
- No support for testnet/devnet configurations
- Upgrades to dependent programs require code changes

3. 
Security Audit Challenges:

- Hard to verify program ID correctness
- No clear indication of program version requirements
- Difficult to track program dependencies

Example problematic scenario:

```

// Developer trying to test with local Meteora deployment
// Must deploy with exact address or modify source code
let meteora = await anchor.deploy("meteora", {
    address: "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB" // Must match hardcoded ID
});

```

### Impact


Reduced flexibility in deployment, testing difficulties, and potential security risks from hardcoded dependencies.


### Recommendation


Use configuration-based program IDs


## [12] Lock Pool Instruction Lacks Escrow Account Address Validation Leading to Potential Fund Lock


The Pump Science protocol’s `lock_pool` instruction creates and uses escrow accounts for locking LP tokens, but lacks proper validation of the escrow account addresses. In `lock_pool.rs` :

```

pub struct LockPool<'info> {
    // ... other accounts ...
    #[account(mut)]
    /// CHECK lock escrow
    pub lock_escrow: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Escrow vault
    pub escrow_vault: UncheckedAccount<'info>,
}

pub fn lock_pool(ctx: Context<LockPool>) -> Result<()> {
    // ... validations ...
    
    // Create Lock Escrow without address validation
    let escrow_accounts = vec![
        AccountMeta::new(ctx.accounts.pool.key(), false),
        AccountMeta::new(ctx.accounts.lock_escrow.key(), false),
        // ... other accounts ...
    ];

```

The issues are:

1. 
No PDA Validation:

- `lock_escrow` and `escrow_vault` are marked as `UncheckedAccount`
- No validation that addresses match expected PDA derivation
- Could allow incorrect escrow accounts to be used

2. 
Missing Ownership Checks:

- No validation of escrow account ownership
- No validation of escrow vault ownership
- Could allow unauthorized escrow accounts

For example:

```

// Attacker could provide their own escrow account
let malicious_escrow = Keypair::new();
await program.lock_pool({
    lock_escrow: malicious_escrow.publicKey(),
    // ... other accounts ...
});
// LP tokens could be locked in wrong escrow

```

### Impact


LP tokens could be locked in incorrect or malicious escrow accounts, potentially leading to permanent fund loss.


### Recommendation


Add proper PDA and ownership validation


# Disclosures


C4 is an open organization governed by participants in the community.

C4 audits incentivize the discovery of exploits, vulnerabilities, and bugs in smart contracts. Security researchers are rewarded at an increasing rate for finding higher-risk issues. Audit submissions are judged by a knowledgeable security researcher and disclosed to sponsoring developers. C4 does not conduct formal verification regarding the provided code but instead provides final verification.

C4 does not provide any guarantee or warranty regarding the security of this project. All smart contract software should be used at the sole risk and responsibility of users.