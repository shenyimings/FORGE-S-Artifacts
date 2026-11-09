# SVM Spoke Incremental Audit


Summary
-------

Type

Cross Chain

Timeline

From 2025-03-31

To 2025-04-08

Languages

Rust

Total Issues

5 (4 resolved, 1 partially resolved)

Critical Severity Issues

0 (0 resolved)

High Severity Issues

0 (0 resolved)

Medium Severity Issues

1 (1 resolved)

Low Severity Issues

1 (1 resolved)

Notes & Additional Information

3 (2 resolved, 1 partially resolved)

Scope
-----

OpenZeppelin conducted a diff audit of the [across-protocol/contracts](https://github.com/across-protocol/contracts) repository at commit [71e990b](https://github.com/across-protocol/contracts/tree/71e990b3f908ecf994be8723e041b584c7d49318) of the [solana-march-audit-2](https://github.com/across-protocol/contracts/tree/solana-march-audit-2) branch.

In scope were the following pull requests:

*   [PR 892](https://github.com/across-protocol/contracts/pull/892): Remove self-relay
*   [PR 891](https://github.com/across-protocol/contracts/pull/891): Permit historical fill deadline on deposit
*   [PR 861](https://github.com/across-protocol/contracts/pull/861): Update SVM to match new EVM function interfaces
*   [PR 939](https://github.com/across-protocol/contracts/pull/939): Remove enabled deposit route check
*   [PR 942](https://github.com/across-protocol/contracts/pull/942): Test native SOL deposits

Overview of Changes
-------------------

### Remove Self-Relay

Previously, when the [recipient token account matched the relayer token account](https://github.com/across-protocol/contracts/pull/892/files#diff-531de13c511cb5c24ff486b9795755e8e9d37f39320160b36c21d5034cf25664L123) during a relayer fill, no token transfer would occur since the recipient and receiver would be the same account. In the solidity version, this was implemented by returning early, which also bypassed message handling and introduced a [potential bug](https://mirror.xyz/0x9D6b7f5e8d1b9dFea8dDD29c0DbD81687e721601/mrt70ckjaZymv9keUy_TzHVIzjBOQr-Hx_KI1ydFeoQ).

The implemented mitigation was [to simply remove the early return behavior](https://github.com/across-protocol/contracts/pull/880/files) so that the messages were processed in every case. To maintain consistency with the Solidity version, the self-relay optimization in the Solana codebase was also removed.

### Permit Historical Fill Deadline on Deposit

This change removes the lower bound on the fill deadline during deposits, allowing the deadline to be in the past. This allows relayers to fill deposits optimistically, which may be desirable under some circumstances, or to avoid synchronization errors with discrepancies in block timestamps between different chains.

### Update SVM to Match New EVM Function Interfaces

This change updates the naming convention throughout the codebase to match that of the Solidity counterpart. In particular, all references to "v3" have been removed.

### Remove Enabled Deposit Route Check

In previous iterations of the SVM spoke program, the deposit route (i.e., the type of tokens deposited and the destination chain) had to be created by the admin and enabled. An [error would be generated](https://github.com/across-protocol/contracts/pull/939/files#diff-cba2b3c01b372a1d1cec7767a7fb0fd415844f25d8c5e673cf85838db2e6df8eL42) for attempted deposits using a route that had not been enabled. The change under review removes all logic related to creating and enabling route accounts, effectively removing all restrictions around which token types can be sent to which chains. While any route is now possible, since depositors require sending tokens to a [vault account](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/instructions/deposit.rs#L53), the vault token account PDA must be initialized in order to receive tokens. [A helper script is provided](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/scripts/svm/createVault.ts) to initialize the vault PDAs, which can be executed by anyone.

### Test Native SOL Deposits

Since all token transfers in the SVM SpokePool use SPL token transfers, native SOL [must be wrapped](https://spl.solana.com/token#wrapping-sol) in order to be deposited. A [script demonstrating native deposits via wrapping](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/scripts/svm/nativeDeposit.ts) has been added.

Medium Severity
---------------

### Deposit Tokens Transferred from Depositor Token Account Instead of Signer

A deposit action to the SpokePool program is meant to [pull tokens from the caller](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/lib.rs#L213) (i.e., the [signer](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/instructions/deposit.rs#L30)) which may be deposited on behalf of any `depositor` account. However, when the signer is not the depositor, a `transfer_from` operation is performed with the [depositor token account as the source address](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/instructions/deposit.rs#L105). Such an operation would fail unless the depositor had delegated at least the input amount to the state PDA. This presents two consequences:

1.  The signer will not be able to deposit for another account, disabling the intended feature of an [account being able to deposit on behalf of someone else](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/lib.rs#L210).
2.  If any token account delegates to the state PDA, then it is possible for anyone to call the deposit function, passing the victim's account as the [`depositor_token_account`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/instructions/deposit.rs#L45) and performing a successful deposit with arbitrary arguments other than the input token and amount. A malicious user could then submit a deposit with their own recipient address to steal depositor funds. This is possible due to the state PDA being passed as the [authority and signer seed](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/utils/transfer_utils.rs#L19-L28) to the `transfer_checked` call, which, when delegated to, will pass [the transfer validation logic](https://docs.rs/spl-token/latest/src/spl_token/processor.rs.html#274). Note that this scenario is mitigated by the fact that native instruction batching in Solana transactions would typically involve delegation and transferring in one operation, making such a front-running scenario less likely but still possible.

Consider validating the `signer` token account in the same way as `depositor_token_account`, and performing the transfer from the signer token account.

_**Update:** Resolved in [pull request #971](https://github.com/across-protocol/contracts/pull/971), at commit [58f2665](https://github.com/across-protocol/contracts/commit/58f2665031836d5c9b0686870418f22f856a8e5a). The team stated:_

> _We replaced the one “state” PDA with two distinct PDAs—one for deposit and one for fill\_relay. Now users must explicitly delegate to the correct PDA before anyone can pull their tokens, restoring safe third‑party deposits and eliminating the risk of a single authority being misused to steal funds._

Low Severity
------------

### Unclear Vault Initialization Process

[Pull request #939](https://github.com/across-protocol/contracts/pull/939) removes route-enabling functionality in the SVM Spoke, allowing depositors to provide any input token for any destination chain, not only those with the route enabled. Previously, when a route was enabled, the [Associated Token Account (ATA) of the vault for user deposits](https://github.com/across-protocol/contracts/pull/939/files#diff-f52705e50e7f3f740fed5f68d5e5807eccc7354ed50fe83a3ac43c9c8942dad9L173) would be created. The [`DisabledRoute`](https://github.com/across-protocol/contracts/pull/939/files#diff-cba2b3c01b372a1d1cec7767a7fb0fd415844f25d8c5e673cf85838db2e6df8eL42) error would also be generated if a user tried to deposit to a disabled route. Since, by this point, all routes would be enabled, a user may try to submit a deposit where the vault ATA has not been initialized. This will cause the deposit transaction to fail without a descriptive error message.

Thus, if the vault has not been created, the depositor would have to initialize the PDA themselves, for example, using the [`createVault.ts`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/scripts/svm/createVault.ts) script. However, this is not a simple process for the majority of users and it may be unclear why the deposit failed. This could cause users to abandon their deposit attempt and consider alternative bridging solutions. A more straightforward solution would be to initialize the vault during `deposit` using the `init_if_needed` flag as was [previously done](https://github.com/across-protocol/contracts/pull/939/files#diff-f52705e50e7f3f740fed5f68d5e5807eccc7354ed50fe83a3ac43c9c8942dad9L167) when the admin enabled a route.

Consider adding the `init_if_needed` flag or clearly documenting the desired vault creation process within the docstring for the `deposit` function.

_**Update:** Resolved in [pull request #957](https://github.com/across-protocol/contracts/pull/957), at commit [3b8cf77](https://github.com/across-protocol/contracts/commit/3b8cf7769e733ec0fe2c538639142382b64fa483). The team stated:_

> _We added the `init_if_needed` flag to the `vault` account in the `deposit` method._

Notes & Additional Information
------------------------------

### Incomplete Renaming

[Pull request #861](https://github.com/across-protocol/contracts/pull/861) renames several functions and types to remove the "V3" qualifier from the Solana spoke codebase. However, the renaming is incomplete:

*   There is still a [`HandleV3AcrossMessage`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/multicall-handler/src/lib.rs#L87) struct passed to a [`handle_v3_across_message`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/multicall-handler/src/lib.rs#L33) function, with a [`HANDLE_V3_ACROSS_MESSAGE_DISCRIMINATOR`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/utils/message_utils.rs#L9) constant.
*   There are comments \[[1](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/lib.rs#L232), [2](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/instructions/deposit.rs#L1)\] that mention functions with the "V3" qualifier.
*   [`nativeDeposit.ts`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/scripts/svm/nativeDeposit.ts#L81) mentions a "V3" deposit.

Consider updating all functions and comments that mention "V3" for consistency with the rest of the code base.

_**Update:** Partially resolved in [pull request #964](https://github.com/across-protocol/contracts/pull/964), at commit [a78241f](https://github.com/across-protocol/contracts/commit/a78241faa0caf1021034c70cbee0e34354710722). The team stated:_

> _We addressed the issue by removing the mention of "V3" from the rest of the SVM code base. The only exception to this is `handle_v3_across_message` function and its related structs and constants as "V3" notion is also kept in the EVM where [`AcrossMessageHandler`](https://github.com/across-protocol/contracts/blob/solana-march-audit-2/contracts/interfaces/SpokePoolMessageHandler.sol) still has the `handleV3AcrossMessage` in its interface._

### Misleading Documentation

Throughout the codebase, multiple instances of misleading documentation were identified:

*   [Line 68](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/scripts/svm/nativeDeposit.ts#L68) of `nativeDeposit.ts` states that the vault ATA is created when the route is enabled. However, route enabling has been removed in [pull request #939](https://github.com/across-protocol/contracts/pull/939).
*   [Line 1](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/scripts/svm/createVault.ts#L1) of `createVault.ts` says that the script is used by the chain admin, but it can be used by anyone to create the vault PDA when necessary.

Consider correcting the aforementioned comments to improve the overall clarity and readability of the codebase.

_**Update:** Resolved in [pull request #963](https://github.com/across-protocol/contracts/pull/963), at commit [24017ac](https://github.com/across-protocol/contracts/commit/24017acd54fabf4216f23f733aca2eb421b9c807). The team stated:_

> _We addressed the issue by clarifying the comments in the scripts._

### Naming Inconsistency

The `fill_relay` function [accepts a `_relay_hash` parameter](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/lib.rs#L409) but the `FillRelay` struct constraint [calls it `relay_hash`](https://github.com/across-protocol/contracts/blob/71e990b3f908ecf994be8723e041b584c7d49318/programs/svm-spoke/src/instructions/fill.rs#L19) (without the leading underscore).

To avoid confusion, consider maintaining consistent variable names when using the `instruction` constraint.

_**Update:** Resolved in [pull request #962](https://github.com/across-protocol/contracts/pull/962), at commit [9f8a199](https://github.com/across-protocol/contracts/commit/9f8a1993c997e6052b02580dcc0bae37e1c51efb). The team stated:_

> _We addressed the issue by using the same variable names when using the `instruction` constraint._

Conclusion
----------

The reviewed changes include incremental updates to the SVM Spoke. They slightly generalize the functionality and improve consistency with the EVM Spoke. No significant issues were in the codebase.

The Risk Labs team is appreciated for the extensive documentation and their forthcomingness throughout the audit period.

[Request Audit](https://www.openzeppelin.com/cs/c/?cta_guid=dd34a2f8-61fa-4427-8382-ae576a88942a&signature=AAH58kGeU4tTnv4-etNTlzFJLKXQxm24fA&utm_referrer=https%3A%2F%2Fwww.openzeppelin.com%2Fresearch&portal_id=7795250&pageId=189745615381&placement_guid=7809b604-3f30-4cd5-be58-36982828e327&click=7f64c5c8-2e06-4d9d-a551-7777311aa1fe&redirect_url=APefjpHSaKCkkIcWUgAVSf94RkrGmZhxRgKHKFuu4zPa9AxR8COtQ77Fqsyp058bZK_hOEFF9NXM6T37jontBhukvlAbbSilTTds9ogORFIKnwHO0R-16KkB0d5hRDPr6oHbY3y8F3lSSFCFOrOalakD7aVX8pMeM7y-IlIT4KrMqv621tmdF-W7VhcvpK8vOrNFFYmGWZ88cDLgjjzdEUlcr1L8WWT-WrCB_HvQGxChS-hS3xdLmgR_-BTkBHYjq9bnsjtrjCBP&hsutk=351cc181285977e4c1135a3559acf4f5&canon=https%3A%2F%2Fwww.openzeppelin.com%2Fnews%2Fsvm-spoke-incremental-audit&ts=1767596172985&__hstc=214186568.351cc181285977e4c1135a3559acf4f5.1767592763816.1767592763816.1767595160995.2&__hssc=214186568.7.1767595160995&__hsfp=1569236428&contentType=blog-post "Request Audit") hbspt.cta.\_relativeUrls=true;hbspt.cta.load(7795250, '7809b604-3f30-4cd5-be58-36982828e327', {"useNewLoader":"true","region":"na1"});