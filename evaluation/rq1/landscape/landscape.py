# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pydantic>=2.0",
#     "pandas>=2.0",
#     "plotly>=5.0",
#     "kaleido",
# ]
# ///
"""RQ2: Security landscape — CWE distribution, severity, treemap."""

from __future__ import annotations

import json
import os
from collections import Counter, namedtuple
from dataclasses import dataclass, field
from typing import Dict, List, Literal, Optional, Union

import plotly.graph_objects as go
from pydantic import BaseModel, Field, RootModel
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
DATASET_DIR = SCRIPT_DIR.parent.parent.parent / "dataset"
CWE_DICT_PATH = SCRIPT_DIR.parent.parent / "rq3" / "evaluator" / "cwe_dict.json"


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
            for f in files:
                if f.endswith(".json"):
                    json_files.append(os.path.join(root, f))
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

    def operate_reduce(self, results: List, result_type):
        res = {}
        for f in result_type._fields:
            res[f] = []
        for result in results:
            for f in result_type._fields:
                res[f].extend(getattr(result, f))
        return res

    def process_files(self, analysis_func=None, modify_func=None) -> List:
        results = []
        json_files = self._get_all_json_files()
        for json_file in json_files:
            with open(json_file, "r", encoding="utf8") as f:
                data = json.load(f)
            report = Report(**data)
            if analysis_func:
                result = analysis_func(report)
                results.append(result)
            if modify_func:
                modified_report: Report = modify_func(report)
                with open(json_file, "w", encoding="utf8") as f:
                    f.write(
                        modified_report.model_dump_json(
                            indent=4, exclude={"project_info": {"compiler_version"}}
                        )
                    )
        return results


# ── CWE database ─────────────────────────────────────────────────────────────


class CWEEntry(BaseModel):
    ID: int
    Name: str
    Description: str = ""
    Abstraction: Literal["Pillar", "Class", "Base", "Variant", "Compound"]
    Mapping: Literal["Allowed", "Allowed-with-Review", "Discouraged", "Prohibited"]
    Peer: List = Field(default_factory=list)
    Parent: List = Field(default_factory=list)
    Child: List[int] = Field(default_factory=list)

    def __str__(self) -> str:
        return f"CWE-{self.ID}: {self.Name}"

    def __hash__(self):
        return hash(str(self))

    def add_child(self, child_cwe: "CWEEntry"):
        self.Child.append(child_cwe)
        child_cwe.Parent.append(self)


class CWEDatabase(RootModel):
    root: Dict[str, CWEEntry]

    def get_by_id(self, id: int | str):
        return self.root[f"CWE-{id}"]

    def get_by_name(self, name: str):
        return self.root[name]


severity_score_map = {
    "informational": (0, 0, 0),
    "low": (0.1, 3.9, 2),
    "medium": (4.0, 6.9, 5.45),
    "high": (7.0, 8.9, 7.95),
    "critical": (9.0, 10.0, 9.5),
}


class CWEHandler:
    def __init__(self, cwe_database: CWEDatabase):
        self.db = cwe_database
        self.setup_relationships()

    @classmethod
    def load_from_file(cls, file_path: str) -> "CWEHandler":
        with open(file_path, "r") as f:
            data = json.load(f)
        db = CWEDatabase.model_validate(data)
        return cls(db)

    def setup_relationships(self):
        for cwe in self.db.root.values():
            cleaned_parents = []
            for p in cwe.Parent:
                if isinstance(p, CWEEntry):
                    cleaned_parents.append(p)
                elif isinstance(p, int):
                    try:
                        cleaned_parents.append(self.db.get_by_id(p))
                    except KeyError:
                        pass
            cwe.Parent = cleaned_parents

        for cwe_key, cwe in self.db.root.items():
            for child_id in cwe.Child:
                try:
                    child_cwe = self.db.get_by_id(child_id)
                    if cwe not in child_cwe.Parent:
                        child_cwe.Parent.append(cwe)
                except KeyError:
                    continue

    def get_cwe(self, cwe_id: Union[int, str]) -> Optional[CWEEntry]:
        try:
            if isinstance(cwe_id, int):
                return self.db.get_by_id(cwe_id)
            elif isinstance(cwe_id, str):
                if cwe_id.startswith("CWE-"):
                    return self.db.root[cwe_id]
                else:
                    return self.db.get_by_id(int(cwe_id))
        except (KeyError, ValueError):
            return None

    def get_all_descendants(self, cwe_id: Union[int, str]) -> List[CWEEntry]:
        cwe = self.get_cwe(cwe_id)
        if not cwe:
            return []
        descendants = set()
        queue = [cwe]
        while queue:
            current = queue.pop(0)
            for child_id in current.Child:
                child = self.get_cwe(child_id)
                if child and child not in descendants:
                    descendants.add(child)
                    queue.append(child)
        return list(descendants)

    def get_root_parents(self, cwe_id: Union[int, str]) -> List[CWEEntry]:
        cwe = self.get_cwe(cwe_id)
        if not cwe:
            return []
        roots = set()
        queue = [cwe]
        visited = set()
        while queue:
            current = queue.pop(0)
            if current in visited:
                continue
            visited.add(current)
            if current.Abstraction == "Pillar":
                roots.add(current)
                continue
            if not current.Parent:
                roots.add(current)
                continue
            for parent in current.Parent:
                queue.append(parent)
        return list(roots)


