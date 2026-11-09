# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pydantic>=2.0",
# ]
# ///
"""RQ1: Dataset statistics — findings, projects, contracts, severity."""

from __future__ import annotations

import json
import os
import re
from collections import Counter, namedtuple
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Union

from pydantic import BaseModel

SCRIPT_DIR = Path(__file__).parent
DATASET_DIR = SCRIPT_DIR.parent.parent.parent / "dataset"


# ── Shared data models ───────────────────────────────────────────────────────

@dataclass
class ProjectInfo:
    url: Union[str, int, List, None] = "n/a"
    commit_id: Union[str, int, List, None] = "n/a"
    address: Union[str, int, List, None] = "n/a"
    chain: Union[str, int, List, None] = "n/a"
    compiler_version: Union[str, List, None] = "n/a"
    audit_date: Union[str, int, List, None] = "n/a"
    project_path: Union[str, List, Dict, None] = "n/a"

    def is_empty(self):
        if (self.url == "n/a" and self.address == "n/a") or (
            not self.url and not self.address
        ):
            return True
        return False


@dataclass
class Finding:
    id: Union[str, int] = 0
    category: Dict = field(default_factory=dict)
    title: str = ""
    description: str = ""
    severity: Optional[str] = ""
    location: Union[str, int, List] = ""


class Report(BaseModel):
    path: str = ""
    project_info: ProjectInfo = field(default_factory=ProjectInfo)
    findings: List[Finding] = field(default_factory=list)


class JSONFileProcessor:
    def __init__(self, directory: str):
        self.directory = directory
        self.file_count = 0

    def _get_all_json_files(self) -> List[str]:
        json_files = []
        for root, _, files in os.walk(self.directory):
            for file in files:
                if file.endswith(".json"):
                    json_files.append(os.path.join(root, file))
                    self.file_count += 1
        return json_files

    def operate_add(self, results: List, result_type):
        res = {}
        for f in result_type._fields:
            if isinstance(getattr(results[0], f), int):
                res[f] = 0
            else:
                res[f] = []
        for result in results:
            for f in result_type._fields:
                res[f] += getattr(result, f)
        return res

    def process_files(self, analysis_func=None) -> List:
        results = []
        json_files = self._get_all_json_files()
        for json_file in json_files:
            with open(json_file, "r", encoding="utf8") as f:
                data = json.load(f)
            report = Report(**data)
            if analysis_func:
                result = analysis_func(report)
                results.append(result)
        return results


# ── Count findings & projects ────────────────────────────────────────────────

def count_findings():
    FindingResult = namedtuple("FindingResult", ["total_files", "total_projects", "total_findings"])

    def count_finding(report: Report):
        if isinstance(report.project_info.url, list):
            project_count = len(report.project_info.url)
        else:
            project_count = 1
        return FindingResult(
            total_findings=len(report.findings),
            total_files=1,
            total_projects=project_count,
        )

    processor = JSONFileProcessor(str(DATASET_DIR / "findings"))
    results = processor.process_files(analysis_func=count_finding)
    res = processor.operate_add(results, FindingResult)
    print("=== Findings & Projects ===")
    print(res)
    return res


# ── Check & count contracts ──────────────────────────────────────────────────

def count_contracts():
    Result = namedtuple(
        "Result",
        ["total_files", "total_projects", "valid_projects", "rust_files", "lines_of_code"],
    )

    def count_lines(filepath: Path):
        try:
            with open(filepath, "r", encoding="utf8") as rust_file:
                return len(rust_file.readlines())
        except Exception as e:
            print(f"Error opening file {filepath}: {e}")
            return 0

    def check_projects(report: Report):
        if report.project_info.is_empty():
            print(f"Empty project info in {report.path}")
            return Result(total_files=1, total_projects=0, valid_projects=0, rust_files=0, lines_of_code=0)

        valid_projects = len(report.project_info.url) if isinstance(report.project_info.url, list) else 1
        rust_files = 0
        lines_of_code = 0

        v = report.project_info.project_path
        if v and isinstance(v, str):
            total_projects = len(report.project_info.url) if isinstance(report.project_info.url, list) else 1
            project_path = DATASET_DIR / v.lstrip("/")
            if project_path.exists():
                rust_file_paths = [p for p in project_path.glob("**/*.rs") if p.is_file()]
                rust_files += len(rust_file_paths)
                for path in rust_file_paths:
                    lines_of_code += count_lines(path)
        else:
            total_projects = 0

        return Result(
            total_files=1,
            total_projects=total_projects,
            valid_projects=valid_projects,
            rust_files=rust_files,
            lines_of_code=lines_of_code,
        )

    processor = JSONFileProcessor(str(DATASET_DIR / "findings"))
    results = processor.process_files(analysis_func=check_projects)
    res = processor.operate_add(results, Result)

    print("\n=== Contracts ===")
    print(f"Total files: {res['total_files']}")
    print(f"Total projects: {res['total_projects']}")
    print(f"Valid projects: {res['valid_projects']}")
    print(f"Total rust files: {res['rust_files']}")
    print(f"Total lines of code: {res['lines_of_code']}")
    if res["valid_projects"] > 0:
        print(f"Average lines of code: {res['lines_of_code'] / res['valid_projects']:.2f}")
        print(f"Average files per project: {res['rust_files'] / res['valid_projects']:.2f}")
    return res


# ── Severity statistics ──────────────────────────────────────────────────────

def count_severity_stats():
    severity_category = ["na", "informational", "low", "medium", "high", "critical"]
    SeverityResult = namedtuple("SeverityResult", severity_category)

    def count_severity(report: Report):
        severity_dict = {s: 0 for s in severity_category}
        for finding in report.findings:
            severity = finding.severity
            if severity:
                severity = finding.severity.lower().strip()
            if severity is None or severity == "" or severity == "n/a":
                severity = "na"
            if severity not in severity_category:
                print(f"Unknown severity: {severity}")
                continue
            severity_dict[severity] += 1
        return SeverityResult(**severity_dict)

    processor = JSONFileProcessor(str(DATASET_DIR / "findings"))
    results = processor.process_files(analysis_func=count_severity)
    res = processor.operate_add(results, SeverityResult)
    print("\n=== Severity Statistics ===")
    print(res)
    return res


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    count_findings()
    count_contracts()
    count_severity_stats()


if __name__ == "__main__":
    main()
