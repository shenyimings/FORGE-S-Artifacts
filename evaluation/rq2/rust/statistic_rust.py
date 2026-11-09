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
"""RQ4: Solana vs Traditional Rust — severity & frequency comparison via OSV advisories."""

from __future__ import annotations

import copy
import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Dict, List, Literal, Optional, Union

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from pydantic import BaseModel, Field, RootModel, field_validator
import cvss

SCRIPT_DIR = Path(__file__).parent
CWE_DICT_PATH = SCRIPT_DIR.parent.parent / "rq3" / "evaluator" / "cwe_dict.json"
OSV_DIR = SCRIPT_DIR / "OSV"

plt.switch_backend("Agg")


# ── OSV Advisory models ──────────────────────────────────────────────────────

class Package(BaseModel):
    name: str
    ecosystem: str
    purl: Optional[str] = None


class RangeEvent(BaseModel):
    introduced: Optional[str] = None
    fixed: Optional[str] = None


class Range(BaseModel):
    type: str
    events: List[RangeEvent]


class Affected(BaseModel):
    package: Package
    ranges: List[Range] = []
    database_specific: Optional[Dict[str, Any]] = None


class Reference(BaseModel):
    type: str
    url: str

    def __hash__(self):
        return hash(f"{self.type}: {self.url}")


class Severity(BaseModel):
    type: str
    score: str

    def __hash__(self):
        return hash(f"{self.type}: {self.score}")

    def get_severity_score(self) -> float:
        try:
            if self.type.lower() == "cvss_v4":
                return float(cvss.CVSS4(self.score).base_score)
            if self.type.lower() == "cvss_v3":
                return float(cvss.CVSS3(self.score).base_score)
            return -1.0
        except ValueError:
            print(f"Invalid severity score: {self.score}")
            return -1.0


class DatabaseSpecific(BaseModel):
    cwe_ids: List[str] = Field(default_factory=list)
    severity: Optional[str] = None
    github_reviewed: Optional[bool] = None

    class Config:
        extra = "allow"


class Advisory(BaseModel):
    schema_version: str
    id: str
    published: str
    modified: str
    aliases: List[str] = Field(default_factory=list)
    summary: str
    details: str
    affected: List[Affected] = []
    references: List[Reference] = []
    severity: List[Severity] = []
    database_specific: Optional[DatabaseSpecific] = None

    def get_severity_score(self) -> float:
        if not self.severity:
            return -1.0
        if len(self.severity) > 1:
            max_sev = -1.0
            for sev in self.severity:
                if sev.type == "CVSS_V4":
                    return sev.get_severity_score()
                max_sev = max(max_sev, sev.get_severity_score())
            return max_sev
        return self.severity[0].get_severity_score()

    def merge_rustsec_data(self, other: "Advisory"):
        if self.summary.lower() != other.summary.lower():
            if other.summary:
                self.summary = f"{self.summary}\n{other.summary}"
        if self.details.lower() != other.details.lower():
            if other.details:
                self.details = f"{self.details}\n{other.details}"
        self.references = list(set(self.references).union(set(other.references)))
        self.severity = list(set(self.severity).union(set(other.severity)))
        current_cwes = set(self.database_specific.cwe_ids if self.database_specific else [])
        other_cwes = set(other.database_specific.cwe_ids if other.database_specific else [])
        merged = list(current_cwes.union(other_cwes))
        if self.database_specific:
            self.database_specific.cwe_ids = merged
        else:
            self.database_specific = DatabaseSpecific(cwe_ids=merged)


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
            cleaned = []
            for p in cwe.Parent:
                if isinstance(p, CWEEntry):
                    cleaned.append(p)
                elif isinstance(p, int):
                    try:
                        cleaned.append(self.db.get_by_id(p))
                    except KeyError:
                        pass
            cwe.Parent = cleaned
        for cwe in self.db.root.values():
            for child_id in cwe.Child:
                try:
                    child = self.db.get_by_id(child_id)
                    if cwe not in child.Parent:
                        child.Parent.append(cwe)
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


# ── Load & deduplicate OSV advisories ────────────────────────────────────────

