# RQ3: LLM-Based Vulnerability Detection

This directory contains the evaluation framework and results for assessing state-of-the-art LLMs on Solana vulnerability detection.

## Directory Structure

```
rq3/
├── evaluator/
│   ├── main.py             # LLM evaluation entry point
│   ├── processor.py        # Detection pair construction
│   ├── calc_vuln_pairs.py  # Counts total detection pairs
│   ├── reasoner.py         # Tree-of-Thoughts (ToT) pipeline
│   ├── cwe_dict.json       # CWE taxonomy used for classification
│   └── pyproject.toml
├── results/                # Raw LLM outputs (one JSON per model)
│   ├── claude45.json
│   ├── gpt52.json
│   ├── deepseek-r1.json
│   └── qwen3.json
├── analyze.py              # Metrics computation and figure generation
├── analyze.ipynb
├── rq3_performance.pdf     # Overall F1/P/R bar chart
├── rq3_heatmap_f1.pdf      # Per-CWE-Pillar F1 heatmap across models
├── rq3_heatmap_det.pdf     # TP and FP distribution heatmap
└── rq3_temporal_f1.pdf     # F1 score trends over audit date
```

## Hierarchical Evaluation Metrics

Standard binary classification metrics evaluate only the final leaf-node CWE prediction and ignore partial correctness along the taxonomy path. We adopt hierarchical precision, recall, and F1 following established work on hierarchical text classification:

```
hP  = sum_i |P_i ∩ T_i| / sum_i |P_i|
hR  = sum_i |P_i ∩ T_i| / sum_i |T_i|
hF1 = 2 * hP * hR / (hP + hR)
```

where `P_i` is the set of predicted CWE nodes along the taxonomy path for instance `i`, and `T_i` is the ground truth path. The intersection `P_i ∩ T_i` counts correctly predicted nodes at any level.

Example: if the ground truth path is CWE-284 → CWE-282 → CWE-283 and the model predicts CWE-284 → CWE-282 → CWE-708, traditional metrics yield zero precision and recall. Hierarchical metrics give hP = hR = 2/3, crediting the two correctly identified ancestor nodes.

## Detection Pair Construction

Vulnerabilities that share affected source files are grouped into a single detection pair. This ensures the LLM has complete code context when multiple vulnerabilities co-occur in the same files.

Construction algorithm: for a project with findings A (files {f1, f2}), B (file {f3}), and C (files {f1, f4}), findings A and C share f1 and are merged into one pair covering files {f1, f2, f4} with ground truth labels {CWE-284, CWE-703}. Finding B has no overlap and forms its own pair ({f3}, {CWE-682}). This snowball expansion is a connected-components computation over the bipartite finding↔file graph: two findings are joined whenever they touch a common file, and the closure is taken transitively (so if A and C share f1, and C and D share f4, all of A/C/D end up in one pair even if A and D share no file directly). The dataset contains **857** such detection pairs from 229 source-available projects.

`evaluator/processor.py` implements this algorithm (see `process_reports`). `evaluator/calc_vuln_pairs.py` reproduces the total count from `dataset/findings/`.

### Worked example from the dataset

`GMX Solana Protocol - Zenith Audit Report.json` contains three findings that all touch `gmsol-timelock/src/lib.rs`:

| Finding | File(s) | CWE path (1→2→3) |
|---|---|---|
| #12 "`unchecked_revoke_role()` will always fail due to wrong signer" | `bypass/revoke_role.rs`, `lib.rs` | CWE-435 → CWE-436 → CWE-115 |
| #13 "Incorrect access control for `set_expected_price_provider()`" | `lib.rs` | CWE-284 → CWE-285 → CWE-863 |
| #23 "Malicious `TIMELOCK_KEEPER` can set `is_signer` for another `TIMELOCK_KEEPER`'s account to steal funds" | `lib.rs` | CWE-284 → CWE-285 → CWE-863 |

`revoke_role.rs` and `lib.rs` are joined by finding #12; `lib.rs` alone links in #13 and #23 via file overlap, so the snowball closes all three into a single pair:

```
files 𝓕 = {gmsol-timelock/src/instructions/bypass/revoke_role.rs,
           gmsol-timelock/src/lib.rs}
raw ground-truth paths (3) = [CWE-435→CWE-436→CWE-115,
                               CWE-284→CWE-285→CWE-863,
                               CWE-284→CWE-285→CWE-863]
deduplicated ground-truth 𝓖 = {CWE-435→CWE-436→CWE-115,
                                CWE-284→CWE-285→CWE-863}
```

Findings #13 and #23 classify to the identical taxonomy path, so deduplication collapses them into one ground-truth label — the pair is evaluated against 2 unique paths, not 3. Note a fourth finding (#35, "Unexpected transfer of treasury fees") also touches a file named `lib.rs`, but it resolves to a different module (`gmsol-store/src/lib.rs`); since overlap is computed on resolved absolute paths rather than basenames, it correctly forms its own separate pair.

## Prompt Template