# ── CWE-Id statistics ────────────────────────────────────────────────────────


def cwe_statistics(cwe_handler: Optional[CWEHandler]):
    CWE_tuple = namedtuple("CWE", ["cwe_ids"])

    def extract_last_cwe_ids(report: Report):
        cwe_ids = []
        for finding in report.findings:
            if finding.category:
                try:
                    max_key = max(
                        [int(k) for k in finding.category.keys() if k.isdigit()]
                    )
                    cwe_value = finding.category.get(str(max_key), [""])[0]
                    if isinstance(cwe_value, str) and cwe_value.startswith("CWE-"):
                        cwe_ids.append(cwe_value)
                except (ValueError, AttributeError):
                    continue
        return CWE_tuple(cwe_ids=cwe_ids)

    def extract_pillar_cwe_ids(report: Report):
        cwe_ids = []
        for finding in report.findings:
            if finding.category:
                try:
                    max_key = max(
                        [int(k) for k in finding.category.keys() if k.isdigit()]
                    )
                    cwe_value = finding.category.get(str(max_key), [""])[0]
                    if isinstance(cwe_value, str) and cwe_value.startswith("CWE-"):
                        if cwe_handler:
                            pillars = cwe_handler.get_root_parents(cwe_value)
                            for pillar in pillars:
                                cwe_ids.append(f"CWE-{pillar.ID}")
                        else:
                            cwe_value = finding.category.get("1", [""])[0]
                            if isinstance(cwe_value, str) and cwe_value.startswith(
                                "CWE-"
                            ):
                                cwe_ids.append(cwe_value)
                except (ValueError, AttributeError):
                    continue
        return CWE_tuple(cwe_ids=cwe_ids)

    processor = JSONFileProcessor(str(DATASET_DIR / "findings"))
    results = processor.process_files(analysis_func=extract_last_cwe_ids)
    results_pillar = processor.process_files(analysis_func=extract_pillar_cwe_ids)

    cwe_pillar_result = processor.operate_reduce(results_pillar, CWE_tuple)
    cwe_result = processor.operate_reduce(results, CWE_tuple)

    unique_cwe_ids = set(cwe_result["cwe_ids"])
    cwe_counter = Counter(cwe_result["cwe_ids"])

    print(f"Total unique CWE IDs: {len(unique_cwe_ids)}")
    print("Top 10 CWE IDs by occurrence:")
    for cwe_id, count in cwe_counter.most_common(10):
        print(f"{cwe_id}: {count}")

    print(f"\nTotal unique Pillar CWE IDs: {len(set(cwe_pillar_result['cwe_ids']))}")
    print("Top 10 Pillar CWE IDs by occurrence:")
    cwe_pillar_counter = Counter(cwe_pillar_result["cwe_ids"])
    for cwe_id, count in cwe_pillar_counter.most_common(10):
        print(f"{cwe_id}: {count}")


# ── Severity analysis (with and without informational) ───────────────────────


