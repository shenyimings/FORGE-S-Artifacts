# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pydantic>=2.0",
#     "pandas>=2.0",
#     "matplotlib>=3.8",
#     "seaborn>=0.13",
#     "numpy>=1.24",
#     "cvss>=3.2",
# ]
# ///
"""RQ3: Solana vs EVM — severity & frequency comparison (violin + bar)."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from collections import Counter, namedtuple
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Literal, Optional, Union

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from pydantic import BaseModel, Field, RootModel

SCRIPT_DIR = Path(__file__).parent
DATASET_DIR = SCRIPT_DIR.parent.parent / "dataset"
CWE_DICT_PATH = SCRIPT_DIR.parent / "rq3" / "evaluator" / "cwe_dict.json"
FORGE_REPO_URL = "https://github.com/shenyimings/FORGE-Artifacts"
FORGE_DIR = SCRIPT_DIR / "FORGE-Artifacts"

plt.switch_backend("Agg")


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


class CWEDatabase(RootModel):
    root: Dict[str, CWEEntry]

    def get_by_id(self, id: int | str):
        return self.root[f"CWE-{id}"]

    def get_by_name(self, name: str):
        return self.root[name]


severity_score_map = {
    "info": (0, 0, 0),
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


# ── Clone FORGE EVM dataset ──────────────────────────────────────────────────


def clone_forge_dataset():
    if FORGE_DIR.exists():
        print(f"Repository already exists at: {FORGE_DIR}")
        return True
    print(f"Cloning repository from {FORGE_REPO_URL}...")
    try:
        subprocess.run(
            ["git", "clone", FORGE_REPO_URL, str(FORGE_DIR)],
            capture_output=True,
            text=True,
            check=True,
        )
        print(f"Successfully cloned to: {FORGE_DIR}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error cloning repository: {e.stderr}")
        return False


# ── EVM dataset statistics ───────────────────────────────────────────────────


def evm_statistics(cwe_handler: CWEHandler, cwe_database: CWEDatabase):
    print("\n=== EVM Dataset Statistics ===")
    evm_results_dir = str(FORGE_DIR / "dataset" / "results")

    # Count findings
    FindingResult = namedtuple(
        "FindingResult", ["total_files", "total_projects", "total_findings"]
    )

    def count_finding(report: Report):
        project_count = (
            len(report.project_info.url)
            if isinstance(report.project_info.url, list)
            else 1
        )
        return FindingResult(
            total_findings=len(report.findings),
            total_files=1,
            total_projects=project_count,
        )

    processor = JSONFileProcessor(evm_results_dir)
    results = processor.process_files(analysis_func=count_finding)
    res = processor.operate_add(results, FindingResult)
    print(f"Findings: {res}")

    # Severity
    severity_category = ["na", "info", "low", "medium", "high", "critical"]
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
                continue
            severity_dict[severity] += 1
        return SeverityResult(**severity_dict)

    processor2 = JSONFileProcessor(evm_results_dir)
    sev_results = processor2.process_files(analysis_func=count_severity)
    sev_res = processor2.operate_add(sev_results, SeverityResult)
    print(f"Severity: {sev_res}")

    # CWE statistics
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
                        pillars = cwe_handler.get_root_parents(cwe_value)
                        for pillar in pillars:
                            cwe_ids.append(f"CWE-{pillar.ID}")
                except (ValueError, AttributeError):
                    continue
        return CWE_tuple(cwe_ids=cwe_ids)

    processor3 = JSONFileProcessor(evm_results_dir)
    results_last = processor3.process_files(analysis_func=extract_last_cwe_ids)
    results_pillar = processor3.process_files(analysis_func=extract_pillar_cwe_ids)
    cwe_result = processor3.operate_reduce(results_last, CWE_tuple)
    cwe_pillar_result = processor3.operate_reduce(results_pillar, CWE_tuple)

    cwe_counter = Counter(cwe_result["cwe_ids"])
    print(f"\nTotal unique CWE IDs: {len(set(cwe_result['cwe_ids']))}")
    print("Top 10 CWE IDs by occurrence:")
    for cwe_id, count in cwe_counter.most_common(10):
        print(f"{cwe_id}: {count}")

    cwe_pillar_counter = Counter(cwe_pillar_result["cwe_ids"])
    print(f"\nTotal unique Pillar CWE IDs: {len(set(cwe_pillar_result['cwe_ids']))}")
    print("Top 10 Pillar CWE IDs by occurrence:")
    for cwe_id, count in cwe_pillar_counter.most_common(10):
        print(f"{cwe_id}: {count}")

    # Severity analysis (DAG-aware)
    result_tuple = namedtuple("CWE", ["cwe_name", "severity_score"])
    no_severity_count = {}

    def count_cwe(report: Report):
        cwe_names = []
        severity_score = []
        for finding in report.findings:
            if not finding.category:
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
            if cwe_list:
                cwe_names.extend(cwe_list)
            if not finding.severity or finding.severity.lower().strip() == "n/a":
                if cwe_list:
                    no_severity_count[cwe_list[0]] = (
                        no_severity_count.get(cwe_list[0], 0) + 1
                    )
            else:
                if cwe_list:
                    s_map = severity_score_map.get(
                        finding.severity.lower().strip(), (0, 0, 0)
                    )
                    severity_score.append((cwe_list[0], s_map[2]))
        return result_tuple(cwe_name=cwe_names, severity_score=severity_score)

    processor4 = JSONFileProcessor(evm_results_dir)
    cwe_results = processor4.process_files(analysis_func=count_cwe)
    cwe_res = processor4.operate_reduce(cwe_results, result_tuple)

    cwe_raw_stats = {}
    for cwe_id, score in cwe_res["severity_score"]:
        if cwe_id not in cwe_raw_stats:
            cwe_raw_stats[cwe_id] = {"sum_score": 0.0, "count_rated": 0}
        cwe_raw_stats[cwe_id]["sum_score"] += score
        cwe_raw_stats[cwe_id]["count_rated"] += 1

    cwe_count = Counter(cwe_res["cwe_name"])
    cwe_severity_map = {}
    for cwe_id, stats in cwe_raw_stats.items():
        if stats["count_rated"] > 0:
            cwe_severity_map[cwe_id] = stats["sum_score"] / stats["count_rated"]

    print(f"\nTop 10 CWEs by Average Severity Score:")
    severity_sorted = sorted(cwe_severity_map.items(), key=lambda x: x[1], reverse=True)
    rank = 0
    for cwe_id, sev_score in severity_sorted:
        if rank >= 10:
            break
        if cwe_count[cwe_id] > 15:
            cwe_info = cwe_database.get_by_id(cwe_id)
            print(
                f"CWE-{cwe_id}: {cwe_info.Name}, {cwe_count[cwe_id]}, {sev_score:.3f}"
            )
            rank += 1

    print(f"\nTop 10 Pillar CWEs by Average Severity Score (DAG-aware):")
    pillars = [284, 435, 664, 682, 691, 693, 697, 703, 707, 710]
    pillar_stats = []
    for pillar_id in pillars:
        descendants = cwe_handler.get_all_descendants(pillar_id)
        pillar_cwe = cwe_handler.get_cwe(pillar_id)
        relevant = [pillar_cwe] + descendants if pillar_cwe else descendants
        total_occ = 0
        total_sum = 0
        total_rated = 0
        for cwe in relevant:
            cid = cwe.ID
            total_occ += cwe_count.get(cid, 0)
            if cid in cwe_raw_stats:
                s = cwe_raw_stats[cid]
                total_sum += s["sum_score"]
                total_rated += s["count_rated"]
        avg = total_sum / total_rated if total_rated > 0 else 0
        pillar_stats.append((pillar_id, avg, total_occ))

    pillar_stats.sort(key=lambda x: x[1], reverse=True)
    for pid, avg, cnt in pillar_stats:
        if avg > 0:
            info = cwe_database.get_by_id(pid)
            print(f"CWE-{pid}: {info.Name}, {cnt} occurrences, {avg:.3f} avg severity")

    return pillar_stats


# ── Raw severity extraction ─────────────────────────────────────────────────


def collect_raw_severity(findings_dir: str, cwe_handler: CWEHandler) -> pd.DataFrame:
    """Extract per-finding severity label mapped to CWE pillars."""
    sev_norm = {"info": "informational"}
    valid = {"informational", "low", "medium", "high", "critical"}
    rows = []
    for root, _, files in os.walk(findings_dir):
        for fn in files:
            if not fn.endswith(".json"):
                continue
            with open(os.path.join(root, fn), "r", encoding="utf8") as f:
                data = json.load(f)
            for finding in data.get("findings", []):
                sev = (finding.get("severity") or "").lower().strip()
                sev = sev_norm.get(sev, sev)
                if sev not in valid:
                    continue
                cat = finding.get("category", {})
                if not cat:
                    continue
                final_key = max(cat.keys())
                cwe_list = cat.get(final_key, [])
                if not cwe_list:
                    continue
                cwe_name = cwe_list[0] if isinstance(cwe_list, list) else cwe_list
                if not isinstance(cwe_name, str) or not cwe_name.startswith("CWE-"):
                    continue
                pillars = cwe_handler.get_root_parents(cwe_name)
                for p in pillars:
                    rows.append({"pillar": f"CWE-{p.ID}", "severity": sev})
    return pd.DataFrame(rows)


# ── Heatmap + bar charts ────────────────────────────────────────────────────

SEV_ORDER = ["informational", "low", "medium", "high", "critical"]
SEV_LABELS = ["Info", "Low", "Med", "High", "Crit"]


def plot_dot_severity(solana_data, evm_data, rust_data):
    """Cleveland dot plot: x = CWE pillar, y = min-max normalized severity, size = freq %."""

    def to_df(data, name):
        df = pd.DataFrame(data, columns=["CWE", "Count", "Avg_Severity"])
        total = df["Count"].sum()
        df["Pct"] = df["Count"] / total * 100
        lo, hi = df["Avg_Severity"].min(), df["Avg_Severity"].max()
        df["Norm"] = (df["Avg_Severity"] - lo) / (hi - lo) if hi > lo else 0.5
        df["Ecosystem"] = name
        return df

    df = pd.concat(
        [
            to_df(solana_data, "Solana"),
            to_df(evm_data, "EVM"),
            to_df(rust_data, "Rust"),
        ],
        ignore_index=True,
    )

    pillar_order = sorted(df["CWE"].unique())
    x_map = {cwe: i for i, cwe in enumerate(pillar_order)}

    sns.set_theme(style="ticks")
    colors = sns.color_palette("Paired", 10)  # 取前 3 色
    palette = {"Solana": colors[-1], "EVM": "#8a92ba", "Rust": "#db905b"}
    offsets = {"Solana": 0, "EVM": 0.0, "Rust": 0}
    size_scale = 8.0

    fig, ax = plt.subplots(figsize=(3.45, 2))

    for eco in ["Rust", "EVM", "Solana"]:
        sub = df[df["Ecosystem"] == eco]
        xs = [x_map[c] + offsets[eco] for c in sub["CWE"]]
        ax.scatter(
            xs,
            sub["Norm"],
            s=sub["Pct"] * size_scale,
            color=palette[eco],
            edgecolors="white",
            linewidths=0.3,
            alpha=0.85,
            zorder=3,
            label=eco,
            clip_on=False,  # let bubbles centred at 1.0 render fully past the top
        )

    ax.set_xticks(range(len(pillar_order)))
    ax.set_xticklabels(pillar_order, fontsize=5.5, rotation=35, ha="right")
    ax.set_ylabel("Rel. Severity", fontsize=7)
    # Headroom above 1.0 so the largest bubbles (centred at Norm=1.0) are not
    # clipped, while the highest *labelled* tick stays 1.0 (values are min-max
    # normalized and cannot exceed 1.0 by definition).
    ax.set_ylim(-0.1, 1.28)
    ax.set_yticks([0, 0.5, 1.0])
    ax.tick_params(axis="y", labelsize=6)
    ax.grid(axis="y", linestyle="--", alpha=0.25, zorder=0)
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)

    # Ecosystem legend, lifted clear of the plotting area (dot size already
    # conveys frequency, so the separate frequency legend is omitted).
    # Use uniform, small proxy markers so the three legend dots match in size
    # (a scatter with variable `s` would otherwise give each a different size).
    from matplotlib.lines import Line2D

    legend_labels = {"Solana": "Solana", "EVM": "EVM", "Rust": "Traditional Rust"}
    legend_handles = [
        Line2D(
            [0],
            [0],
            marker="o",
            linestyle="",
            markersize=4,
            markerfacecolor=palette[eco],
            markeredgecolor="white",
            markeredgewidth=0.3,
            label=legend_labels[eco],
        )
        for eco in ["Solana", "EVM", "Rust"]
    ]
    ax.legend(
        handles=legend_handles,
        fontsize=6,
        loc="lower left",
        frameon=False,
        ncol=3,
        handletextpad=0.2,
        columnspacing=0.6,
        borderpad=0.1,
        bbox_to_anchor=(0, 1.0),
    )

    plt.tight_layout()
    out = SCRIPT_DIR / "rq3_severity.pdf"
    fig.savefig(out, bbox_inches="tight", pad_inches=0.02, dpi=300)
    print(f"Saved {out}")
    plt.close()


def plot_grouped_bar_frequency(solana_data, evm_data):
    total_solana = sum(item[1] for item in solana_data)
    total_evm = sum(item[1] for item in evm_data)

    df_solana = pd.DataFrame(solana_data, columns=["CWE", "Count", "Avg_Severity"])
    df_evm = pd.DataFrame(evm_data, columns=["CWE", "Count", "Avg_Severity"])
    df_solana["Pct"] = (df_solana["Count"] / total_solana) * 100
    df_evm["Pct"] = (df_evm["Count"] / total_evm) * 100

    all_cwes = sorted(set(df_solana["CWE"]) | set(df_evm["CWE"]))
    df = pd.DataFrame({"CWE": all_cwes})
    df = pd.merge(df, df_solana[["CWE", "Pct"]], on="CWE", how="left").rename(
        columns={"Pct": "Solana"}
    )
    df = pd.merge(df, df_evm[["CWE", "Pct"]], on="CWE", how="left").rename(
        columns={"Pct": "EVM"}
    )
    df = df.fillna(0).sort_values("Solana", ascending=False)

    sns.set_theme(style="ticks")
    palette = [sns.color_palette("muted")[0], sns.color_palette("muted")[1]]
    fig, ax = plt.subplots(figsize=(3.5, 2.8))
    y = np.arange(len(df))
    h = 0.35

    ax.barh(
        y - h / 2, df["Solana"], height=h, label="Solana", color=palette[0], alpha=0.9
    )
    ax.barh(y + h / 2, df["EVM"], height=h, label="EVM", color=palette[1], alpha=0.9)

    ax.set_yticks(y)
    ax.set_yticklabels(df["CWE"], fontsize=7)
    ax.set_xlabel("Percentage (%)", fontsize=8)
    ax.tick_params(axis="x", labelsize=7)
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.legend(fontsize=7, loc="lower right", frameon=False)
    ax.grid(axis="x", linestyle="--", alpha=0.3)
    plt.tight_layout()
    out = SCRIPT_DIR / "rq3_frequency.pdf"
    fig.savefig(out, bbox_inches="tight", pad_inches=0.02, dpi=300)
    print(f"Saved {out}")
    plt.close()


# ── Live pillar-stats computation (replaces hardcoded arrays) ────────────────

PILLAR_IDS = [284, 435, 664, 682, 691, 693, 697, 703, 707, 710]


def _aggregate_pillars(cwe_count, cwe_raw_stats, cwe_handler, cwe_database):
    """Roll last-level CWE counts/scores up to the 10 Pillar roots (DAG-aware).

    Returns ``[("CWE-<pid>", occurrences, avg_severity), ...]`` in PILLAR_IDS order.
    """
    out = []
    for pid in PILLAR_IDS:
        descendants = cwe_handler.get_all_descendants(pid)
        pcwe = cwe_handler.get_cwe(pid)
        relevant = ([pcwe] + descendants) if pcwe else descendants
        occ = sum(cwe_count.get(c.ID, 0) for c in relevant)
        ssum = sum(cwe_raw_stats.get(c.ID, {}).get("sum_score", 0.0) for c in relevant)
        rated = sum(cwe_raw_stats.get(c.ID, {}).get("count_rated", 0) for c in relevant)
        out.append((f"CWE-{pid}", occ, ssum / rated if rated else 0.0))
    return out


def report_pillar_data(findings_dir, cwe_handler, cwe_database):
    """Pillar (count, avg-severity) for Report-format datasets (Solana, EVM).

    Severity uses ``severity_score_map`` ('informational' -> 0, 'n/a' unrated),
    matching the methodology used elsewhere in the artifact.
    """
    cwe_count = Counter()
    cwe_raw_stats = {}

    def collect(report: Report):
        for finding in report.findings:
            if not finding.category:
                continue
            final_key = max(finding.category.keys())
            names = finding.category.get(final_key) or []
            cid = None
            for name in names:
                try:
                    cid = cwe_database.get_by_name(name).ID
                    break
                except KeyError:
                    continue
            if cid is None:
                continue
            cwe_count[cid] += 1
            sev = (finding.severity or "").lower().strip()
            if sev and sev != "n/a":
                score = severity_score_map.get(sev, (0, 0, 0))[2]
                st = cwe_raw_stats.setdefault(cid, {"sum_score": 0.0, "count_rated": 0})
                st["sum_score"] += score
                st["count_rated"] += 1
        return None

    JSONFileProcessor(findings_dir).process_files(analysis_func=collect)
    return _aggregate_pillars(cwe_count, cwe_raw_stats, cwe_handler, cwe_database)


def rust_pillar_data(cwe_handler, cwe_database):
    """Pillar (count, avg-severity) for the OSV / traditional-Rust baseline.

    Reuses rq4's OSV loader so advisory parsing/CVSS scoring lives in one place.
    """
    sys.path.insert(0, str(SCRIPT_DIR / "rust"))
    import statistic_rust as osv  # noqa: E402

    filtered = osv.load_advisories()
    cwe_count = Counter()
    cwe_raw_stats = {}
    for adv in filtered:
        cwe_ids = adv.database_specific.cwe_ids if adv.database_specific else []
        score = adv.get_severity_score()
        for cwe_id in cwe_ids:
            cid = int(cwe_id.replace("CWE-", ""))
            cwe_count[cid] += 1
            st = cwe_raw_stats.setdefault(cid, {"sum_score": 0.0, "count_rated": 0})
            st["sum_score"] += score
            st["count_rated"] += 1
    return _aggregate_pillars(cwe_count, cwe_raw_stats, cwe_handler, cwe_database)


# ── Main ─────────────────────────────────────────────────────────────────────


def main():
    clone_forge_dataset()

    print("Loading CWE database...")
    cwe_handler = CWEHandler.load_from_file(str(CWE_DICT_PATH))
    cwe_database = cwe_handler.db

    # Verbose EVM diagnostics (top CWEs, severity ranking, etc.)
    evm_statistics(cwe_handler, cwe_database)

    # Pillar-level aggregate data computed live from each dataset:
    # (CWE, occurrences, avg_severity). No hardcoded numbers.
    solana_data = report_pillar_data(
        str(DATASET_DIR / "findings"), cwe_handler, cwe_database
    )
    evm_data = report_pillar_data(
        str(FORGE_DIR / "dataset" / "results"), cwe_handler, cwe_database
    )
    rust_data = rust_pillar_data(cwe_handler, cwe_database)

    print("\n=== Pillar aggregates used for plotting (CWE, count, avg_sev) ===")
    for name, data in [("Solana", solana_data), ("EVM", evm_data), ("Rust", rust_data)]:
        print(f"{name}: " + ", ".join(f"{c}({n},{s:.3f})" for c, n, s in data))

    plot_dot_severity(solana_data, evm_data, rust_data)
    plot_grouped_bar_frequency(solana_data, evm_data)


if __name__ == "__main__":
    main()