def load_advisories() -> List[Advisory]:
    print("Loading OSV advisories...")
    advisories = []
    for p in OSV_DIR.glob("*.json"):
        data = json.loads(p.read_text())
        advisories.append(Advisory.model_validate(data))
    print(f"{len(advisories)} advisories loaded.")

    # Deduplicate per-package
    advisories_by_package = defaultdict(list)
    for adv in advisories:
        if adv.affected:
            for affected in adv.affected:
                advisories_by_package[affected.package.name].append(copy.deepcopy(adv))
        else:
            advisories_by_package["__unknown__"].append(copy.deepcopy(adv))

    unique: List[Advisory] = []
    for pkg, pkg_advs in advisories_by_package.items():
        grouped: List[Advisory] = []
        for adv in pkg_advs:
            is_dup = False

            def get_filtered_refs(a):
                return set(ref.url for ref in a.references if "nvd" in ref.url or "rustsec" in ref.url)

            refs_a = get_filtered_refs(adv)
            if refs_a:
                for u_adv in grouped:
                    refs_b = get_filtered_refs(u_adv)
                    if refs_a & refs_b:
                        is_dup = True
                        u_adv.merge_rustsec_data(adv)
                        break
            if not is_dup:
                grouped.append(adv)
        unique.extend(grouped)

    print(f"{len(unique)} unique advisories after deduplication.")

    filtered = [
        adv for adv in unique
        if (adv.database_specific and adv.database_specific.cwe_ids) and adv.get_severity_score() >= 0.0
    ]
    print(f"{len(filtered)} advisories have CWE and severity score.")
    return filtered


# ── CWE statistics ───────────────────────────────────────────────────────────

def cwe_statistics(filtered: List[Advisory], cwe_handler: CWEHandler, cwe_database: CWEDatabase):
    print("\n=== CWE Statistics ===")

    cwe_result = []
    cwe_pillar_result = []
    for adv in filtered:
        cwe_ids = adv.database_specific.cwe_ids if adv.database_specific else []
        cwe_result.extend(cwe_ids)
        for cwe_id in cwe_ids:
            for parent in cwe_handler.get_root_parents(cwe_id):
                if parent.Abstraction == "Pillar":
                    cwe_pillar_result.append(f"CWE-{parent.ID}")

    cwe_counter = Counter(cwe_result)
    print(f"Total unique CWE IDs: {len(set(cwe_result))}")
    print("Top 10 CWE IDs by occurrence:")
    for cwe_id, count in cwe_counter.most_common(10):
        print(f"{cwe_id}: {count}")

    cwe_pillar_counter = Counter(cwe_pillar_result)
    print(f"\nTotal unique Pillar CWE IDs: {len(set(cwe_pillar_result))}")
    print("Top 10 Pillar CWE IDs by occurrence:")
    for cwe_id, count in cwe_pillar_counter.most_common(10):
        print(f"{cwe_id}: {count}")

    # Severity analysis
    cwe_raw_stats = {}
    res_cwe_name = []
    for adv in filtered:
        cwe_ids = adv.database_specific.cwe_ids if adv.database_specific else []
        score = adv.get_severity_score()
        for cwe_id in cwe_ids:
            id_int = int(cwe_id.replace("CWE-", ""))
            res_cwe_name.append(id_int)
            if id_int not in cwe_raw_stats:
                cwe_raw_stats[id_int] = {"sum_score": 0.0, "count_rated": 0}
            cwe_raw_stats[id_int]["sum_score"] += score
            cwe_raw_stats[id_int]["count_rated"] += 1

    cwe_count = Counter(res_cwe_name)
    cwe_severity = {}
    for cid, stats in cwe_raw_stats.items():
        if stats["count_rated"] > 0:
            cwe_severity[cid] = stats["sum_score"] / stats["count_rated"]

    print(f"\nTop 10 CWEs by Average Severity Score:")
    severity_sorted = sorted(cwe_severity.items(), key=lambda x: x[1], reverse=True)
    rank = 0
    for cid, sev in severity_sorted:
        if rank >= 10:
            break
        if cwe_count[cid] > 5:
            try:
                info = cwe_database.get_by_id(cid)
                print(f"CWE-{cid}: {info.Name}, {cwe_count[cid]}, {sev:.3f}")
                rank += 1
            except Exception:
                pass

    print(f"\nTop 10 Pillar CWEs by Average Severity Score (DAG-aware):")
    pillars = [284, 435, 664, 682, 691, 693, 697, 703, 707, 710]
    pillar_stats = []
    for pid in pillars:
        descendants = cwe_handler.get_all_descendants(pid)
        pcwe = cwe_handler.get_cwe(pid)
        relevant = [pcwe] + descendants if pcwe else descendants
        total_occ = sum(cwe_count.get(c.ID, 0) for c in relevant)
        total_sum = sum(cwe_raw_stats.get(c.ID, {}).get("sum_score", 0) for c in relevant)
        total_rated = sum(cwe_raw_stats.get(c.ID, {}).get("count_rated", 0) for c in relevant)
        avg = total_sum / total_rated if total_rated > 0 else 0
        pillar_stats.append((pid, avg, total_occ))

    pillar_stats.sort(key=lambda x: x[1], reverse=True)
    for pid, avg, cnt in pillar_stats:
        if avg > 0:
            info = cwe_database.get_by_id(pid)
            print(f"CWE-{pid}: {info.Name}, {cnt} occurrences, {avg:.3f} avg severity")