def severity_analysis(
    cwe_handler: CWEHandler,
    cwe_database: CWEDatabase,
    exclude_informational: bool = False,
):
    label = "(excl. informational)" if exclude_informational else "(all)"
    print(f"\n{'='*60}")
    print(f"SEVERITY ANALYSIS {label}")
    print(f"{'='*60}")

    result_tuple = namedtuple("CWE", ["cwe_name", "severity_score"])
    no_severity_count = {}

    def count_cwe(report: Report):
        findings = report.findings
        cwe_names = []
        severity_score = []
        for finding in findings:
            if not finding.category:
                continue
            severity_text = (finding.severity or "").lower().strip()
            if exclude_informational and severity_text == "informational":
                continue

            final_key = max(finding.category.keys())
            cwe_name_list = finding.category.get(final_key, None)
            try:
                cwe_list = []
                if cwe_name_list:
                    for name in cwe_name_list:
                        try:
                            obj = cwe_database.get_by_name(name)
                            cwe_list.append(obj.ID)
                        except KeyError:
                            pass
                cwe_list = cwe_list[0:1]
            except Exception as e:
                print(f"Error finding {finding.id} in {report.path}: {e}")
                continue

            if not cwe_list:
                continue
            cwe_id = cwe_list[0]
            cwe_names.extend(cwe_list)

            if not finding.severity or severity_text == "n/a":
                no_severity_count[cwe_id] = no_severity_count.get(cwe_id, 0) + 1
            else:
                s_map = severity_score_map.get(severity_text, (0, 0, 0))
                severity_score.append((cwe_id, s_map[2]))

        return result_tuple(cwe_name=cwe_names, severity_score=severity_score)

    processor = JSONFileProcessor(str(DATASET_DIR / "findings"))
    results = processor.process_files(analysis_func=count_cwe)
    res = processor.operate_reduce(results, result_tuple)

    cwe_raw_stats = {}
    for cwe_id, score in res["severity_score"]:
        if cwe_id not in cwe_raw_stats:
            cwe_raw_stats[cwe_id] = {"sum_score": 0.0, "count_rated": 0}
        cwe_raw_stats[cwe_id]["sum_score"] += score
        cwe_raw_stats[cwe_id]["count_rated"] += 1

    cwe_count = Counter(res["cwe_name"])
    cwe_severity = {}
    for cwe_id, stats in cwe_raw_stats.items():
        if stats["count_rated"] > 0:
            cwe_severity[cwe_id] = stats["sum_score"] / stats["count_rated"]
        else:
            cwe_severity[cwe_id] = 0

    cwe_count = dict(sorted(cwe_count.items(), key=lambda x: x[1], reverse=True))

    print(f"\nTop 10 CWEs by Average Severity Score:")
    original_severity_sorted = sorted(
        cwe_severity.items(), key=lambda x: x[1], reverse=True
    )
    severity_rank = 0
    for cwe_id, sev_score in original_severity_sorted:
        if severity_rank >= 10:
            break
        if cwe_count[cwe_id] > 9:
            cwe_info = cwe_database.get_by_id(cwe_id)
            print(
                f"CWE-{cwe_id}: {cwe_info.Name}, {cwe_count[cwe_id]}, {sev_score:.3f}"
            )
            severity_rank += 1

    print(f"\nTop 10 Pillar CWEs by Average Severity Score (DAG-aware):")
    pillars = [284, 435, 664, 682, 691, 693, 697, 703, 707, 710]
    pillar_stats = []
    for pillar_id in pillars:
        descendants = cwe_handler.get_all_descendants(pillar_id)
        pillar_cwe = cwe_handler.get_cwe(pillar_id)
        relevant_cwes = [pillar_cwe] + descendants if pillar_cwe else descendants

        total_occurrence = 0
        total_weighted_sum = 0
        total_rated = 0
        for cwe in relevant_cwes:
            cid = cwe.ID
            total_occurrence += cwe_count.get(cid, 0)
            if cid in cwe_raw_stats:
                stats = cwe_raw_stats[cid]
                total_weighted_sum += stats["sum_score"]
                total_rated += stats["count_rated"]

        avg_sev = total_weighted_sum / total_rated if total_rated > 0 else 0
        pillar_stats.append((pillar_id, avg_sev, total_occurrence))

    pillar_stats.sort(key=lambda x: x[1], reverse=True)
    for pillar_id, avg_sev, total_count in pillar_stats:
        if avg_sev > 0:
            cwe_info = cwe_database.get_by_id(pillar_id)
            print(
                f"CWE-{pillar_id}: {cwe_info.Name}, {total_count} occurrences, {avg_sev:.3f} avg severity"
            )

    return cwe_count, cwe_severity, cwe_raw_stats


# ── Treemap visualization ────────────────────────────────────────────────────


