# Introduction

We express our gratitude to the Polymesh team for the collaborative engagement that enabled the execution of this Blockchain Protocol Security Assessment.

Polymesh is a Layer 1 blockchain specifically designed for security tokens, emphasizing compliance within regulated markets. It facilitates the issuance, transfer, and management of security tokens while ensuring adherence to regulatory standards.

Key features include identity verification mechanisms, governance frameworks, and compliance tools that enhance liquidity and create new funding opportunities for organizations. By prioritizing regulatory compliance and security, Polymesh aims to revolutionize the landscape of asset tokenization.


# Review Scope

Repository	https://github.com/PolymeshAssociation/polymesh
Commit	e5cc1de8208e6d3461b40b97ed5cf4338ad96010

# Audit Findings

## F-2024-6616 Benchmarking Gaps in Contract Instantiation Weight Calculation

Status
fixed
Severity
Medium
Impact
2
Likelihood
4
Description
The identified issue pertains to the weight calculation mechanisms for the extrinsics instantiate_with_code_perms, instantiate_with_hash_perms, instantiate_with_code_as_primary_key, and instantiate_with_hash_as_primary_key within the contracts pallet. These extrinsics accept a data parameter, which is subsequently passed to the contract's constructor upon instantiation. An example function definition, along with its corresponding weight function, is shown below:

pallets/contracts/src/lib.rs:463:

#[weight = Module::<T>::weight_instantiate_with_code(&code, &salt, Some(perms)).saturating_add(*gas_limit)]
pub fn instantiate_with_code_perms(
    origin,
    endowment: Balance,
    gas_limit: Weight,
    storage_deposit_limit: Option<Balance>,
    code: Vec<u8>,
    data: Vec<u8>,
    salt: Vec<u8>,
    perms: Permissions
) -> DispatchResultWithPostInfo
Currently, the data vectors are not subject to any explicit size constraints. Furthermore, as illustrated in the code snippet above, the weight benchmarking process—responsible for determining dispatchable fee calculations—does not account for the potential impact of the data parameter. Instead, benchmarks are conducted with empty vectors, resulting in weight estimations that do not adequately reflect the resource consumption associated with non-empty or large data inputs.

This oversight introduces a considerable risk of denial-of-service (DoS) attacks, as malicious actors could exploit the system by submitting extrinsics with excessively large data vectors without being subject to proportional fee adjustments. Such misuse could lead to resource exhaustion, adversely affecting node performance or even causing temporary halts.

## F-2024-6453 Denial of Service Risk Due to Oversized PortfolioId Vectors

Status
fixed
Severity
Medium
Impact
3
Likelihood
2
Description
A denial-of-service (DoS) risk has been identified in the settlement pallet, related to the use of unbounded Vec<PortfolioId> inputs in several extrinsics, including:

affirm_with_receipts

add_and_affirm_instruction

affirm_instruction

withdraw_affirmation

affirm_with_receipts_with_count

affirm_instruction_with_count

withdraw_affirmation_with_count

add_and_affirm_with_mediators

The core issue arises from the fact that the weight calculations for these extrinsics omit the length of the Vec<PortfolioId>. For example, the weight function for add_and_affirm_instruction accounts solely for the legs vector, as shown below:

fn add_and_affirm_instruction_legs(legs: &[Leg]) -> Weight
Furthermore, there are no restrictions on the number of PortfolioId entries that can be passed, allowing attackers to submit transactions with excessively large vectors without bearing a corresponding increase in gas costs.

This creates an opportunity for malicious actors to exploit the system by flooding the network with transactions containing oversized vectors, potentially overloading nodes and leading to degraded performance, such as delayed block production or temporary node halts.

## F-2024-6030 Vulnerable and Unmaintained Dependencies
Status
mitigated
Severity
Medium
Impact
4
Likelihood
1
Description
During a security analysis of the Polymesh node using cargo audit, several vulnerabilities and maintenance issues were identified across both direct and indirect dependencies. This report outlines significant vulnerabilities and concerns within the codebase.

