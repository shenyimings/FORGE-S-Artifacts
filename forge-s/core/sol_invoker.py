import os
import sys
import ell
import yaml
import dotenv
from loguru import logger
from openai import OpenAI
from typing import Any

with open("config.yaml", "r") as f:
    CONFIG = yaml.safe_load(f)

VERBOSE = CONFIG["global"]["verbose"]
INTERVAL = CONFIG["global"]["interval"]
TIMEOUT = CONFIG["global"]["timeout"]
LOG_LEVEL = CONFIG["global"]["log_level"]
logger.remove()
logger.add(
    sink=sys.stdout,
    colorize=True,
    level=LOG_LEVEL,
)
LOG_DIR = CONFIG["global"]["log_dir"]
LOG_FILE = CONFIG["global"]["log_file"]
MAX_RETRIES = CONFIG["global"]["max_retries"]
dotenv.load_dotenv(CONFIG["global"]["env_path"])
API_KEY = os.getenv("API_KEY")

MODEL = CONFIG["llm"]["model"]
BASE_URL = CONFIG["llm"].get("base_url", None)
TEMPERATURE = CONFIG["llm"]["parameters"]["temperature"]
CHUNK_LENGTH = CONFIG["extractor"]["chunk_length"]
CLIENT = OpenAI(
    api_key=API_KEY if API_KEY else "ollama",
    base_url=BASE_URL,
    timeout=TIMEOUT,
    max_retries=MAX_RETRIES,
)



os.environ["TIKTOKEN_CACHE_DIR"] = (
    os.getenv("TIKTOKEN_CACHE_DIR") or "extractor/__pycache__"
)
ell.init(verbose=VERBOSE, store=LOG_DIR, autocommit=False)
ell.config.register_model(
    MODEL, CLIENT, supports_streaming=CONFIG["llm"]["parameters"]["streaming"]
)


@ell.simple(model=MODEL, client=CLIENT, temperature=TEMPERATURE)
def invoke_map(document: str):
    return [
        ell.system(
            "You are Axiom, an AI expert specializing in smart contract security and audit report analysis. All extracted details must be directly supported by the document. For any missing information, you MUST use a null value."
        ),
        ell.user(
            """
You are acting as a 'Mapper' in a MapReduce process. You will be given a single chunk of a long smart contract audit report. Your task is to extract project information and all security vulnerability findings from this chunk.

You MUST format your output strictly in YAML.

**YAML Output Structure:**

```yaml
# Explain why you are extracting the following project information.
reason_for_project_info: |
  The document chunk mentions the project's GitHub repository and an on-chain address, which are crucial for identifying the audited codebase.

project_info:
  # The URL(s) of the project's source code repository (e.g., GitHub).
  url: 
    - "https://github.com/project/repo"
  # The specific commit hash or branch name that was audited.
  commit_id: 
    - "a1b2c3d4e5f6..." # The commit id or branch name at the start of the audit, not for the fix after the audit. Match the url(s) one-to-one in both quantity and position.
  # A list of on-chain addresses for the AUDITED version of the contracts. Do NOT include addresses of fixed versions.
  address:
    - "Address1...(if exists, otherwise null)"
  # The blockchain network
  chain: "evm/base/polygon/solana/etc."
  audit_date: "2023-03-22" # Start Date of the Audit

# Explain why you are extracting the following vulnerability findings.
reason_for_vulnerability_findings: |
  The chunk details two security issues, one related to integer overflow and another concerning access control, the first is valid finding while the second is partial, need to be merged with another chunk.

vulnerability_findings:
  - title: "Vulnerability Title" # If the finding seems incomplete because the chunk ends abruptly, append (Part). e.g., "Integer Overflow in Calculation (Part)"
    description: |
      A multi-sentence, detailed text. It MUST cover the following four aspects:
      1.  **Description:** A clear description of what the vulnerability is.
      2.  **Cause:** The root cause of the vulnerability (e.g., improper authorization, lack of input validation, incorrect logic).
      3.  **Exploitation:** The potential method an attacker could use to exploit it.
      4.  **Impact:** The consequences of a successful exploit (e.g., loss of funds, denial of service).
    severity: "Critical" # e.g., Critical, High, Medium, Low, Informational or null. Do not assume a severity if it is not directly provided in the document chunk.
    location:
      # A list of vulnerable locations provided by the document.
      # If a part of the format is not available, omit it but maintain the structure. e.g., 'Filename::Function#line' or 'Function#line' or 'Filename#line'.
      - "StakingPool.sol::deposit#105-110"
      - "RewardsManager.sol::calculate_rewards#25"
```

Instructions:

Analyze the provided document chunk.

If any item of information (e.g., project URL, commit ID, audit date, vulnerability severity, location) is not present in the document chunk, use null value or an empty list `[]` for lists.

If you find the last vulnerability information that seems incomplete because the chunk cuts off mid-sentence or mid-section (e.g., a vulnerability description is clearly cut short), you MUST append (Part) to its corresponding title. This signals to the next step (the Reducer) that it needs to be merged.

Fill in the YAML structure based on your findings.

If no relevant project or vulnerability information is found in the chunk, you should simply reply "n/a".
"""
        ),
        ell.user(
            "Assistant: Yes, I understand. I am Axiom, a smart contract security expert. I will act as a Mapper, extracting information from the document chunk and formatting it strictly as the specified YAML structure, marking any partial findings with (Part)."
        ),
        ell.user(
            f"Document fragment:\n---\n{document}\n---\n\nPlease analyze the document chunk and generate the YAML output based on the rules provided."
        ),
    ]