def build_treemap(
    cwe_handler: CWEHandler,
    cwe_database: CWEDatabase,
    cwe_count_orig,
    cwe_severity,
    cwe_raw_stats,
):
    print("\n=== Building Treemap ===")

    cwe_count = cwe_count_orig.copy()
    cwe_count[284] = 0
    cwe_count[707] = 0

    def get_parents(cwe_id):
        try:
            cwe_obj = cwe_database.get_by_id(cwe_id)
            params = cwe_obj.Parent
            if not params:
                return ""
            for p in params:
                p_id = p.ID
                if p_id in cwe_count:
                    return p_id
            first_p = params[0]
            return get_parents(first_p.ID)
        except Exception:
            return ""

    parents = []
    for k, v in cwe_count.items():
        parent = get_parents(k)
        parents.append(f"CWE-{parent}" if parent != "" else "")

    # Duplicate loop from notebook (builds parents twice; kept for consistency)
    for k, v in cwe_count.items():
        parent = get_parents(k)
        parents.append(f"CWE-{parent}" if parent != "" else "")

    values = [v for k, v in cwe_count.items()]
    labels = [f"CWE-{cwe_database.get_by_id(k).ID}" for k, v in cwe_count.items()]
    colors = [
        cwe_severity.get(cwe_database.get_by_name(labels[i]).ID, 0) ** 2
        for i in range(len(labels))
    ]

    values.insert(0, 0)
    labels.insert(0, "")
    colors.insert(0, 0)
    parents.insert(0, "")

    # Recompute colors using DAG-aware severity
    new_colors = colors.copy()
    for i, label in enumerate(labels):
        if label == "":
            continue
        try:
            cwe_id = int(label.split("-")[1])
        except (IndexError, ValueError):
            continue

        relevant_cwes = list(cwe_handler.get_all_descendants(cwe_id))
        relevant_cwes.append(cwe_database.get_by_id(cwe_id))

        total_weighted_sum = 0
        total_rated = 0
        processed = set()
        for cwe_obj in relevant_cwes:
            if not cwe_obj:
                continue
            cid = cwe_obj.ID
            if cid in processed:
                continue
            processed.add(cid)
            if cid in cwe_raw_stats:
                stats = cwe_raw_stats[cid]
                total_weighted_sum += stats["sum_score"]
                total_rated += stats["count_rated"]

        if total_rated > 0:
            avg_sev = total_weighted_sum / total_rated
            new_colors[i] = avg_sev**1.2
        else:
            new_colors[i] = 0

    colors = new_colors
    max_color = max(colors) if max(colors) > 0 else 1
    colors = [min((c / max_color) * 2, 1) for c in colors]

    titles = []
    for i in range(len(labels)):
        if labels[i] == "":
            titles.append("")
        else:
            titles.append(f"{labels[i]} {cwe_database.get_by_name(labels[i]).Name}")

    print(f"Treemap data prepared:")
    print(f"Values (first 10): {values[:10]}")
    print(f"Labels (first 10): {labels[:10]}")
    print(f"Parents (first 10): {parents[:10]}")

    palettes = ["#FCF7F3", "#F8EDE3", "#DFD3C3", "#D0B8A8", "#C5705D", "#8D493A"]
    colors[0] = 0

    fig = go.Figure(
        go.Treemap(
            labels=labels,
            values=values,
            parents=parents,
            maxdepth=2,
            marker=dict(colors=colors, colorscale=palettes),
            textinfo="label+percent entry",
            hovertext=titles,
            hoverinfo="text+percent entry",
            # textfont=dict(size=10),
            pathbar=dict(visible=False),
        )
    )
    fig.update_layout(
        margin=dict(t=0, l=0, r=0, b=10),
        # uniformtext=dict(minsize=8, mode="hide"),
        font=dict(size=10),
        # pathbar_visible=False,
    )

    out = SCRIPT_DIR / "cwe_treemap.html"
    fig.write_html(str(out))
    try:
        fig.write_image(str(SCRIPT_DIR / "cwe_treemap.pdf"), width=480, height=360)
    except Exception as e:
        print(f"PDF export skipped: {e}")
    print(f"Saved {out}")


# ── Main ─────────────────────────────────────────────────────────────────────


def main():
    print("Loading CWE database...")
    cwe_handler = CWEHandler.load_from_file(str(CWE_DICT_PATH))
    cwe_database = cwe_handler.db

    cwe_statistics(cwe_handler)

    cwe_count, cwe_severity, cwe_raw_stats = severity_analysis(
        cwe_handler, cwe_database, exclude_informational=False
    )
    severity_analysis(cwe_handler, cwe_database, exclude_informational=True)

    build_treemap(cwe_handler, cwe_database, cwe_count, cwe_severity, cwe_raw_stats)


if __name__ == "__main__":
    main()