# ── Butterfly charts ─────────────────────────────────────────────────────────

def plot_butterfly_severity(solana_data, rust_data, exclude_info: bool = False):
    label = "_excl_info" if exclude_info else ""
    avg_solana = sum(item[2] for item in solana_data) / len(solana_data)
    avg_rust = sum(item[2] for item in rust_data) / len(rust_data)

    df_solana = pd.DataFrame(solana_data, columns=["CWE", "Count", "Avg_Severity"])
    df_rust = pd.DataFrame(rust_data, columns=["CWE", "Count", "Avg_Severity"])

    all_cwes = sorted(set(df_solana["CWE"]) | set(df_rust["CWE"]))
    df = pd.DataFrame({"CWE": all_cwes})
    df = pd.merge(df, df_solana[["CWE", "Avg_Severity"]], on="CWE", how="left").rename(columns={"Avg_Severity": "Solana"})
    df = pd.merge(df, df_rust[["CWE", "Avg_Severity"]], on="CWE", how="left").rename(columns={"Avg_Severity": "Rust"})
    df = df.fillna(0).sort_values("Solana", ascending=True)

    sns.set_theme(style="ticks")
    solana_color, rust_color = "#4c366c", "#db905b"
    fig, ax = plt.subplots(figsize=(7, 4.5))
    y = np.arange(len(df))
    h = 0.35

    ax.barh(y, -df["Solana"], height=h, label="Solana", color=solana_color, alpha=0.9)
    ax.barh(y, df["Rust"], height=h, label="Traditional Rust", color=rust_color, alpha=0.9)
    ax.axvline(x=-avg_solana, color=solana_color, linestyle="--", linewidth=1.5, alpha=0.5, ymin=0.05, ymax=0.97)
    ax.axvline(x=avg_rust, color=rust_color, linestyle="--", linewidth=1.5, alpha=0.5, ymin=0.05, ymax=0.97)

    bbox_avg = dict(boxstyle="round,pad=0.2", fc="white", ec="none", alpha=0.2)
    ax.text(-avg_solana, 0.6, f"Avg: {avg_solana:.2f}", color=solana_color, ha="center", va="top", fontsize=12, fontweight="bold", bbox=bbox_avg, zorder=20)
    ax.text(avg_rust - 1.5, 0.78, f"Avg: {avg_rust:.2f}", color=rust_color, ha="center", va="top", fontsize=12, fontweight="bold", bbox=bbox_avg, zorder=20)

    bbox = dict(boxstyle="round,pad=0.2", fc="white", ec="none", alpha=0.8)
    for i, (_, row) in enumerate(df.iterrows()):
        if row["Solana"] > 0:
            ax.text(-row["Solana"] - 0.05, y[i], f"{row['Solana']:.2f}", va="center", ha="right", fontsize=10, bbox=bbox, zorder=10)
        if row["Rust"] > 0:
            ax.text(row["Rust"] + 0.05, y[i], f"{row['Rust']:.2f}", va="center", ha="left", fontsize=10, bbox=bbox, zorder=10)

    ax.set_yticks(y)
    ax.set_yticklabels(df["CWE"], ha="right")
    ax.set_xlabel("Average Severity Score", fontsize=12)
    ticks = ax.get_xticks()
    ax.set_xticklabels([f"{abs(t):.1f}" for t in ticks])
    ax.grid(axis="x", linestyle="--", alpha=0.3)
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.spines["left"].set_visible(False)
    ax.axvline(0, color="black", linewidth=0.8)
    ax.legend(loc="upper left", bbox_to_anchor=(0, 1.08), ncol=2, frameon=False, fontsize=12)
    plt.tight_layout()
    out = SCRIPT_DIR / f"rq4_severity{label}.pdf"
    fig.savefig(out, bbox_inches="tight", pad_inches=0)
    print(f"Saved {out}")
    plt.close()