@ell.simple(model=MODEL, client=CLIENT, temperature=TEMPERATURE)
def invoke_reduce(map_results: str):
    output_example = """```yaml
project_info:
  url: 
    - "https://github.com/project/repo"
  commit_id: 
    - "a1b2c3d4e5f6..." # Match the url(s) one-to-one in both quantity and position.
  address:
    - "Addressxxxx...(if exists, otherwise null) "
  chain: "evm/base/polygon/solana/etc."
  audit_date: "2023-03-22"
findings:
  - id: 0
    title: "Re-entrancy in Withdraw Function"
    description: |
      The withdraw function is vulnerable to a re-entrancy attack. The cause is that the user's balance is updated after the external call to transfer funds. An attacker could create a malicious contract that calls the withdraw function repeatedly before the first call completes, allowing them to drain more funds than their actual balance. A successful exploit would lead to a complete loss of funds in the contract.
    severity: "High"
    location:
      - "RewardsManager.sol::calculate_rewards#25-30"
  - id: 1
    # ... more findings
```"""
    return [
        ell.system(
            "You are Axiom, an AI expert specializing in smart contract security and audit report analysis. Your role is to synthesize and finalize a security report from partial data."
        ),
        ell.user(
            f"""
You are acting as a 'Reducer' in a MapReduce process. You will receive a collection of YAML-formatted data fragments from multiple Mappers. Each fragment is an extraction from a chunk of a single, large audit report.

Your task is to consolidate, deduplicate, and merge these fragments into a single, complete, and final YAML report.

**Instructions:**
1.  **Merge Partial Information:** Pay close attention to any `title` fields ending with `(Part)`. These indicate fragmented findings or project info that MUST be merged. Combine their `description` and `location` lists into a single, coherent entry.
2.  **Deduplicate:** Consolidate all `project_info` sections into one. Remove any duplicate `url`, `commit_id`, and `address` entries. Similarly, if multiple mappers extracted the exact same vulnerability, include it only once in the final report.
3.  **Finalize and Format:** Generate a single, clean YAML output.
    *   The final YAML should have two main keys: `project_info` and `findings`.
    *   The `findings` section should be a list of vulnerability objects, each with a unique sequential `id` starting from 0.
    *   The final output **MUST NOT** contain any `(Part)` tags or the `reason` fields (`reason_for_project_info`, `reason_for_vulnerability_findings`).
4.  **Be Comprehensive:** Ensure all unique information from all mappers is included in the final report. Use `null` or an empty list `[]` for any fields that could not be determined after reviewing all inputs.

**Example of Final Output Format:**
{output_example}
"""
        ),
        ell.user(
            "Assistant: Yes, I understand. I am Axiom. I will act as a Reducer to process the aggregated data from the mappers. I will merge all partial findings, deduplicate all information, and produce a single, final, consolidated YAML report in the specified format."
        ),
        ell.user(
            f"Aggregated data from Mappers:\n---\n{map_results}\n---\n\nPlease process the data and generate the final, consolidated YAML report."
        ),
    ]


