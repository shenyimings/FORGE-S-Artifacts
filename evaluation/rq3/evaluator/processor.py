import os
from pathlib import Path
from typing import List, Dict, Set, Union
from collections import defaultdict

from model import Finding, ProjectInfo, Report, VulnPair, CWEHandler, CWEDatabase


def load_report(report_path: str) -> Report:
    """Load a report from a JSON file."""
    import json

    with open(report_path, "r") as f:
        data = json.load(f)
    return Report.model_validate(data)


def process_reports(
    input_path: Union[str, Path], dataset_path="dataset/"
) -> List[VulnPair]:
    """
    Process reports from a file or directory and generate VulnPairs.
    Aggregates findings based on overlapping files.
    """
    input_path = Path(input_path)
    reports = []

    if input_path.is_file():
        if input_path.suffix == ".json":
            reports.append(load_report(str(input_path)))
    elif input_path.is_dir():
        for file_path in input_path.glob("*.json"):
            reports.append(load_report(str(file_path)))

    vuln_pairs = []

    for report in reports:
        # Group findings by connected components of files
        # Graph where nodes are findings and edges exist if they share a file
        # Or nodes are files and edges exist if they share a finding?
        # User requirement: "finding A in A,B,C; finding B in B,D -> ABCD, {A, B}"
        # This means transitive closure of file overlap.

        # 1. Map each file to the set of findings that touch it
        file_to_findings: Dict[str, Set[int]] = defaultdict(set)
        # Also map finding index to its files for easy access
        finding_files: Dict[int, Set[str]] = {}

        # We need absolute paths for files
        pp = report.project_info.project_path
        # Skip reports without source code (no project_path): they have no
        # localizable files and thus cannot form detection pairs. Such reports
        # exist after merging the no-source findings into dataset/findings.
        if pp is None or (isinstance(pp, str) and pp.strip().lower() in ("", "n/a")):
            continue
        if isinstance(pp, dict):
            project_root = Path(next(iter(pp.values())))
        else:
            project_root = Path(pp)

        valid_findings = []
        for i, finding in enumerate(report.findings):

            # Filter out findings with no files or no category
            if not finding.files:
                continue

            # Resolve file paths
            resolved_files = set()
            for f_rel in finding.files:
                # Sometimes paths might be absolute or relative
                # Assuming relative to project_path as per instruction
                try:
                    abs_path = (Path(dataset_path) / project_root / f_rel).resolve()
                    # get file's extension
                    ext = abs_path.suffix.lower()
                    if ext in {".md", ".pdf.json"}:
                        continue
                    if ext not in {".rs", ".ts", ".sol", ".toml", ".sh", ".json"}:
                        # print(f"Resolved file: {abs_path} with extension {ext}")
                        continue

                    # if not os.path.exists(abs_path):
                    #     print(f"Warning: Resolved file does not exist: {abs_path}")
                    if os.path.exists(abs_path):

                        resolved_files.add(str(abs_path))

                except Exception:
                    print(
                        f"Warning: Could not resolve file path {f_rel} in project {project_root}"
                    )
                    continue

            if not resolved_files:
                continue

            valid_findings.append(finding)
            f_idx = len(valid_findings) - 1
            finding_files[f_idx] = resolved_files

            for f_path in resolved_files:
                file_to_findings[f_path].add(f_idx)

        # 2. Build connected components of findings
        # Two findings are connected if they share a file
        # We can use Union-Find or BFS/DFS

        num_findings = len(valid_findings)
        visited_findings = [False] * num_findings

        for i in range(num_findings):
            # print(f"Finding {i}: visited={visited_findings[i]}")  # Debugging line
            if visited_findings[i]:
                continue

            # Start a new component
            component_findings_indices = set()
            queue = [i]
            visited_findings[i] = True

            while queue:
                # Debugging line
                curr_idx = queue.pop(0)
                component_findings_indices.add(curr_idx)

                # Get all files for this finding
                files = finding_files[curr_idx]

                # For each file, get all other findings that touch it
                for f_path in files:
                    linked_findings = file_to_findings[f_path]
                    for linked_idx in linked_findings:
                        if not visited_findings[linked_idx]:
                            visited_findings[linked_idx] = True
                            queue.append(linked_idx)

            # 3. Construct VulnPair for this component
            comp_files = set()
            comp_ground_truth = []
            comp_vuln_ground_truth = []

            for f_idx in component_findings_indices:

                comp_files.update(finding_files[f_idx])
                # Extract category/CWE info
                # finding.category is a Dict, e.g. {"1": "CWE-123", ...}
                # VulnPair expects List[Dict[str, List[str]]]
                # We need to convert finding.category values to lists if they aren't already
                # Assuming finding.category is Dict[str, str] based on previous context,
                # but user requested List[Dict[str, List]] in VulnPair.
                # Let's adapt.

                finding = valid_findings[f_idx]
                gt_vuln = {"title": finding.title, "description": finding.description}
                gt_item = {}
                for k, v in finding.category.items():
                    # Ensure value is a list
                    if isinstance(v, list):
                        gt_item[k] = v
                    else:
                        gt_item[k] = [str(v)]
                comp_ground_truth.append(gt_item)
                comp_vuln_ground_truth.append(gt_vuln)

            if not comp_files:
                continue

            vp = VulnPair(
                project=project_root.name,
                files=[Path(f) for f in comp_files],
                cwe_ground_truth=comp_ground_truth,
                cwe_detected=[],  # To be filled by reasoner
                vuln_ground_truth=comp_vuln_ground_truth,
            )
            vuln_pairs.append(vp)

    return vuln_pairs


# if __name__ == "__main__":

    # input_path = "solana-security/dataset/findings/squads_v4_audit_final.pdf.json"
    # vuln_pairs = process_reports(input_path)
    # print(f"Processed {len(vuln_pairs)} VulnPairs from {input_path}")
    # for i, vp in enumerate(vuln_pairs):
    #     print(f"VulnPair {i}:")
    #     print(f"  Project: {vp.project}")
    #     print(f"  Files: {[str(f) for f in vp.files]}")
    #     print(f"  Ground Truth: {vp.cwe_ground_truth}")
    #     print()
    #     vp.cwe_detected = [{"1": ["CWE-682"], "2": ["CWE-1339"]}]
    #     print(f"  Detected: {vp.cwe_detected}")
    #     vp.judge_detection(
    #         CWEHandler.load_from_file("evaluation/rq3/auditor/cwe_dict.json")
    #     )
    #     print(f"  Detection Judged.")
    #     print(vp.tfnp_level, vp.prf_level)
    #     print(vp.tfnp_cwe, vp.prf_cwe)

