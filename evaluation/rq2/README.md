# RQ2: Cross-Ecosystem Comparison

This directory contains scripts and results for comparing Solana's vulnerability landscape with EVM smart contracts and traditional Rust software.

## Directory Structure

```
rq2/
├── statistic_evm.py        # Pillar-level frequency and severity for Solana vs. EVM
├── statistic_evm.ipynb
├── rq3_frequency.pdf       # Frequency comparison figure: Solana vs. EVM
├── rq3_severity.pdf        # Severity comparison figure: Solana vs. EVM
├── rust/
│   ├── statistic_rust.py   # Pillar-level statistics for OSV Rust dataset
│   ├── statistic_rust.ipynb
│   ├── OSV/                # OSV database entries for Rust (995 entries)
│   ├── rq4_frequency.pdf   # Frequency comparison: Solana vs. Rust
│   └── rq4_severity.pdf    # Severity comparison: Solana vs. Rust
└── FORGE-Artifacts/        # EVM baseline dataset (FORGE, ICSE'26)
```

## Baselines

**EVM baseline:** The FORGE dataset (6,454 audit reports, 27,497 findings). Source: [FORGE-Artifacts](https://github.com/shenyimings/FORGE-Artifacts).

**Traditional Rust baseline:** OSV database (995 Rust entries), sourced from GitHub Security Advisories and RustSec. Source: [rust/OSV/](rust/OSV/).

## Comparison with EVM

Solana's overall average severity (3.329) exceeds EVM's (3.016). EVM's extensive audit history includes many code-quality findings: CWE-710 accounts for 33.07% of EVM vulnerabilities but only 16.21% in Solana.

**CWE-697: Incorrect Comparison.** Solana shows a notably higher average severity than EVM (4.099 vs. 2.718). In Solana, CWE-697 findings involve critical logic flaws where comparison operators contradict the intended implementation — for example, reversed greater-than/less-than operators in oracle price validation functions. In EVM, 41.9% of CWE-697 findings are low-severity checklist items from automated audit tools (e.g., "tautologies or contradictions"). Excluding those checklist findings raises EVM's CWE-697 average to 3.32.

**CWE-664: Improper Control of a Resource Through its Lifetime.** Appears 1.8 times more frequently in Solana (15.00% vs. 8.55%). EVM's stateful contract model manages state automatically; Solana's stateless architecture requires programs to explicitly handle account creation, initialization, closure, and reallocation, creating more opportunities for resource lifecycle errors.

**CWE-435: Improper Interaction Between Multiple Correctly-Behaving Entities.** Higher severity in EVM (3.518) than Solana (3.303), reflecting EVM's more mature DeFi composability ecosystem. Solana's interaction failures more often manifest as protection mechanism faults (CWE-693) due to its younger cross-protocol landscape.

**CWE-362: Race Conditions.** Appears 97 times (severity 4.770) in EVM vs. 19 times (severity 3.721) in Solana. EVM's 12-second block time and public mempool enable front-running; Solana's 400ms block time and FIFO Gulf Stream ordering make it structurally harder. Before 2024, most Solana CWE-362 findings were low-severity. Jito's block space auction infrastructure introduced validator-controlled transaction reordering after 2024, and our dataset captures the resulting high-severity front-running vulnerabilities.

**CWE-347: Improper Verification of Cryptographic Signature.** Comparable average severity in both ecosystems (Solana: 5.007, 14 instances; EVM: 4.909, 29 instances). Both face signature replay and malleability issues. Solana's tight integration of signatures into account authentication causes many signature-related failures to be classified as CWE-20 or CWE-346 rather than CWE-347, reflecting how the two platforms handle authentication differently at the architectural level.

**Reentrancy (CWE-1265).** Absent in Solana. Solana enforces a maximum nested call depth, eliminating the entire class of reentrancy attacks that rank as EVM's most severe vulnerability type (281 instances, average severity 5.143).

## Comparison with Traditional Rust

Traditional Rust (OSV) shows a significantly higher average severity (6.56) than Solana (3.329). This gap reflects methodological differences: OSV records only publicly disclosed, CVE-assigned vulnerabilities representing production-threatening exploits, while Solana audit reports cover the full severity spectrum including code quality findings.

**Survivor bias in severity distribution.** CWE-710 (Improper Adherence to Coding Standards) in OSV carries average severity 6.268 because the recorded instances represent coding violations that directly cause security exploits or crashes. The same CWE-710 sub-categories appear in Solana with comparably high scores, confirming that the overall severity difference is a data-collection artifact rather than a real security gap.

**Dominant threat models differ fundamentally.** CWE-664 dominates traditional Rust at 55.7% frequency (severity 7.226), covering buffer overflows (CWE-787), use-after-free (CWE-416), and double-free (CWE-415). In Solana, CWE-664 accounts for 15.0% (severity 3.647) and manifests as account lifecycle issues rather than memory corruption. CWE-682 (Incorrect Calculation) is 4 times more prevalent in Solana (13.6% vs. 3.4%) but at lower absolute severity (4.039 vs. 6.260): Rust's CWE-682 involves low-level platform arithmetic errors, while Solana's occur at the business logic layer and directly threaten financial assets.

## Reproduction

```bash
cd evaluation/rq2
uv run python statistic_evm.py      # Solana vs. EVM Pillar statistics

cd rust
uv run python statistic_rust.py     # Traditional Rust Pillar statistics
```