def plot_butterfly_frequency(solana_data, rust_data, exclude_info: bool = False):
    label = "_excl_info" if exclude_info else ""
    total_solana = sum(item[1] for item in solana_data)
    total_rust = sum(item[1] for item in rust_data)

    df_solana = pd.DataFrame(solana_data, columns=["CWE", "Count", "Avg_Severity"])
    df_rust = pd.DataFrame(rust_data, columns=["CWE", "Count", "Avg_Severity"])
    df_solana["Percentage"] = (df_solana["Count"] / total_solana) * 100
    df_rust["Percentage"] = (df_rust["Count"] / total_rust) * 100

    all_cwes = sorted(set(df_solana["CWE"]) | set(df_rust["CWE"]))
    df = pd.DataFrame({"CWE": all_cwes})
    df = pd.merge(df, df_solana[["CWE", "Percentage"]], on="CWE", how="left").rename(columns={"Percentage": "Solana"})
    df = pd.merge(df, df_rust[["CWE", "Percentage"]], on="CWE", how="left").rename(columns={"Percentage": "Rust"})
    df = df.fillna(0).sort_values("Solana", ascending=True)

    sns.set_theme(style="ticks")
    solana_color, rust_color = "#4c366c", "#db905b"
    fig, ax = plt.subplots(figsize=(7, 4.5))
    y = np.arange(len(df))
    h = 0.35

    ax.barh(y, -df["Solana"], height=h, label="Solana", color=solana_color, alpha=0.9)
    ax.barh(y, df["Rust"], height=h, label="Traditional Rust", color=rust_color, alpha=0.9)

    bbox = dict(boxstyle="round,pad=0.2", fc="white", ec="none", alpha=0.8)
    for i, (_, row) in enumerate(df.iterrows()):
        if row["Solana"] > 0:
            ax.text(-row["Solana"], y[i], f"{row['Solana']:.2f}%", va="center", ha="right", fontsize=10, bbox=bbox)
        if row["Rust"] > 0:
            ax.text(row["Rust"] + 0.5, y[i], f"{row['Rust']:.2f}%", va="center", ha="left", fontsize=10, bbox=bbox)

    ax.set_yticks(y)
    ax.set_yticklabels(df["CWE"], ha="right")
    ax.set_xlabel("Percentage (%)", fontsize=12)
    ticks = ax.get_xticks()
    ax.set_xticklabels([f"{abs(t):.0f}" for t in ticks])
    ax.grid(axis="x", linestyle="--", alpha=0.3)
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    ax.spines["left"].set_visible(False)
    ax.axvline(0, color="black", linewidth=0.8)
    ax.legend(loc="upper left", bbox_to_anchor=(0, 1.08), ncol=2, frameon=False, fontsize=12)
    plt.tight_layout()
    out = SCRIPT_DIR / f"rq4_frequency{label}.pdf"
    fig.savefig(out, bbox_inches="tight", pad_inches=0)
    print(f"Saved {out}")
    plt.close()


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    cwe_handler = CWEHandler.load_from_file(str(CWE_DICT_PATH))
    cwe_database = cwe_handler.db

    filtered = load_advisories()
    cwe_statistics(filtered, cwe_handler, cwe_database)

    # All findings (matches notebook cell-8)
    solana_data_all = [
        ("CWE-697", 56, 4.099), ("CWE-284", 260, 4.205), ("CWE-691", 261, 3.842),
        ("CWE-682", 396, 4.054), ("CWE-693", 252, 4.209), ("CWE-664", 441, 3.646),
        ("CWE-435", 100, 3.290), ("CWE-707", 492, 3.144), ("CWE-703", 189, 2.993),
        ("CWE-710", 470, 1.491),
    ]
    rust_data = [
        ("CWE-693", 106, 6.790), ("CWE-664", 691, 7.226), ("CWE-284", 66, 6.502),
        ("CWE-707", 100, 7.192), ("CWE-691", 143, 7.100), ("CWE-682", 42, 6.260),
        ("CWE-703", 60, 6.582), ("CWE-435", 8, 5.750), ("CWE-697", 5, 5.960),
        ("CWE-710", 19, 6.268),
    ]
    plot_butterfly_severity(solana_data_all, rust_data)
    plot_butterfly_frequency(solana_data_all, rust_data)

    # Excluding informational (matches notebook cell-10/11)
    solana_data_excl = [
        ("CWE-697", 49, 4.737), ("CWE-284", 219, 5.034), ("CWE-691", 218, 4.656),
        ("CWE-682", 345, 4.680), ("CWE-693", 207, 5.161), ("CWE-664", 350, 4.645),
        ("CWE-435", 77, 4.326), ("CWE-707", 382, 4.121), ("CWE-703", 140, 4.139),
        ("CWE-710", 209, 3.572),
    ]
    plot_butterfly_severity(solana_data_excl, rust_data, exclude_info=True)
    plot_butterfly_frequency(solana_data_excl, rust_data, exclude_info=True)


if __name__ == "__main__":
    main()
