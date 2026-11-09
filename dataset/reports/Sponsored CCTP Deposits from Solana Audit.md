# Sponsored CCTP Deposits from Solana Audit

**Summary**

**Type:** Cross Chain  
**Timeline:** From 2025-10-27 → To 2025-11-10  
**Languages:** Rust

**Findings**  
Total issues: 9 (8 resolved)  
Critical: 0 (0 resolved) · High: 1 (1 resolved) · Medium: 1 (0 resolved) · Low: 3 (3 resolved)

**Notes & Additional Information**  
3 notes raised (3 resolved)

Scope
-----

OpenZeppelin audited the [across-protocol/contracts](https://github.com/across-protocol/contracts/) repository at commit [30eed19](https://github.com/across-protocol/contracts/tree/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f).

In scope were the following files:

`src
├── error.rs
├── event.rs
├── instructions
│   ├── admin.rs
│   ├── deposit.rs
│   └── mod.rs
├── lib.rs
├── state.rs
└── utils
    ├── mod.rs
    ├── signer.rs
    ├── sponsored_cctp_quote.rs
    └── testable_utils.rs` 

System Overview
---------------

The Sponsored CCTP Source Periphery is a Solana program that lets users initiate cross-chain transfers through Circle’s CCTP v2 while an operator sponsors rent for transient accounts (per-quote nonce PDA and CCTP’s `MessageSent` account). The periphery:

*   verifies an EVM-signed quote from a trusted signer stored on chain
*   enforces single-use nonces via a `UsedNonce` PDA per quote
*   pays upfront and later reclaims rent for temporary accounts, improving UX
*   delegates burning, fee/denylist checks, and message emission to CCTP v2 via CPI

This program is intentionally minimal and acts as a UX layer that facilitates user-selected actions on the destination chain (see “Off-Chain Orchestration & Destination Execution”), while keeping all critical bridging invariants enforced by CCTP.

### Sponsored Feature

A “sponsored” deposit covers the lamports needed to create temporary accounts so that users do not have to.

1.  **Fund rent pool**: The operator pre-funds the `rent_fund` PDA (`["rent_fund"]`).
2.  **User deposit (`deposit_for_burn`)**:
    *   The program refunds the depositor the rent needed to initialize the per-quote `UsedNonce` PDA.
    *   It initializes `UsedNonce` at `["used_nonce", nonce]` and records the quote’s deadline (the deadline is part of the signed quote - users do not set it).
    *   It verifies the EVM signature, checks `source_domain`, and enforces deadline freshness.
    *   It CPIs to `TokenMessengerMinterV2` `deposit_for_burn_with_hook`, designating `rent_fund` as the event rent payer for CCTP’s `MessageSent` account.
    *   It emits events, including the created `message_sent_event_data` address for later reclaim.
3.  **Reclaim CCTP event rent**: After CCTP’s event window elapses, `reclaim_event_account` CPIs into `MessageTransmitterV2` to close `message_sent_event_data`, returning lamports to `rent_fund`.
4.  **Reclaim nonce rent**: After the recorded quote deadline passes, `reclaim_used_nonce_account` closes the per-quote `UsedNonce` PDA, returning its lamports to `rent_fund`.

### CCTP Integration

CCTP v2 is Circle’s burn-and-mint bridge. After the periphery validates the signed quote and records the nonce, it:

*   burns the user’s tokens via `TokenMessengerMinterV2` and emits a Message stored in a `MessageSent` account (rent paid by `rent_fund`)
*   relies on CCTP’s attestation to mint on the destination domain
*   later reclaims the `MessageSent` account rent via `MessageTransmitterV2` once the event window has elapsed

CCTP enforces denylist, fee bounds, token-pairing, message size, and finality constraints. The periphery pre-filters inputs, handles rent UX, and routes the call safely.

### Off-Chain Orchestration & Destination Execution (HyperCore/HyperEVM)

The periphery is a UX facilitator for actions executed on destination stacks:

*   **HyperCore**: supports _transfer-to-core_ and _swap-with-core_ flows
*   **HyperEVM**: supports arbitrary actions via a `MulticallHandler`

* * *

From the UI, the user chooses the desired action. An off-chain component constructs the EVM-typed quote and encodes the appropriate hook data for CCTP (targets, calldata, and parameters for HyperCore/HyperEVM).  
The periphery verifies the quote/signature and passes the hook data through`deposit_for_burn_with_hook`, enabling seamless destination execution while preserving on-chain checks on the source side.

* * *

Security Model and Trust Assumptions
------------------------------------

During the audit, the following trust assumptions were identified:

*   **Trusted EVM Signer authority**: Quotes (including deadline, destinations, hook parameters, fees/limits) are authorized by a trusted EVM address stored in `State`. Security of this key is critical. Destination periphery/contracts are expected to re-validate the same quote.
*   **CCTP correctness**: Safety of burn/mint, denylist, fee bounds, message integrity, and attestation rests with official CCTP v2 programs. The periphery verifies program IDs via account constraints and delegates execution by CPI.
*   **Nonce uniqueness & replay resistance**: Each quote consumes a unique `UsedNonce` PDA. Closing is allowed only after the signed quote deadline.
*   **Rent solvency**: Successful deposits require a sufficiently funded `rent_fund`. Otherwise, the deposit reverts before CCTP CPI.
*   **Deadline Duration**: The duration must be set by the off-chain operator. If a user could choose an arbitrarily large deadline, they could grief the system by freezing the rent tied to their `UsedNonce` account.
*   **CCTP pausability**: `reclaim_event_account` requires `MessageTransmitterV2` to be unpaused. If paused, rent tied to `MessageSent` event accounts cannot be reclaimed, temporarily freezing those funds in `rent_fund` until unpaused.

* * *

*   **CCTP Data Layout Stability**: The program relies on hardcoded calculations for the size (`MessageSentSpace`) and exact struct layout (`MessageSent`) of CCTP event accounts. It is assumed that CCTP upgrades will not modify the byte-layout or increase the initialization size of these accounts.  
    Violations of this assumption could lead to Denial of Service during deposits (due to insufficient rent calculations) or permanently locked rent funds (due to deserialization failures during reclamation).  
    The Across team must actively monitor Circle's CCTP changelogs and upgrade announcements to react immediately to such changes.

* * *

### Privileged Roles

Throughout the in-scope codebase, the following privileged roles were identified:

*   **Upgrade Authority Address**
    *   `initialize`: sets immutable `source_domain` and the initial trusted EVM signer
    *   `set_signer`: updates the trusted EVM signer
    *   `withdraw_rent_fund`: can move lamports from `rent_fund` to any recipient
*   **Trusted EVM Signer (off-chain key whose address is stored in `State`)**
    *   Authorizes quotes (amounts, domains, recipients, deadline, fee caps, hook data)
*   **Operator (off-chain)**
    *   Constructs UI-driven quotes and hook data for HyperCore/HyperEVM interactions

High Severity
-------------

### Program-Sponsored Rents Allow Cheap Griefing That Starves `rent_fund` and Triggers Repeated Temporary DoS

The [`deposit_for_burn`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L105C8-L105C24) function sponsors account rents so that users can initiate cross-domain burns with minimal friction. Each call refunds the user for the `UsedNonce` account rent while the periphery’s `rent_fund` pays to create the CCTPv2 `MessageSent` event account. The event’s space (and rent) scales with the message body length, which includes user-controlled hook data. Event-account rent is only reclaimable after a fixed five-day window, and reclaiming requires operational follow-through. The `UsedNonce` reclaim window appears short (≈1 hour from tests) but this value is not confirmed.

This creates a pronounced cost asymmetry. An attacker can obtain quotes and submit many deposits at the minimum token amount while maximizing hook data up to the protocol limit (e.g., near an 8192-byte message body), driving per-call `MessageSent` rent toward ~0.0579072 SOL. Since the periphery immediately refunds the user’s `UsedNonce` rent and pays the `MessageSent` rent from `rent_fund`, the attacker’s out-of-pocket cost is essentially just the network fee, whereas the periphery must lock significant lamports for at least five days per call.

* * *

The attacker can repeat this until `rent_fund` is depleted, then do it again whenever `rent_fund` is replenished or the reclaimed funds arrive. Since the `MessageSent` space is a direct function of the message body, arbitrary hook data (arbitrary actions on HyperEVM via `MulticallHandler`) lets the attacker deterministically maximize the sponsored rent on each attempt, further accelerating depletion.  
However, even without maximizing the message body size, the default sponsored costs remain high relative to the attacker’s near-zero cost, so the attack is still quite feasible.

* * *

The impact is a recurring, temporary DoS. Once `rent_fund` is drained, new deposits that rely on sponsorship cannot proceed, effectively halting normal user activity. The five-day reclaim window delays recovery, and operational requirements for reclaim add overhead.  
As the attacker can wait for top-ups or reclaimed lamports and then immediately drain them again, the system can be kept in a near-permanent DoS loop at very low attacker cost.

* * *

Consider shifting protections to the off-chain quote generator and using signature validation as the enforcement boundary.

*   **1\. Strengthen Off-Chain Controls (Primary Defense):**
    
    *   Enforce a strict cap on `hook data` length, well below the protocol maximum.
    *   Require a meaningful minimum token amount that is at least commensurate with the worst-case sponsored rent.
    *   Rate-limit quote issuance and deposits per originator.
*   **2\. Implement On-Chain Backstop:**
    
    *   As a minimal backstop, add an on-chain `minimum_deposit_amount` check. This ensures basic economic symmetry even if off-chain controls fail or are misconfigured.
*   **3\. Monitoring & Resilience:**
    
    *   Actively monitor the `rent_fund` account to alert on depletion.
    *   Consider implementing a fallback mechanism: If `rent_fund` is empty, the transaction can still succeed by switching the `event_rent_payer` to the user and not refunding the `UsedNonce` rent. This allows the periphery to remain operational even when the `rent_fund` cannot cover the rent fees. If this fallback is implemented, consider updating `reclaim_event_account` and `reclaim_used_nonce_account` to return rent to the original payer rather than `rent_fund`.

_**Update:** Resolved in [pull request #38](https://github.com/UMAprotocol/across-contracts-private/pull/38), at commit [4f9cc6d](https://github.com/UMAprotocol/across-contracts-private/commit/4f9cc6d460a687928f45c4e2744975517468545f)._

Medium Severity
---------------

### Destination Hook Fails Can Permanently Lock Funds

* * *

The [`deposit_for_burn` function](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L105) invokes CCTP V2’s `deposit_for_burn_with_hook` function, which immediately burns tokens on the source chain and relies on destination-side message handling to run arbitrary hook logic. In this flow, the destination program must execute successfully for funds to be minted or released.  
If the destination hook reverts—for example due to time-sensitive checks or changing on-chain conditions—the receive path fails, and the message remains unprocessed, while the source-side burn remains irreversible.

* * *

CCTP V2 removes V1-style replacement/rescue entry points. As a result, once tokens are burned, parameters such as recipients or hook data cannot be changed. If the destination hook’s preconditions cannot be satisfied in future retries, the transfer can remain stuck indefinitely.  
Operational cleanup helpers like event-account reclamation do not recover value and do not mitigate this failure mode.

* * *

Consider pre-simulating the exact destination call (accounts, calldata/message body, and timing assumptions) before initiating the burn, and presenting a clear user-facing warning that no refund/unburn is available if the destination hook fails.

_**Update:** Acknowledged, will resolve. The team has some partial mitigations in place and intends to add a UI-level notice to make end-users aware of this CCTP behavior. The team stated:_

> * * *
> 
> _We acknowledge that under CCTP v2 the token burn on the source chain is irreversible - there is no “rescue” or “unburn” mechanism if the destination-side hook cannot complete. This is a fundamental constraint of CCTP v2, not something that can be addressed on the source chain.  
> Our mitigation strategy therefore focuses on controlling destination behavior and off-chain pre-validation: ###**1\. Strict Control of Mint Recipient & Hook Data** \* Users cannot choose arbitrary `mint_recipient`; our quoting backend signs messages so that CCTP always mints to our single destination handler [`SponsoredCCTPDstPeriphery`](https://github.com/UMAprotocol/across-contracts-private/blob/audit-oct-20/contracts/periphery/mintburn/sponsored-cctp/SponsoredCCTPDstPeriphery.sol) that is currently also being audited. \* Hook data is structured (`SponsoredCCTPQuote`) and signature-validated on destination; malformed or tampered messages should not revert - these simply should return early, keeping the funds safely in the periphery contract. We do recognize currently there is a potential issue of `SponsoredCCTPDstPeriphery` contract reverting on `abi.decode` failures when processing the received message, but this would be addressed as part of EVM contract audit process. This eliminates the risk of users directing mints to arbitrary contracts or untrusted hook receivers. ### **2\. Destination Behavior Is Largely Non-Reverting After `receiveMessage`** Once Circle’s `receiveMessage` has validated attestation and minted the bridged token to the periphery contract, the destination handler attempts to avoid reverts under normal operation: \* Invalid quotes or expired signatures do not revert; they fall back to a “base-token only” path.  
> \* Market-dependent conditions (slippage too high, bridge liquidity unsafe, etc.) do not revert; execution switches to an EVM-fallback path.  
> \* For arbitrary-action flows, the`MulticallHandler` is configured with a non-zero fallback recipient (the periphery itself). \* If any subcall reverts, the multicall wrapper suppresses the revert, emits `CallsFailed`, and drains all tokens back to the periphery. \* The periphery then treats this as a “no-swap/zero-output” scenario without reverting.  
> Thus, many common time-sensitive hook failure scenarios (bad swap outcome, bad market price, arbitrary call reverting, etc.) would not cause destination-side reverts.  
> ###**3\. Off-Chain Pre-Simulation & UX Safeguard** Before we ever perform a `deposit_for_burn_with_hook` on the source chain, the quoting service has full visibility into: \* the exact encoded message, \* the execution mode, \* action data for arbitrary calls, \* recipient, amounts, slippage settings, etc. We would be able to pre-simulate the destination call under current chain state.  
> If simulation fails or conditions suggest the destination execution may not succeed reliably, the quote would not be issued and no burn occurs. We could also provide clear UI-level disclosure noting that CCTP v2 burns are final and cannot be reversed._
> 
> * * *

Low Severity
------------

### Missing Rent-Fund Balance Precheck Causes Late Revert and Poor UX in `deposit_for_burn`

The [`deposit_for_burn`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L105) flow relies on the program’s `rent_fund` PDA to reimburse the user for the per-quote `used_nonce` account and fund the CCTP `MessageSent` event account via CPI. The instruction currently attempts the refund first (`refund_used_nonce_creation`) and later passes `rent_fund` as the `event_rent_payer` to the CCTP program. Both steps require sufficient lamports to be present in `rent_fund` at the time of submission.

However, there is no early validation ensuring that the `rent_fund` holds enough lamports to cover these obligations. If the `rent_fund` balance is insufficient, the transaction reverts when trying to transfer the refund amount to the user on `refund_used_nonce_creation`, wasting user fees and resulting in a poor user experience.

Consider adding an early balance check on `rent_fund` to fail fast before any CPI or account initialization work is performed and disabling UI actions when on-chain checks or cached telemetry indicate the `rent_fund` balance is below the threshold. In addition, consider supporting a non-sponsored fallback path that skips the refund and sets `event_rent_payer` to the user. If this path is enabled, consider updating `reclaim_event_account` and `reclaim_used_nonce_account` to return the rent to the original payer rather than `rent_fund`.

_**Update:** Resolved in pull requests [#57](https://github.com/UMAprotocol/across-contracts-private/pull/57) and [#76](https://github.com/UMAprotocol/across-contracts-private/pull/76), at commit [b9dd893](https://github.com/UMAprotocol/across-contracts-private/commit/b9dd8933ccd5765bd0dc683e09b1f91951da50a8)._

### Repeated PDA Bump Search Wastes Compute Units

The program derives the [`rent_fund`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L28) and [`state`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L25) PDAs without persisting their bump seeds, causing Anchor to search for a valid bump at runtime whenever these accounts are referenced. When a `bump` is omitted, Anchor uses [`find_program_address`](https://github.com/anza-xyz/solana-sdk/blob/9ffe95499bf2a91e735d4b7deadc373b66848f56/address/src/syscalls.rs#L279) under the hood, which iteratively probes bump values and increases compute usage on every invocation. These PDAs are accessed in high-frequency, common functions and CPIs that form the program’s hot paths, in contrast to cold-path PDAs such as `used_nonce`, which are touched primarily during initialization (`deposit_for_burn`) and reclaimed once.

This design inflates and varies compute costs across common paths, which can push near-budget transactions over limits and create avoidable performance instability.

Consider persisting each PDA’s bump during initialization and referencing it explicitly in all constraints and invocations (e.g., `#[account(seeds = [b"state"], bump = state.bump)]`).

_**Update:** Resolved in [pull request #59](https://github.com/UMAprotocol/across-contracts-private/pull/59), at commit [b066738](https://github.com/UMAprotocol/across-contracts-private/commit/b066738291b0505c8d10d6713f3419717acb9d3e). The team stated:_

> _They persisted the `state` PDA bump seed in storage to avoid repeated bump searches, but did not do the same for `rent_fund` because it must remain system-owned (no data) to serve as the payer for CCTP `MessageSent` event account creation, since the System Program can only debit lamports from accounts it owns. They considered pre-funding the event account by manually transferring lamports from a stateful `rent_fund`, but rejected this approach because it depends on accurate data size estimation and could break if CCTP's implementation changes. Instead, they kept `rent_fund` as a stateless, system-owned PDA that signs transfers using seeds, and optimized by selecting a program ID that results in `rent_fund` having bump = 255 (the highest possible value), minimizing the compute cost of bump searches while maintaining compatibility with CCTP's requirements._

### Overuse of `emit_cpi!` on Privileged Paths Increases Compute Cost Without Clear Security Benefit

Several privileged/admin instructions (e.g., [`initialize`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/admin.rs#L42), [`set_signer`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/admin.rs#L83), and [`withdraw_rent_fund`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/admin.rs#L128)) emit events using `emit_cpi!`. CPI-based emission introduces an extra cross-program invocation for each event, adding serialization overhead and increasing compute unit consumption along administrative code paths that are already permissioned. The stated motivation of hardening against log truncation attacks is not compelling for these flows because the threat model inherently assumes cooperative behavior from upgrade authority and administrators.

The additional CPI can meaningfully raise the CU footprint of low-frequency but high-importance maintenance operations, raising the probability of CU-limit failures in composite transactions and lengthening execution time without providing material integrity guarantees. For similar reasons, CPI-backed events on non-critical, low-risk public flows such as [`reclaim_event_account`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L216) or [`reclaim_used_nonce_account`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L251) appear unjustified, since these events are unlikely to serve as authorization-critical signals and offer limited value against log truncation by an already trusted actor.

Consider replacing `emit_cpi!` with standard `emit!` for all privileged/admin instructions and non-critical public flows, retaining `emit_cpi!` only for public, user-triggered paths whose events are security-relevant (for example, deposit/mint/burn finalization signals that are consumed by off-chain components).

_**Update:** Resolved in [pull request #63](https://github.com/UMAprotocol/across-contracts-private/pull/63), at commit [54c3fd7](https://github.com/UMAprotocol/across-contracts-private/commit/54c3fd70771b9b9f58012703963bfdba169bfc20)._

Notes & Additional Information
------------------------------

### Missing Default Pubkey Validation in `withdraw_rent_fund`

The [`withdraw_rent_fund`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/admin.rs#L128-L143) instruction does not validate that the recipient address is not `Pubkey::default()`. The function only checks that the [withdrawal amount is positive](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/admin.rs#L129-L131) and allows the upgrade authority to withdraw funds to any account, including the default pubkey. If the upgrade authority accidentally or maliciously specifies `Pubkey::default()` as the recipient, funds will be sent to an uncontrolled address. While the transaction will succeed, these lamports will effectively become irrecoverable, resulting in a permanent loss of funds from the `rent_fund`.

Consider adding a validation check to prevent withdrawals to the default pubkey.

_**Update:** Resolved in [pull request #60](https://github.com/UMAprotocol/across-contracts-private/pull/60), at commit [28ae33d](https://github.com/UMAprotocol/across-contracts-private/commit/28ae33db2d43d0c3871e021af146107d6bff5d39)._

### Incorrect Documentation Comment for `deposit_for_burn` Parameter

[This comment](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/lib.rs#L126) for the `deposit_for_burn` instruction in the `lib.rs` incorrectly states the `quote` parameter is `ABI-encoded` and `fixed length`. In fact, `quote` is a `SponsoredCCTPQuote` struct (variable length due to `action_data`), serialized by Anchor, not ABI-encoded. ABI encoding occurs inside the `deposit_for_burn` instruction when [`encode_hook_data()`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L158) is called, not before. Misleading documentation can cause developers to misunderstand the input format and when ABI encoding occurs, leading to incorrect assumptions about the instruction's behavior.

Consider updating the comment to state that `quote` is a `SponsoredCCTPQuote` struct serialized by Anchor.

_**Update:** Resolved in [pull request #61](https://github.com/UMAprotocol/across-contracts-private/pull/61), at commit [483e813](https://github.com/UMAprotocol/across-contracts-private/commit/483e813851a9ba2701fe33233a5e4ea99658d5fb)._

### Redundant `current_time` Field in Production `State` Struct Increases Storage and Compute Costs

The `State` struct includes a [`current_time: u64`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/state.rs#L8) field that is only used in test mode. In production, the field is initialized to `0` and never updated, while `get_current_time()` uses [`Clock::get()`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/utils/testable_utils.rs#L43-L46) directly instead of reading from `state.current_time`. Despite being unused, the field still consumes 8 bytes of storage and requires additional compute units per `state` deserialization. This affects hot-path instructions like `deposit_for_burn`, and `reclaim_used_nonce_account`, which deserialize the `state` account on every invocation, causing unnecessary compute overhead.

Consider using a separate test-only account structure if compute becomes critical. Alternatively, consider documenting that `current_time` is unused in production and incurs additional compute cost during state deserialization in the hot path, in exchange for the testing benefit.

_**Update:** Resolved in [pull request #62](https://github.com/UMAprotocol/across-contracts-private/pull/62), at commit [d2413d7](https://github.com/UMAprotocol/across-contracts-private/commit/d2413d7d4a2ec0009d97c6254a623b9d7bd793cd)._

Client Reported
---------------

### Oversized CCTP Messages Can Permanently Lock Rent in MessageSent Event Accounts

In `sponsored-cctp-src-periphery`, [`deposit_for_burn`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L105) allows CCTP messages large enough that the corresponding [`reclaim_event_account`](https://github.com/across-protocol/contracts/blob/30eed1972adf4e90c9fb65fe818d7a6f5e9aca5f/programs/sponsored-cctp-src-periphery/src/instructions/deposit.rs#L216) call can no longer fit within Solana’s packet size limit.

*   `deposit_for_burn` funds a `MessageSent` event account and stores the source message; this transaction only needs a single user signature and fits under the limit.
*   `reclaim_event_account` requires both `destination_message` and the CCTP `attestation`, where the latter contains multiple signatures and can push the transaction size beyond the 1232-byte limit.

For messages near this boundary, `deposit_for_burn` succeeds but `reclaim_event_account` will always fail, making the event account impossible to close. This permanently locks the rent paid by the protocol and enables a griefing vector: an attacker can repeatedly submit “too-large-to-reclaim” messages to drain the rent fund and eventually disrupt new deposits until the fund is manually replenished. User-bridged funds remain safe, but protocol funds are lost and liveness is impacted.

To address this, the protocol should:

*   Enforce an upper bound on the message/payload size in `deposit_for_burn`, derived from `MAX_TX_SIZE` minus a conservative upper bound for `attestation` and instruction overhead, rejecting messages that cannot be safely reclaimed.
*   Optionally reduce reclaim transaction size by reconstructing `destination_message` on-chain from the stored source message plus a few compact instruction arguments (e.g., nonce, finality threshold, fee, expiration), ensuring the reconstructed message exactly matches CCTP’s serialization and still passes `hash_source_fields` / attestation verification.

_**Update:** Resolved in [pull request #75](https://github.com/UMAprotocol/across-contracts-private/pull/75), at commit [6202968](https://github.com/UMAprotocol/across-contracts-private/commit/6202968d7c057a45ed4e316b382b4e2ef9b5fa2c)._

Conclusion
----------

The Sponsored CCTP Source Periphery system facilitates cross-chain token transfers via Circle's CCTP v2 while sponsoring rent for transient accounts, thereby improving UX. The design distributes responsibilities among the operator, users, and the CCTP infrastructure, with the periphery handling quote verification, nonce uniqueness, and rent sponsorship while delegating critical bridging invariants to official CCTP v2 programs.

The most notable risk identified during the audit involves a DoS vulnerability whereby attackers can cheaply drain the `rent_fund` by repeatedly submitting deposits with minimal token amounts while maximizing hook data length, creating a severe cost asymmetry that allows repeated temporary DoS of the system. In addition, a medium-severity issue was identified whereby destination hook failures can permanently lock funds, as tokens are immediately burned on the source chain and CCTP v2 lacks recovery mechanisms for failed destination hooks. Multiple lower-severity issues were also observed, primarily related to insufficient early failure checks when the rent fund lacks sufficient lamports to sponsor account creation, missing validation checks for default recipient address in rent fund withdrawals, and inefficient compute unit usage.

The Risk Labs team is appreciated for their support and responsiveness during the audit.

Ready to secure your code?

[Request an Audit →](https://www.openzeppelin.com/request?utm_campaign=Audit%20Reports%202024&utm_source=mybloglog&utm_medium=social&utm_term=CTARequestAudit&utm_content=AuditBlogPost)