@ell.simple(model=MODEL, client=CLIENT, temperature=TEMPERATURE)
def invoke_classify(vuln_info: str, cwe_info: str, last_chosen: list | None = None):

    last_chosen_instruction = ""
    if last_chosen:
        last_chosen_instruction = f"""
IMPORTANT: You are now at a more specific level of the CWE hierarchy. Your previous, broader choice was {", ".join([str(cwe_node)+" ("+cwe_node.Description+")" for cwe_node in last_chosen])}.
If you determine that NONE of the new, more specific candidates below accurately describe the vulnerability, you should select your last choice as the final answer. This means the previous category was the most appropriate level of detail.
To do this, simply output the most suitable one in "{", ".join(["CWE-"+str(cwe_node.ID) for cwe_node in last_chosen])}" as your answer.

For candidate CWEs that are already highly specific or granular, 'look-ahead' analysis is not necessary. Evaluate them based on their direct applicability alone."""

    return [
        ell.system(
            """You are Axiom, an AI expert in vulnerability analysis. Your task is to perform hierarchical Root Cause Analysis on a given vulnerability. You will be guided through a tree of Common Weakness Enumeration (CWE) categories, from broad pillars to specific weaknesses. At each step, your goal is to select the category from the provided list that most accurately describes the vulnerability. Your expertise is crucial for navigating this hierarchy and pinpointing the most precise classification. 
            For each task, you must use a rigorous think step-by-step process that incorporates a 'look-ahead' strategy. Your reasoning should be structured as follows:

1.  **Analyze:** Scrutinize the provided vulnerability (`vuln_info`) to identify its fundamental weakness, distinguishing the cause from its consequences.
2.  **Evaluate & Look Ahead:** Assess each candidate CWE from the `cwe_info` list. Critically, if the candidate CWEs are abstract node, you must anticipate the potential child (or leaf) CWEs of each candidate.
    * **Rule:** If Candidate A is a decent fit, but Candidate B has a child node (a more specific CWE) that perfectly describes the vulnerability, you should prioritize Candidate B. Your goal is to select the path that leads to the most accurate final classification.
3.  **Decide & Format:** Conclude with your final choice, justifying it based on your look-ahead analysis. Then, state the answer in the required format.
"""
        ),
        ell.user(
            """I will provide a vulnerability's title and description. We will work together to map it to the most specific CWE ID possible through a step-by-step, hierarchical process.

<example>
USER:
**Vulnerability Title:** Onboarding location payment and accounting
**Vulnerability Information:**
1. **Description:** The `rewardable_entity_config` is not properly validated, allowing location setting without paying fees when the wrong config type is provided.\n2. **Cause:** The condition checks for `IotConfig` in both `onboard_iot_hotspot_v0` and `onboard_mobile_hotspot_v0`, even when a `MobileConfig` is expected.\n3. **Exploitation:** An attacker could bypass location staking fees by providing a mismatched config type.\n4. **Impact:** Enables free location updates, undermining the economic model and potentially enabling spam or abuse.

**Candidate CWEs for this level:**
`[CWE-284: Improper Access Control, CWE-664: Improper Control of a Resource Through its Lifetime, CWE-707: Improper Neutralization, ...]`

Now, please prepare for this vulnerability. Follow the hierarchical analysis process.


ASSISTANT:
**Thought:**
1.  **Analyze:** The vulnerability is caused by the lack of proper validation for the `rewardable_entity_config` type during onboarding. This is a failure in verifying that the provided data matches expected types, leading to incorrect processing of location payments.

2.  **Evaluate & Look Ahead:**
    * `CWE-284: Improper Access Control`: This is a tempting but incorrect choice. While access control is related, the issue here is not about who can access what, but rather about validating the data being provided. The flaw is not about permissions but about data integrity.
    * `CWE-664: Improper Control of a Resource...`: This is also incorrect. The problem is not about resource lifecycle management, but about ensuring the correct type of configuration is used. This CWE does not lead to a more specific child that fits the scenario.
    * `CWE-707: Improper Neutralization`: This is the strongest candidate. It directly addresses the failure to properly validate and neutralize incorrect input data. Looking ahead, `CWE-20: Improper Input Validation` is a child of `CWE-707` and perfectly describes the failure to validate the config type. Since `CWE-707` leads directly to this precise weakness, it is the correct path to follow.

3.  **'Look-Ahead' Clincher:** CWE-707 leads to CWE-20: Improper Input Validation, which perfectly captures the essence of the vulnerability. This makes `CWE-707` the best choice at this level.

4.  **Decide:** `CWE-707` is the most accurate choice. It correctly frames the problem as a failure of a security mechanism and is the only candidate that leads directly to the precise, underlying weakness of insufficient data authenticity verification.

```
Answer: CWE-707
```
</example>
"""
        ),
        ell.user(
            "Yes, I understand. I am Axiom, and I will analyze the provided vulnerability by navigating the CWE hierarchy to find the most precise root cause."
        ),
        ell.user(
            f"""Let's begin or continue the mapping process.

### Vulnerability to Analyze ###
{vuln_info}
\n\n
---
\n\n
{last_chosen_instruction}

### Candidate CWEs for this level ###
{cwe_info}

First, provide your step-by-step thinking process. Then, provide the final answer on a new line in the format:

```\nAnswer: CWE-XXX```"""
        ),
    ]