Direct dependency:

Crate: parity-util-mem

Version: 0.12.0

Issues: parity-util-mem is unmaintained

Details: https://rustsec.org/advisories/RUSTSEC-2022-0080 →

Indirect Dependencies:

Crate: rustls

Version: 0.19.1, 0.20.9

Issues: rustls::ConnectionCommon::complete_io could fall into an infinite loop based on network input

Details: RUSTSEC-2024-0336 →

Severity: 7.5 (High)

Solution: Upgrade to >= 0.23.5 OR >= 0.22.4, < 0.23.0 OR >= 0.21.11, < 0.22.0

Crate: webpki

Version: 0.21.4

Issues: webpki: CPU denial of service in certificate path building

Details: RUSTSEC-2023-0052 →

Severity: 7.5 (High)

Solution: Upgrade to >= 0.22.2

Crate: curve25519-dalek

Version: 2.1.3, 3.2.0, 4.1.1

Issues: Timing variability in curve25519-dalek's Scalar29::sub/Scalar52::sub

Details: RUSTSEC-2024-0344 →

Solution: Upgrade to >= 4.1.3

Crate: ed25519-dalek

Version: 1.0.1

Issues: Double Public Key Signing Function Oracle Attack on ed25519-dalek

Details: RUSTSEC-2022-0093 →

Solution: Upgrade to >= 2

Crate: aes-soft

Version: 0.6.4

Issues: aes-soft is unmaintained. It has been merged into the aes crate

Details: https://rustsec.org/advisories/RUSTSEC-2021-0060 →

Solution: Please use the aes crate going forward.

Crate: aesni

Version: 0.10.0

Issues: aesni is unmaintained. It has been merged into the aes crate

Details: https://rustsec.org/advisories/RUSTSEC-2021-0059 →

Solution: Please use the aes crate going forward.

Crate: ansi_term

Version: 0.12.1

Issues: ansi_term is unmaintained

Details: https://rustsec.org/advisories/RUSTSEC-2021-0139 →

Crate: mach

Version: 0.3.2

Issues: mach is unmaintained

Details: https://rustsec.org/advisories/RUSTSEC-2020-0168 →

Crate: parity-wasm

Version: 0.45.0

Issues: Crate parity-wasm deprecated by the author, wasm-tools → is adviced to be used instead.

Details: https://rustsec.org/advisories/RUSTSEC-2022-0061 →

Crate: proc-macro-error

Version: 1.0.4

Issues: proc-macro-error is unmaintained

Details: https://rustsec.org/advisories/RUSTSEC-2024-0370 →

Crate: serde_cbor

Version: 0.11.2

Issues: serde_cbor is unmaintained

Details: https://rustsec.org/advisories/RUSTSEC-2021-0127 →

Crate: atty

Version: 0.2.14

Issues: Unsound. Potential unaligned read

Severity: Low

Details: https://rustsec.org/advisories/RUSTSEC-2021-0145 

## F-2024-6615 Insecure Arithmetic Practices Affecting Weight Calculations
Status
fixed
Severity
Low
Impact
3
Likelihood
1
Description
The issue pertains to the use of unsafe arithmetic operations in weight calculation routines, which play a pivotal role in determining appropriate transaction fees. Inaccurate weight calculations can introduce the risk of Denial of Service (DoS) by failing to align fees with the actual resource usage.

Two specific occurrences have been identified:

In the utility pallet, three instances of unsafe arithmetic appear in the weight calculations for the batch and batch_all functions, identifiable by searching for base_weight + weight. Given that the utility pallet underpins various functionalities across multiple pallets, any vulnerabilities in its weight calculations pose a systemic risk.

In the assets pallet, the weight calculation for the create_asset_with_custom_type extrinsic presents a potential issue. Specifically, the summation of the weights for the create_asset and register_custom_asset_type could lead to overflow. This overflow would result in a misalignment between the calculated fees and the actual computational and storage resources consumed. The implementation is as follows:

