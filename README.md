# [FORGE-S] An Empirical Study on Solana Ecosystem through a Large-Scale Vulnerability Dataset

[![ICSE 2027](https://img.shields.io/badge/ICSE-2027-blue)](#citation)
[![Dataset](https://img.shields.io/badge/Dataset-blue)](dataset-curated)
[![Solana](https://img.shields.io/badge/ecosystem-Solana%20%2F%20Rust-9945FF)](#dataset-statistics)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)
![Visitors](https://visitor-badge.laobi.icu/badge?page_id=shenyimings.FORGE-S-Artifacts)

**FORGE-S is the largest public dataset of real-world Solana program vulnerabilities: 2,603 CWE-classified findings extracted from 308 professional audit reports covering 353 Solana projects, with the audited Rust source code and file-level vulnerability locations.** It is built by FORGE-S, the Solana extension of the [FORGE](https://github.com/shenyimings/FORGE-Artifacts) framework (ICSE'26), and is the artifact of our ICSE 2027 paper ([citation](#citation)).

This repository contains:

1. **[Dataset](dataset/)**: audit reports, structured findings (JSON) and the source code of the audited Solana programs.
2. **[FORGE-S tool](forge-s/)**: an LLM pipeline that extracts vulnerabilities from audit reports, classifies them into CWE, fetches the audited source code and localizes each vulnerability to source files.
3. **[Evaluation](evaluation/)**: results and reproduction scripts for all research questions (RQ1–RQ3), including a CWE benchmark of four LLMs on Solana vulnerability detection.


## At a Glance

- **Scale:** 308 audit reports, 353 Solana projects, 2,603 vulnerability findings, 194 CWE categories; 244 reports (281 projects, 31,772 Rust files) come with matched source code.
- **Most common Solana weaknesses:** incorrect calculation (CWE-682, 197 findings), improper validation of consistency (CWE-1288, 135) and of specified quantities (CWE-1284, 114).
- **Solana vs. EVM:** Solana's average severity (3.33) exceeds EVM's (3.02). Resource-lifecycle bugs (CWE-664) are 1.8× more frequent on Solana because programs must manage account creation, initialization, closure and reallocation explicitly, while reentrancy, EVM's most severe class, does not occur.
- **LLMs on Solana:** the best of four state-of-the-art LLMs reaches only 0.26 hierarchical F1 on 857 detection pairs; protection-mechanism failures (CWE-693) are systematically missed.

## FORGE Family

FORGE-S is part of the FORGE family of LLM-built smart contract vulnerability datasets. All members share the same report-to-dataset methodology and CWE-based labeling.

| Project | Ecosystem | What it is | Best for | Paper |
|---|---|---|---|---|
| [FORGE](https://github.com/shenyimings/FORGE-Artifacts) | EVM / Solidity | The FORGE framework and the large-scale dataset it built: 27,497 findings from 6,454 audit reports | Large-scale studies, training data | ICSE'26 |
| [FORGE-Curated](https://github.com/shenyimings/FORGE-Curated) | EVM / Solidity | Manually verified dataset of 2,485 findings from 323 recent reports (Dec 2024 – Feb 2026) by 11 top audit teams | Leakage-resistant LLM benchmarking | ICSE'26 |
| **FORGE-S** (this repo) | Solana / Rust | Solana dataset of 2,603 findings from 308 reports, plus the FORGE-S tool with file-level localization | Solana security research, LLM evaluation on Solana | ICSE'27 |

### Relation to FORGE

FORGE-S keeps FORGE's pipeline (MapReduce extraction from audit reports, hierarchical Tree-of-Thoughts CWE classification, source-code fetching) and extends it with a ReAct agent that navigates the audited codebase and maps each finding to the source files it affects (`files` field), reaching 0.90 F1 against manual annotation.

## Repository Structure

```
FORGE-S-Artifacts/
├── dataset/
│   ├── reports/                # 244 audit reports with matched source code (PDF/Markdown)
│   ├── reports-without-source/ # 64 audit reports without matched source code
│   ├── contracts/              # Source code of audited Solana projects (mostly git submodules)
│   └── findings/               # Structured vulnerability data, one JSON per report
├── forge-s/                    # FORGE-S tool source code
│   ├── extractor/
│   ├── classifier/
│   ├── fetcher/
│   ├── localizer/
│   ├── main.py
│   └── config.yaml
├── evaluation/
│   ├── rq1/                    # FORGE-S tool evaluation and dataset statistics
│   ├── rq2/                    # Cross-ecosystem comparison (EVM and Rust)
│   └── rq3/                    # LLM-based vulnerability detection evaluation
├── visualization/              # Interactive landscape and evolution visualizations (static HTML)
├── scripts/
│   └── gen_gitmodules.py       # Regenerates .gitmodules from the contract submodules
└── CITATION.cff
```

## Important Notes on the Dataset

- Repositories in `contracts/` that retain `.git` metadata are included as git submodules. Clone with `git clone --recurse-submodules` to fetch all of them (large), or fetch only the ones you need, e.g. `git submodule update --init "dataset/contracts/<name>"`.
- Submodules are not automatically checked out to the audited commit. To work with the specific version that was audited, manually `checkout` the `commit_id` from the finding JSON.
- For some projects, the commit ID from the audit report is no longer in the main repository tree. These have been manually downloaded and do not contain `.git` metadata.
- Projects whose upstream repositories have since been deleted or made private are included as plain source snapshots of the audited commit rather than as submodules.
- When the exact audited commit has been deleted upstream, we selected the nearest available preceding commit.
- Reports in `reports-without-source/` lack matched source code and are used only for ecosystem distribution analysis (RQ2), not for LLM detection evaluation (RQ3).

## Dataset Statistics

FORGE-S is the most comprehensive collection of Solana program vulnerabilities to date, derived from real-world professional security audits.

| Statistics | Numbers |
|---|---|
| Total audit reports | 308 |
| Total Solana projects | 353 |
| Reports with source code | 244 |
| Projects with source code | 281 |
| Total Rust files (source-available) | 31,772 |
| Average Rust files per project | 113 |
| Average lines of code per project | 29,714 |
| Total vulnerability findings | 2,603 |
| Total CWE categories | 194 |
| **Severity Breakdown** | |
| Critical | 91 |
| High | 353 |
| Medium | 520 |
| Low | 858 |
| Informational | 658 |
| N/A | 123 |

Full per-project statistics are available in [evaluation/rq1/](evaluation/rq1/README.md).

## JSON Schema

Each file in `dataset/findings/` follows this schema:

```json
{
    "path": "string",
    "project_info": {
        "url": "string | string[]",
        "commit_id": "string | string[]",
        "address": "string[]",
        "chain": "string",
        "compiler_version": "string",
        "audit_date": "string",
        "project_path": "string | null"
    },
    "findings": [
        {
            "id": "number",
            "category": {
                "1": "string[]",
                "2": "string[]"
            },
            "title": "string",
            "description": "string",
            "severity": "string",
            "location": "string[]",
            "files": "string[]"
        }
    ]
}
```

`project_info.project_path` is `null` for reports without matched source code. `findings.category` keys represent CWE hierarchy levels, from the Pillar-level root (level `1`) down to the most specific CWE (the highest level present), e.g. `{"1": ["CWE-703"], "2": ["CWE-754"], "3": ["CWE-252"]}`.

## FORGE-S Tool

The [forge-s/](forge-s/) directory contains the automated pipeline for constructing the dataset from raw audit reports.

### Installation

```bash
cd forge-s
uv sync
cp .env-example .env  # set API_KEY to your OpenRouter key
```

### Usage

```bash
# Full pipeline: extract → classify → fetch → localize
uv run main.py forge -t <reports_dir> -o <output_dir>

# Individual stages
uv run main.py extract -t <report_file_or_dir> -o <output_dir>
uv run main.py classify -t <json_file_or_dir> -o <output_dir>
uv run main.py fetch -t <json_file_or_dir> -o <output_dir>
uv run main.py localize -t <json_file_or_dir> -o <output_dir>
```

All stages read the model and endpoint from the `llm` section of [forge-s/config.yaml](forge-s/config.yaml). The default is Qwen3-235B-A22B-Instruct-2507 via [OpenRouter](https://openrouter.ai); any OpenAI-compatible endpoint (including a local Ollama server) can be used by changing `base_url` and `model`.

## Evaluation and Reproduction

The [evaluation/](evaluation/) directory contains all scripts and results for the three research questions.

### RQ1: FORGE-S Tool Evaluation and Dataset Statistics

[evaluation/rq1/](evaluation/rq1/README.md) contains:
- Manually annotated ground truth for tool evaluation (`evaluation/rq1/evaluation/sample/`)
- Dataset statistics scripts (`evaluation/rq1/evaluation/statistic.py`, `statistic.ipynb`)
- Landscape analysis scripts with treemap and evolution visualizations (`landscape/`)

FORGE-S reaches 0.96 macro F1 on information extraction (74 annotated reports) and 0.90 F1 on file-level vulnerability localization. See [evaluation/rq1/README.md](evaluation/rq1/README.md) for per-entity-type metrics and the Pillar-level severity methodology.

### RQ2: Cross-Ecosystem Comparison

[evaluation/rq2/](evaluation/rq2/) contains:
- EVM comparison scripts using the [FORGE](https://github.com/shenyimings/FORGE-Artifacts) dataset (`statistic_evm.py`)
- Traditional Rust comparison scripts using the OSV database (`rust/statistic_rust.py`)
- Pillar-level frequency and severity comparison figures

See [evaluation/rq2/README.md](evaluation/rq2/README.md) for detailed category-level analysis of architectural differences between Solana, EVM, and traditional Rust.

### RQ3: LLM-Based Vulnerability Detection

[evaluation/rq3/](evaluation/rq3/) contains:
- Detection pair construction and LLM evaluation pipeline (`evaluator/`)
- Raw results from four LLMs: GPT-5.2, Claude-4.5, DeepSeek-R1-0528, Qwen3-235b-A22b (`results/`)
- Analysis scripts and figures (`analyze.py`, `analyze.ipynb`)

See [evaluation/rq3/README.md](evaluation/rq3/README.md) for hierarchical evaluation metric definitions and detection pair construction methodology.

## FAQ

### What is the largest public Solana vulnerability dataset?

FORGE-S, with 2,603 vulnerability findings from 308 real-world audit reports across 353 Solana projects, each classified into the CWE hierarchy and, for source-available projects, linked to the audited Rust source code and affected files.

### What are the most common vulnerabilities in Solana programs?

At the specific CWE level: incorrect calculation (CWE-682), improper validation of consistency (CWE-1288), improper validation of specified quantity (CWE-1284), insufficient precision (CWE-1339) and improper validation of input type (CWE-1287). At the root level, improper neutralization (CWE-707), coding-standard violations (CWE-710) and resource-lifecycle errors (CWE-664) account for about half of all findings. The full top-10 lists by frequency and by severity are in [evaluation/rq1/README.md](evaluation/rq1/README.md).

### How is Solana security different from Ethereum (EVM)?

Solana programs are stateless and must manage accounts explicitly, so resource-lifecycle bugs (account initialization, closure, reallocation) are 1.8× more frequent than on EVM, while reentrancy, EVM’s most severe vulnerability class, is largely ruled out because the Solana runtime forbids indirect reentrant cross-program invocations. Front-running (CWE-362) is rarer on Solana but became high-severity after validator-controlled transaction ordering (Jito) appeared in 2024. Security intuitions and tooling therefore do not transfer directly between the two execution models. See [evaluation/rq2/README.md](evaluation/rq2/README.md).

### How well do LLMs detect Solana vulnerabilities?

There is still substantial room for improvement, at least under our evaluation setting (guided, CWE-based vulnerability prediction). On 857 detection pairs from 229 source-available projects, the best-performing model (Claude-4.5) achieves a hierarchical F1 of 0.26, followed by GPT-5.2 (0.25), DeepSeek-R1 (0.16) and Qwen3-235B (0.15). Localized arithmetic errors are identified most reliably, while missing protection mechanisms and cross-protocol integration errors remain challenging. Results may differ under other settings, such as different prompting strategies, agentic workflows or tool-augmented analysis. See [evaluation/rq3/README.md](evaluation/rq3/README.md).

### How do I benchmark my own model or tool on Solana vulnerability detection?

Use the findings in `dataset/findings/` together with the audited source in `dataset/contracts/`; the RQ3 evaluator builds detection pairs and scores predictions with hierarchical CWE metrics. Any OpenRouter model can be evaluated with `--model <openrouter-model-id>` (see [evaluation/rq3/README.md](evaluation/rq3/README.md#reproduction)). We are also exploring reward-based environments for evaluating how well agents can exploit Solana vulnerabilities.

### Can I build a vulnerability dataset for another ecosystem?

Yes. FORGE-S takes any collection of audit reports (PDF/Markdown) and produces CWE-labeled findings with source code and file-level locations. For EVM/Solidity, see [FORGE](https://github.com/shenyimings/FORGE-Artifacts) and [FORGE-Curated](https://github.com/shenyimings/FORGE-Curated).

## Contributing

If you find issues with the dataset or have questions, please open an issue. We will respond and work to resolve it. Contributions via pull request are also welcome.

## License

The FORGE-S code and the structured findings are released under the [MIT License](LICENSE). Audit reports and audited project source code remain the property of their respective authors and are redistributed for research purposes only; please contact us if you would like any material removed.

## Citation

If you use the FORGE-S dataset or tool in your research, please cite our ICSE 2027 paper (a machine-readable [CITATION.cff](CITATION.cff) is also provided):

```bibtex
@inproceedings{shen2027solana,
  author    = {Shen, Yiming and Shao, Zhenzhe and Wu, Daoyuan and Wang, Yanlin and Chen, Ting and Sun, Yi and Yang, Xiaohu and Chen, Jiachi},
  title     = {An Empirical Study on Solana Ecosystem through a Large-Scale Vulnerability Dataset},
  booktitle = {Proceedings of the 2027 IEEE/ACM 49th International Conference on Software Engineering},
  series    = {ICSE '27},
  year      = {2027}
}
```

FORGE-S builds on the FORGE framework; if you also use FORGE or its EVM datasets, please cite:

```bibtex
@inproceedings{10.1145/3744916.3764529,
  author    = {Chen, Jiachi and Shen, Yiming and Zhang, Jiashuo and Li, Zihao and Grundy, John and Shao, Zhenzhe and Wang, Yanlin and Wang, Jiashui and Chen, Ting and Zheng, Zibin},
  title     = {FORGE: An LLM-driven Framework for Large-Scale Smart Contract Vulnerability Dataset Construction},
  booktitle = {Proceedings of the 2026 IEEE/ACM 48th International Conference on Software Engineering},
  series    = {ICSE '26},
  pages     = {491--503},
  year      = {2026},
  doi       = {10.1145/3744916.3764529}
}
```
