# Introduction

We express our gratitude to the Astral Holding OU team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

The Sallar ecosystem is a network of community projects that use Sallar as their payment method. With Sallar, you can pay for products and services, such as clothing, and use it as the currency on the Marketplace Community.
# Review Scope

Repository	https://github.com/Sallar-Labs/Sallar
Commit	787caf381e11f75151a34dac0bed5f3cce8bb8d5
7da2425ed76895ea6a9fd9e8f44073c7318ffce9 (numbers adjustment November 2024)

# Audit Findings

## F-2023-1594 Incorrect Value

Status
fixed
Severity
Critical
Impact
5
Likelihood
5
Description
./programs/solana-contracts/src/lib.rs

\-DUSTSPERBLOCK: the documentation states it must be 20000 Sallar, but it is actually 20 Sallar.

\-initialtokendistribution: the documentation states it must distribute 2,600,000,000 Sallar, but it actually distributes 2,600,000 Sallar.

\-final_mining

The literal 25000000 represents 0.025 Sallar, not 25 Sallar like it is stated in the documentation.

The literal 50000000 represents 0.05 Sallar, not 50 Sallar like it is stated in the documentation.

The literal 100000000 represents 0.1 Sallar, not 100 Sallar like it is stated in the documentation.

The literal 250000000 represents 0.25 Sallar, not 250 Sallar like it is stated in the documentation.

The literal 500000000 represents 0.5 Sallar, not 500 Sallar like it is stated in the documentation.

./programs/solana-contracts/src/token_math.rs

FIRST_BP: the documentation states it must be 20,000, but it is actually 20.

calculatetopbp: according to the documentation, the result of the function must be multiplied by 1000.

TOPBOOSTREDUCTION: according to the documentation, the value must be 1.000004498927, whereas actually, it is 1.0000044986146084.

## F-2023-1596 Stuck Funds

Status
mitigated
Severity
High
Impact
5
Likelihood
3
Description
The expression at line 236 determines the fraction of the final staking account balance that will be distributed in a single final staking cycle; precisely, it is 0.1% of the balance.

Note:

The interval between the final staking iterations is at least approximately 20 hours.

There is no other way to extract the balance of the final staking account except this process.

For any given balance of the final staking account, after, for example, 1200 consecutive final staking cycles there will be left 100 * (1 - 0.001)^1200 = 30 (approximately) percent of the initial balance in the account; 1200 cycles run perfectly back-to-back would take approximately 1000 days; therefore, the example shows that it would take about 1000 days (at a minimum) to extract 70% of the value. To extract 99.5%, it would take approximately 10,000 days at a minimum.

Note that the severity of the issue depends on how much funds are in the final staking account. Due to the design of the program, it is not possible to estimate the amount beforehand.

## F-2023-1595 Unset Or Unsettable Token Metadata

Status
fixed
Severity
High
Impact
3
Likelihood
5
Description
The SPL-token mint account of Sallar token is a PDA of the program. The mint authority is also a PDA of the program, which guarantees that the token management functions can only be performed by the program code.

The program code does not set the token metadata nor provides an API to set it later. This denies the presence of token metadata, which is crucial for integration with many important on and off-chain platforms like blockchain explorers, exchanges, etc.

## F-2023-1601 Missing Validation & Corrupted Data

Status
mitigated
Severity
Medium
Impact
5
Likelihood
1
Description
Functions performing conversions between integers and floating point numbers are expected to perform conversions safely; however, they can still result in corrupted values due to rounding errors.

In most cases, convert_u64_to_f64_safely(x) will have a rounding error for x > 2^53 (~90,000,000 ALL). For example, 2^58 - 5 will be rounded to 2^58.

Both the functions have redundant checks that could be removed.

## F-2023-1600 Denial Of Service State

Status
mitigated
Severity
Medium
Impact
3
Likelihood
3
Description
The code does calculations with the limited precision of f64 type. There are cases when the results of such calculations are directly compared to exact values in order to decide whether to allow some process to move forward - without regard to the fact that the comparison may not succeed due to the precision errors.

This may result in state transition failures on seemingly valid inputs.

In all the cases, it is possible to find such inputs that would circumvent a failure. However, such inputs may be artificial and require breaking some higher-level rules to produce. For example, it could be needed to provide a set of user mining/staking requests with right parameters to exhaust a block and switch to the next one, but those requests could be not available naturally (based on the off-chain users’ actions and their states), so the program owner would have to manufacture fake/dummy requests for this purpose. Nevertheless, the off-chain system is out of scope; therefore, it is impossible to precisely reason about the impact of these issues and ways to work around them.

## F-2023-1599 Missing Validation

Status
fixed
Severity
Medium
Impact
5
Likelihood
1
Description
The arguments mint_nonce, top_block_nonce, bottom_block_nonce, final_staking_account_nonce, final_mining_account_nonce represent the bump seeds of some PDAs that the instruction operates upon. However, arbitrary values can be passed for them, not necessarily the correct bump seeds. If that is the case, the contract state will be corrupted without possibility to fix, and subsequent instruction calls could not work properly.

## F-2023-1598 Immutable Ownership

Status
fixed
Severity
Medium
Impact
5
Likelihood
1
Description
The contract is designed in a way that ownership cannot be transferred.

This may lead to the impossibility to update the owner in critical situations.

## F-2023-1597 Invalid Hardcoded Value

Status
fixed
Severity
Medium
Impact
1
Likelihood
5
Description
The value literal of constant REDUCTION_INVERSE is not directly representable by f64 in Rust. Rust will round the literal silently.

Additionally, according to the documentation, the value must be 0.99999430521433, whereas actually, it is 0.9999943052143271.

## F-2023-1602

Inconsistent Data
Status
fixed
Severity
Low
Impact
1
Likelihood
2
Description
The FINAL_STAKING_ACCOUNT_BALANCE_PART_FOR_STAKING_DIVISION_FACTOR was used as f64 in the previous program version. It was changed into u64 to omit unnecessary conversions, but they themselves were not removed.