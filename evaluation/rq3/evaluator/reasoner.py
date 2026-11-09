import yaml
import time
from typing import List, Dict, Set, Optional, Tuple
from pathlib import Path
from model import (
    CWEHandler,
    VulnPair,
    DetectionResult,
    DetectedCWE,
    CWE,
)
from call_llm import call_llm, create_messages


class HierarchicalAuditor:
    def __init__(self, cwe_handler: CWEHandler, model_name: str = "gemini3"):
        self.handler = cwe_handler
        self.model_name = model_name
        # The 10 Pillar CWEs as specified
        self.pillar_ids = [284, 435, 664, 682, 691, 693, 697, 703, 707, 710]

    def audit(self, vuln_pair: VulnPair) -> Tuple[VulnPair, Dict[str, int]]:
        """
        Perform hierarchical vulnerability detection on the files in vuln_pair.
        """
        # Prepare source code context
        source_code_context = self._prepare_source_code(vuln_pair.files)

        total_usage = {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}

        # Step 1: Pillars
        candidates = [
            self.handler.get_cwe(pid)
            for pid in self.pillar_ids
            if self.handler.get_cwe(pid)
        ]

        # print(f"\n--- Level 1 (Pillars) Detection ---")
        # print(f"Candidates: {[f'CWE-{c.ID}' for c in candidates]}")

        # Extract GT for level 1
        gt_at_level_1 = set()
        for item in vuln_pair.cwe_ground_truth:
            if "1" in item:
                gt_at_level_1.update(item["1"])
        # print(f"GT at Level 1: {list(gt_at_level_1)}")

        llm_results = []
        vuln_pair.candidate_cwes.append([f"CWE-{c.ID}" for c in candidates])
        detected_pillars, usage, res = self._detect_cwes(
            source_code_context, candidates
        )
        llm_results.append(res)

        # print(f"LLM Detected: {[f'CWE-{c.ID}' for c in detected_pillars]}")

        # Accumulate usage
        for k in total_usage:
            total_usage[k] += usage.get(k, 0)

        # Initialize active chains with detected pillars
        # active_chains is a list of lists of CWE objects: [[CWE-A], [CWE-B]]
        active_chains = []
        final_chains = []
        for cwe in detected_pillars:
            if f"CWE-{cwe.ID}" not in gt_at_level_1:
                # print(f"Note: Detected pillar CWE-{cwe.ID} not in GT at Level 1")
                final_chains.append([cwe])
                continue
            active_chains.append([cwe])

        # active_chains = [[cwe] for cwe in detected_pillars]

        while active_chains:
            # Gather candidates for the next level (children of the last node in each chain)
            next_level_candidates = set()
            for chain in active_chains:
                last_cwe = chain[-1]
                children = self.handler.get_direct_children(last_cwe.ID)
                for child in children:
                    next_level_candidates.add(child)

            # If no candidates for next level, all current chains are final
            if not next_level_candidates:
                final_chains.extend(active_chains)
                break

            # Debug: Print candidates and GT for this level
            current_level = len(active_chains[0]) + 1
            # print(f"\n--- Level {current_level} Detection ---")
            # print(f"Candidates: {[f'CWE-{c.ID}' for c in next_level_candidates]}")

            # Extract GT for this level for debugging
            gt_at_level = set()
            for item in vuln_pair.cwe_ground_truth:
                if str(current_level) in item:
                    gt_at_level.update(item[str(current_level)])
            # print(f"GT at Level {current_level}: {list(gt_at_level)}")

            # Detect among candidates
            if isinstance(vuln_pair.candidate_cwes, list):
                vuln_pair.candidate_cwes.append(
                    [f"CWE-{c.ID}" for c in next_level_candidates]
                )
            detected_children, usage, res = self._detect_cwes(
                source_code_context, list(next_level_candidates)
            )
            llm_results.append(res)

            # print(f"LLM Detected: {[f'CWE-{c.ID}' for c in detected_children]}")

            # Accumulate usage
            for k in total_usage:
                total_usage[k] += usage.get(k, 0)

            # Extend chains
            new_active_chains = []
            detected_children_set = set(detected_children)

            # Prepare GT set for this level
            current_level = len(active_chains[0]) + 1
            gt_next_level_ids = set()
            for item in vuln_pair.cwe_ground_truth:
                if str(current_level) in item:
                    gt_next_level_ids.update(item[str(current_level)])
            # print(f"GT IDs at Level {current_level}: {list(gt_next_level_ids)}")
            # Track which chains were extended
            for chain in active_chains:
                last_cwe = chain[-1]
                children = self.handler.get_direct_children(last_cwe.ID)

                # Find which of these children were detected
                extensions = [
                    child for child in children if child in detected_children_set
                ]

                # Filter extensions based on GT
                valid_exts = []
                invalid_exts = []
                for ext in extensions:
                    if f"CWE-{ext.ID}" in gt_next_level_ids:
                        valid_exts.append(ext)
                    else:
                        invalid_exts.append(ext)

                if valid_exts:
                    # Branch out for each valid child
                    for ext in valid_exts:
                        new_chain = chain + [ext]
                        new_active_chains.append(new_chain)

                # For invalid extensions, we track them as final results so they count as False Positives
                for ext in invalid_exts:
                    final_chains.append(chain + [ext])

                # Only if the LLM detected nothing at this level do we treat the parent chain as the final result
                if not extensions:
                    final_chains.append(chain)

            active_chains = new_active_chains
            # print(
            #     f"Current final chains: {[ [f'CWE-{c.ID}' for c in chain] for chain in final_chains ]}"
            # )

        # Format final_chains into List[Dict[str, List[str]]]
        formatted_chains = []
        for chain in final_chains:
            chain_dict = {}
            for i, cwe in enumerate(chain):
                level = str(i + 1)
                chain_dict[level] = [f"CWE-{cwe.ID}"]
            formatted_chains.append(chain_dict)
        vuln_pair.results = llm_results

        vuln_pair.cwe_detected = formatted_chains
        return vuln_pair, total_usage

    def _prepare_source_code(self, files: List[Path]) -> str:
        context = ""
        for file_path in files:
            try:
                with open(file_path, "r") as f:
                    content = f.read()
                    context += f"File: {file_path.name}\n```rust\n{content}\n```\n\n"
            except Exception as e:
                print(f"Error reading file {file_path}: {e}")
        return context

    def _detect_cwes(
        self, source_code: str, candidates: List[CWE]
    ) -> Tuple[List[CWE], Dict[str, int], DetectionResult]:
        if not candidates:
            return [], {}, DetectionResult(reasoning="No candidates", findings=[])

        # Construct Prompt
        candidate_text = "\n".join(
            [f"- CWE-{c.ID}: {c.Name} ({c.Description})" for c in candidates]
        )

        # Get Schema
        schema = DetectionResult.get_yaml_schema()

        system_prompt = (
            "You are an expert Solana smart contract security auditor.\n"
            "Your task is to analyze the provided Rust source code to identify potential security issues.\n"
            "You are provided with a list of specific CWEs. Treat these CWEs as the **root causes** of potential vulnerabilities/issues.\n"
            "From the perspective of each of these root causes, explore the code to determine if any security issues exist that stem from them.\n"
            "Be thorough: the code may contain multiple vulnerabilities / issues arising from different root causes, or multiple instances of the same root cause.\n"
            "You must only consider the following list of CWE candidates:\n"
            f"{candidate_text}\n\n"
            "Analyze the code carefully. If you find any security issues arising from these root causes, report them.\n"
            "If a specific CWE type is present, report it. If no issues are found, explicitly state an empty finding field. You can report multiple instances for the same CWE type if applicable.\n"
            "Output your findings in YAML format strictly following this schema:\n"
            f"{schema}\n"
        )

        user_prompt = f"Here is the source code:\n\n{source_code}"

        messages = create_messages(("system", system_prompt), ("user", user_prompt))

        accumulated_usage = {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0,
        }
        max_retries = 3

        for attempt in range(max_retries):
            response_content = ""
            try:
                response_content, prompt_tokens, completion_tokens, total_tokens = (
                    call_llm(messages, model=self.model_name)
                )

                accumulated_usage["prompt_tokens"] += prompt_tokens
                accumulated_usage["completion_tokens"] += completion_tokens
                accumulated_usage["total_tokens"] += total_tokens

                # print(f"LLM Response:\n{response_content}")
                # Robustly extract YAML block even if it's not at the start
                cleaned_content = response_content
                yaml_block = None

                try:
                    # Try to find the content between ```yaml ... ```
                    parts = cleaned_content.split("```yaml", 1)
                    if len(parts) > 1:
                        after_yaml = parts[1]
                        yaml_block = after_yaml.split("```", 1)[0].strip()
                except Exception:
                    yaml_block = None

                if not yaml_block:
                    # Fallback: previous cleaning logic
                    cleaned_content = cleaned_content.strip()
                    if cleaned_content.startswith("```yaml"):
                        cleaned_content = cleaned_content[7:]
                    if cleaned_content.startswith("```"):
                        cleaned_content = cleaned_content[3:]
                    if cleaned_content.endswith("```"):
                        cleaned_content = cleaned_content[:-3]
                    cleaned_content = cleaned_content.replace("**", "")
                    yaml_block = cleaned_content

                data = yaml.safe_load(yaml_block) or {}

                # If the YAML wraps content under tool_args, unwrap it
                if isinstance(data, dict) and "tool_args" in data:
                    data = data.get("tool_args", {})

                # Validate with Pydantic
                result = DetectionResult.model_validate(data)

                detected_cwes = []
                for det in result.findings:
                    # Find the CWE object
                    # det.category might be "CWE-284" or just "284"
                    cwe_id_str = det.category.upper()

                    # Match against candidates
                    for cand in candidates:
                        if (
                            f"CWE-{cand.ID}" == cwe_id_str
                            or str(cand.ID) == cwe_id_str
                            or cwe_id_str.endswith(f"-{cand.ID}")
                        ):
                            detected_cwes.append(cand)
                            break

                return detected_cwes, accumulated_usage, result

            except Exception as e:
                print(
                    f"Error in LLM call or parsing (Attempt {attempt + 1}/{max_retries}): {e}"
                )
                # print(f"LLM Response was:\n{response_content}")
                time.sleep(1)  # brief pause before retry
                if attempt == max_retries - 1:
                    return (
                        [],
                        accumulated_usage,
                        DetectionResult(reasoning=response_content, findings=[]),
                    )

        return [], accumulated_usage, DetectionResult(reasoning="", findings=[])
