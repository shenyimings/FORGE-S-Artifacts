# RQ1: FORGE-S Tool Evaluation and Dataset Statistics

This directory contains scripts and results for evaluating FORGE-S and characterizing the Solana vulnerability landscape.

## Directory Structure

```
rq1/
├── evaluation/         # Information extraction and classification evaluation
│   ├── sample/         # Manually annotated ground truth (74 reports)
│   ├── statistic.py    # Dataset-level counts and severity breakdown
│   └── statistic.ipynb
└── landscape/          # CWE distribution and temporal analysis
    ├── landscape.py    # Treemap data, top-10 tables, Pillar severity
    ├── landscape.ipynb
    ├── evolution_analysis.py
    ├── cwe_treemap.pdf
    └── vulnerability_evolution.pdf
```

## Information Extraction Evaluation

FORGE-S was evaluated on 74 manually annotated audit reports. Per-entity-type results:

| Entity Type | Precision | Recall | F1 | Correct | Wrong | Omitted |
|---|---|---|---|---|---|---|
| URL | 0.9875 | 0.9634 | 0.9753 | 79 | 1 | 3 |
| Commit ID | 0.9875 | 0.9294 | 0.9576 | 79 | 1 | 6 |
| Audit Date | 0.9571 | 0.9571 | 0.9571 | 67 | 3 | 3 |
| Vulnerability Info | 0.9795 | 0.9256 | 0.9517 | 572 | 12 | 46 |
| Severity | 0.9523 | 0.9557 | 0.9540 | 539 | 27 | 25 |
| **Macro Avg** | **0.9728** | **0.9462** | **0.9591** | **1,336** | **44** | **83** |

Two annotators reached a Cohen's kappa of 0.88 on the extraction task.

## CWE Classification Evaluation

For vulnerability classification, three annotators independently labeled a random sample of 93 findings. Inter-rater agreement was measured with Krippendorff's alpha (k-alpha = 0.863), exceeding the 0.80 reliability threshold. The k-alpha annotation data is in `evaluation/rq1/evaluation/sample/annotation_findings.yaml`.

## Vulnerability Localization Evaluation

FORGE-S's file-level bug localization was evaluated against the same 93-finding sample (`location_detection` block in `annotation_findings.yaml`): TP 97, FP 8, FN 13, TN 2.

| Precision | Recall | F1 |
|---|---|---|
| 0.9238 | 0.8818 | 0.9023 |

## Pillar-Level Severity Methodology

The treemap and comparison table use a weighted average severity for each CWE Pillar that aggregates all descendant categories:

```
s_pillar(p) = sum(n_c * s_avg(c) for c in descendants(p)) / sum(n_c for c in descendants(p))
```

where `n_c` is the occurrence count of category `c` and `s_avg(c)` is the average CVSS v4.0 score across all findings in that category. This ensures high-frequency categories contribute proportionally to the Pillar-level score.

Example for CWE-284 (Improper Access Control) with three sub-categories having counts 3, 10, 5 and average severities 5.1, 7.0, 6.0:

```
s_pillar(284) = (3*5.1 + 10*7.0 + 5*6.0) / (3 + 10 + 5) = 6.41
```

## Top-10 Vulnerability Frequency

At the specific CWE level, the ten most frequent categories in our dataset:

| Rank | CWE | Count |
|---|---|---|
| 1 | CWE-682: Incorrect Calculation | 197 |
| 2 | CWE-1288: Improper Validation of Consistency | 135 |
| 3 | CWE-1284: Improper Validation of Specified Quantity | 114 |
| 4 | CWE-1339: Insufficient Precision or Accuracy | 108 |
| 5 | CWE-1287: Improper Validation of Specified Type of Input | 100 |
| 6 | CWE-670: Always-Incorrect Control Flow | 88 |
| 7 | CWE-20: Improper Input Validation | 76 |
| 8 | CWE-372: Incomplete Internal State Distinction | 69 |
| 9 | CWE-862: Missing Authorization | 66 |
| 10 | CWE-754: Improper Check for Exceptional Conditions | 66 |

At the Pillar level: CWE-707 (485 occ, 16.8%), CWE-710 (468 occ, 16.2%), CWE-664 (433 occ, 15.0%) are the three most frequent roots (out of 2,887 pillar-level occurrences across 10 Pillar categories).

## Top-10 Vulnerability Severity

The ten highest-severity CWE categories by average CVSS v4.0 score:

| Rank | CWE | Count | Avg Severity |
|---|---|---|---|
| 1 | CWE-807: Reliance on Untrusted Inputs in a Security Decision | 21 | 6.445 |
| 2 | CWE-826: Premature Release of Resource During Expected Lifetime | 14 | 6.046 |
| 3 | CWE-863: Incorrect Authorization | 30 | 5.589 |
| 4 | CWE-1025: Comparison Using Wrong Factors | 12 | 5.046 |
| 5 | CWE-347: Improper Verification of Cryptographic Signature | 14 | 5.007 |
| 6 | CWE-672: Operation on Resource after Expiration | 16 | 4.940 |
| 7 | CWE-346: Origin Validation Error | 48 | 4.922 |
| 8 | CWE-115: Misinterpretation of Input | 14 | 4.904 |
| 9 | CWE-424: Improper Protection of Alternate Path | 24 | 4.764 |
| 10 | CWE-682: Incorrect Calculation | 197 | 4.541 |

At the Pillar level, CWE-284 (avg 4.205), CWE-693 (avg 4.198) and CWE-697 (avg 4.099) carry the highest weighted severity. CWE-826 and CWE-672 share the parent CWE-666 within the CWE-664 Pillar and form an exploitable attack chain: CWE-826 (premature closure) is a prerequisite for CWE-672 (access after closure).

## Vulnerability Evolution

`landscape/evolution_analysis.py` produces `vulnerability_evolution.pdf`, showing temporal trends of CWE category proportions from 2021 to 2026-Q1. Key observations:

- CWE-703 (exception handling) proportion decreases over time (13.7% in 2021 → ~6% by 2024-2025), indicating maturing audit practices.
- CWE-664 (resource management) proportion rises sharply from 2021 (5.9%) to a 2021-2025 plateau around 15-19%, reflecting deeper auditor focus on account lifecycle issues.
- CWE-435 (integration errors) share stays low (~2-4%) through 2021-2023 before climbing to ~4.8% by 2025, tracking the growth of DeFi composability on Solana.
- Audit volume dips in 2024-Q2 (a trough between neighboring quarters) around the Loopscale Lending and Pump.fun DEX incidents, then rises through 2024-Q3/Q4 into 2025.

## Reproduction

```bash
cd evaluation/rq1/evaluation
uv run statistic.py        # dataset counts and severity breakdown

cd ../landscape
uv run landscape.py        # treemap data and top-10 tables
uv run evolution_analysis.py  # temporal trends
```