Each level of the ToT traversal (Algorithm~1) issues one LLM call via `evaluator/reasoner.py::HierarchicalAuditor._detect_cwes`. The prompt has three parts:

1. **System prompt** — fixed instructions plus the current level's CWE candidate list (Pillar CWEs on the first call; the direct children of every surviving chain on subsequent calls):

   ```text
   You are an expert Solana smart contract security auditor.
   Your task is to analyze the provided Rust source code to identify potential security issues.
   You are provided with a list of specific CWEs. Treat these CWEs as the root causes of potential vulnerabilities/issues.
   From the perspective of each of these root causes, explore the code to determine if any security issues exist that stem from them.
   Be thorough: the code may contain multiple vulnerabilities / issues arising from different root causes, or multiple instances of the same root cause.
   You must only consider the following list of CWE candidates:
   - CWE-284: Improper Access Control (...)
   - CWE-435: Improper Interaction Between Multiple Correctly-Behaving Entities (...)
   ... (all 10 Pillar CWEs at level 1; their children at deeper levels)

   Analyze the code carefully. If you find any security issues arising from these root causes, report them.
   If a specific CWE type is present, report it. If no issues are found, explicitly state an empty finding field. You can report multiple instances for the same CWE type if applicable.
   Output your findings in YAML format strictly following this schema:
   <schema below>
   ```

2. **User prompt** — the concatenated source of every file in $\mathcal{F}$, one fenced Rust block per file (`File: <name>\n\`\`\`rust\n<content>\n\`\`\``).

3. **Output schema** — the model returns structured YAML validated against `model.py::DetectionResult`:

   ```yaml
   reasoning: "The contract exists three kinds of CWE vulnerabilities. First, the function `withdraw` allows any user to withdraw funds because it lacks a check to ensure the caller is the owner...."
   findings:
     - title: "Lack of Access Control in withdraw Function"        # (title of the vulnerability finding)
       description: "The function `withdraw` does not verify the caller's identity, allowing unauthorized users to withdraw funds."  # (detailed description)
       category: "CWE-1000"   # (CWE ID in the candidate list, e.g., CWE-1000)
       severity: "critical"   # (info, low, medium, high, critical)
       confidence: 0.99       # (confidence score for your findings, float between 0 and 1)
   ```

Detected CWEs at level *n* that fall outside that level's ground-truth set are still recorded as final (false-positive) chains; chains with no detected children are also finalized. This is what lets a single ToT run yield hierarchical, partial-credit predictions instead of one leaf label per pair. Up to 3 retries are made per LLM call on malformed YAML (`reasoner.py:226`).

## Models Evaluated

| Model | hF1 | hP | hR |
|---|---|---|---|
| Claude-4.5 | 0.26 | 0.19 | 0.41 |
| GPT-5.2 | 0.25 | 0.20 | 0.33 |
| DeepSeek-R1-0528 | 0.16 | 0.24 | 0.12 |
| Qwen3-235b-A22b | 0.15 | 0.19 | 0.13 |

Claude-4.5 achieves the best F1 (0.26) and highest recall (0.41). DeepSeek-R1 achieves the best precision (0.24) with lowest recall, consistent with its reasoning model self-correction behavior that rejects uncertain detections.

## Per-CWE Performance

`rq3_heatmap_f1.pdf` shows per-model F1 scores across the 10 CWE Pillars. Key patterns:

- CWE-682 (Incorrect Calculation) achieves the highest average F1 (0.46). Arithmetic errors such as overflow and off-by-one manifest as localized code patterns that fit within LLM attention windows.
- CWE-664 (Resource Management) shows high variance across model sizes (average F1 0.34, std 1.0), indicating that account lifecycle knowledge is partially acquired only by larger models.
- CWE-435 (Integration Errors) achieves the lowest F1 (0.12). These vulnerabilities require understanding external interfaces such as oracle implementations or token-standard inconsistencies that lie outside the analyzed codebase.
- CWE-693 (Protection Mechanism Failure) is systematically under-detected. Detecting an absent protection mechanism requires knowing what should be present, which demands Solana-specific domain knowledge that general-purpose LLMs lack.

## Reproduction

```bash
cd evaluation/rq3
uv run analyze.py    # recompute metrics from results/ and regenerate figures
```

To rerun the LLM evaluation, put your OpenRouter key in `evaluator/.env` (see `evaluator/.env-example`), then run from the repository root. `--model` is one of `gpt-5.2`, `claude-sonnet-4.5`, `deepseek-r1`, `qwen3` (or any raw OpenRouter model ID):

```bash
cp evaluation/rq3/evaluator/.env-example evaluation/rq3/evaluator/.env  # then fill in API_KEY
uv run --project evaluation/rq3/evaluator python evaluation/rq3/evaluator/main.py \
  --findings-path dataset/findings/ \
  --detector-path evaluation/rq3/evaluator/detector_history.json \
  --cwe-db-path evaluation/rq3/evaluator/cwe_dict.json \
  --dataset-path dataset/ \
  --model claude-sonnet-4.5
```

Per-pair outputs are written to `evaluation/rq3/evaluator/output/pairs/<model>/`.
