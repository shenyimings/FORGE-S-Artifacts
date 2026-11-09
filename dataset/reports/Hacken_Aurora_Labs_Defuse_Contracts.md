# Introduction

We express our gratitude to the Aurora Labs Limited team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

NEAR Intents is a protocol for multichain financial products built by Aurora.

# Review Scope

Repository	https://github.com/near/intents
Initial Commit	160ba5829fea636283375ec462042d51409a6e66

# Audit Findings

## F-2025-8343 Lack of Full Access Key Verification for Fee Update

Severity: Medium


Description
The set_fee and set_fee_collector contract methods are restricted to the DAO and Fee Manager roles. However, the functions do not validate if the call is initialized by holder of Full Access or Function Call Key.

Function Call Keys are given to various applications to simplify user experience, however, the Function Call Key does not allow NEAR transfer from the account. Verifying if 1 yNEAR was attached guarantees the caller is authorized with Full Access Key.

fn set_fee(&mut self, #[allow(unused_mut)] mut fee: Pips) {
    require!(self.fees.fee != fee, "same");
    mem::swap(&mut self.fees.fee, &mut fee);
    FeeChangedEvent {
        old_fee: fee,
        new_fee: self.fees.fee,
    }
    .emit();
}

fn set_fee_collector(&mut self, #[allow(unused_mut)] mut fee_collector: AccountId) {
    require!(self.fees.fee_collector != fee_collector, "same");
    mem::swap(&mut self.fees.fee_collector, &mut fee_collector);
    FeeCollectorChangedEvent {
        old_fee_collector: fee_collector.into(),
        new_fee_collector: Cow::Borrowed(self.fees.fee_collector.as_ref()),
    }
    .emit();
}
This may lead to unauthorized application being able to set arbitrary fee in the contract and change the fee collector account.

## F-2025-8345 Native Balance Exhausting due to Unpaid Storage Increase


Severity: Medium

Description
NEAR blockchain implements storage staking model which locks some NEAR at contracts when additional storage space is allocated. It is considered reasonable to request storage deposits from users to cover the data increase costs.

The execute_intents function execution causes storage increase due to potential user balances allocation and used nonces recording. The function lacks validation if it was attached enough NEAR to cover the storage increase.

fn execute_intents(&mut self, signed: Vec<MultiPayload>) {
    Engine::new(self, ExecuteInspector::default())
        .execute_signed_intents(signed)
        .unwrap_or_panic()
        .as_mt_event()
        .as_ref()
        .map(MtEvent::emit);
}
In such a way, third-party may perform a dust attack on the contract populating storage and decreasing the available amount of NEAR for covering the allocated space. This potentially may lead to temporary DoS situation.

## F-2025-8338 Profitable intents Interception due to Front Running

Severity: Medium

Description
According to the documentation, the off-chain request solvers might include intents containing self-reward to the transaction. The contract is free to process any combination of signed intents.

In case transaction contains a profitable intent, it can be intercepted by blockchain validator. The validator is able to create similar transaction changing the receiver of the profitable intent and put the malformed transaction in block before the initial one.

This may lead to any profitable intents be malformed by the validators and request solvers are not rewarded for the job done.

## F-2025-8337 Inability to Execute Swap due to Fee Calculation Mechanism

Severity: Medium

Description
The contract takes fee from each Token Diff instruction delta. Negative delta means user requests balance decrease and positive - balance increase. The fee is not implicitly included in the balance increase value and should be predicted off-chain based on fee rate and balance decrease value. After all the intents are executed, the contract verifies that sum of deltas for each token is zero.

/// core/src/intents/token_diff.rs

let amount = delta.unsigned_abs();
let fee = Self::token_fee(&token_id, amount, protocol_fee).fee_ceil(amount);

fees_collected
    .deposit(token_id, fee)
    .ok_or(DefuseError::BalanceOverflow)?;
This causes that fee is taken twice from each funds portion (first time from balance decrease and second time from balance increase). There might be impossible to find positive delta value for certain negative delta and fee rate.

This happens due to pos + ceil(pos * fee) = -neg - ceil(-neg * fee) equation is not always solvable relatively to integer pos for given integer neg.

This may lead to issues in resolving exact user swap requests. System might need to propose user to swap less tokens than was requested what might be unacceptable for tokens with little amount of decimals or small supply.

Example for 0.12% fee.

User negative delta is -10026 of some token.

Solver positive delta is pos of the same token.

At first the contract takes ceil(10026 * 0.0012) = 13 fee from negative delta and there is 10013 of token available.

Then for pos = 10000 plus fee ceil(10000 * 0.0012) = 12 the needed amount is 10012 which is less than expected.

And for pos = 10001 plus fee ceil(10001 * 0.0012) = 13 the needed amount is 10014 which is greater than expected.

So far, it is impossible to satisfy exact user request for given conditions. However, changing negative delta to -10025 solves that particular case.

## F-2025-8339 Lack of Signed Payload Versioning

Severity: Low

Description
The signed payload messages are not properly versioned.

pub struct DefusePayload<T> {
    pub signer_id: AccountId,
    pub verifying_contract: AccountId,
    pub deadline: Deadline,
    #[serde_as(as = "Base64")]
    #[cfg_attr(
        all(feature = "abi", not(target_arch = "wasm32")),
        schemars(example = "self::examples::nonce")
    )]
    pub nonce: Nonce,

    #[serde(flatten)]
    pub message: T,
}
The system is developed as upgradeable. For upgradeable systems it is reasonable to implement versioning possibility from the beginning to simplify further support of different version payloads.