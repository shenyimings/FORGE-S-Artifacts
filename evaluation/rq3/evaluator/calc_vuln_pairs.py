# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic>=2.0"]
# ///
"""Count the detection (vuln) pairs constructed from the current dataset.

Paths are anchored to the repository layout via __file__ so the script is
independent of the working directory. Reports without source code (no
project_path) are skipped by process_reports().
"""
from pathlib import Path

from processor import process_reports

# evaluator/ -> rq3/ -> evaluation/ -> solana-security/
REPO_ROOT = Path(__file__).resolve().parents[3]
FINDINGS_DIR = REPO_ROOT / "dataset" / "findings"
DATASET_PATH = str(REPO_ROOT / "dataset") + "/"

if __name__ == "__main__":
    total_vuln_pairs = 0
    projects_with_pairs = 0
    for json_path in sorted(FINDINGS_DIR.glob("*.json")):
        vuln_pairs = process_reports(json_path, dataset_path=DATASET_PATH)
        if vuln_pairs:
            projects_with_pairs += 1
        total_vuln_pairs += len(vuln_pairs)

    print(f"Reports scanned:            {len(list(FINDINGS_DIR.glob('*.json')))}")
    print(f"Reports yielding >=1 pair:  {projects_with_pairs}")
    print(f"Generated {total_vuln_pairs} VulnPairs (detection pairs)")