#[weight = <T as Config>::WeightInfo::create_asset(
    asset_name.len() as u32,
    asset_identifiers.len() as u32,
    funding_round_name.as_ref().map_or(0, |name| name.len()) as u32
) + <T as Config>::WeightInfo::register_custom_asset_type(custom_asset_type.len() as u32)]
While these scenarios are difficult to exploit and thus present a low probability of occurrence, addressing them is critical to preemptively safeguard against any severe outcomes that may arise from their exploitation.

## F-2024-6557 Weight Calculation Neglect of Portfolio Name Length

Status
fixed
Severity
Low
Impact
1
Likelihood
1
Description
The extrinsic responsible for portfolio creation currently calculates weight without factoring in the length of the PortfolioName. Specifically, the function definition with its associated weight information is as follows:

#[weight = <T as Config>::WeightInfo::create_asset(
    asset_name.len() as u32,
    asset_identifiers.len() as u32,
    funding_round_name.as_ref().map_or(0, |name| name.len()) as u32
) + <T as Config>::WeightInfo::register_custom_asset_type(custom_asset_type.len() as u32)]
This leads to suboptimal and unfair fee calculations, as varying sizes of PortfolioName are treated the same, even though longer names require more storage resources on-chain.

Although the length of the PortfolioName vector is capped at 2048 characters, this discrepancy still allows for potential inefficiencies or minor abuse, where larger names incur the same cost as smaller ones, leading to an imbalanced resource usage.

## F-2024-6413 Ticker Linking Mechanism Deficiency

Status
fixed
Severity
Low
Impact
2
Likelihood
2
Description
The issue arises from the implementation of the link_ticker_to_asset_id extrinsic in the asset pallet. This function establishes a connection between a ticker and an AssetID but fails to validate whether the asset is already linked to an existing ticker. As a result, a new connection is created without properly disconnecting the original ticker. Specifically, the TickerAssetID continues to store the initial ticker-AssetID pair, leading to persistent inconsistencies.

This oversight prevents the original ticker from being transferred, re-registered, or linked to another asset, as the ensure_ticker_not_linked function fails for the ticker. Consequently, tickers may become frozen, introducing risks of storage discrepancies and potential ticker manipulation.

## F-2024-6148 Insufficiencies in Admin Removal Mechanisms

Status
fixed
Severity
Low
Impact
1
Likelihood
4
Description
The current implementation of the multisig pallet allows for the addition of an admin to a multisig account when consensus is reached among the signers. However, there are notable insufficiencies in the admin removal mechanisms. Currently, signers can only remove the admin by assigning another account to this role. The only way to completely remove the admin, once added, is for the admin account itself to execute the remove_admin_via_admin action for its own removal.

While this does not pose an immediate security threat, it creates a risk of excessive centralization of power within the admin account. This becomes problematic if the admin account misbehaves or becomes compromised, as signers have limited options for resolving the issue.

## F-2024-6037 Inconsistent Proposal Handling When Changing Confirmation Requirements
Status
fixed
Severity
Low
Impact
1
Likelihood
4
Description
The issue arises in the change_sigs_required and change_sigs_required_via_admin methods within the multisig pallet, due to a logical inconsistency in how the number of required approving votes is managed. Specifically, when the required vote count changes, existing proposals are not automatically reassessed for eligibility for execution or rejection.

Currently, existing proposals are processed based on the updated threshold only when approve or reject is explicitly called. If the required number of approving votes is reduced, proposals that meet the new criteria for execution remain unprocessed until an additional vote is received. Similarly, if the required number of votes is increased, proposals that should be rejected remain unresolved until further rejecting votes are cast.

While this behavior does not introduce immediate security risks, it can cause unnecessary delays and prevent resolved proposals from being processed. This inconsistency disrupts the proposal lifecycle and undermines the system's overall reliability.