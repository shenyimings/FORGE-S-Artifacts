# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pydantic>=2.0",
#     "pandas>=2.0",
#     "seaborn>=0.13",
#     "matplotlib>=3.8",
#     "numpy>=1.24",
#
# ]
# ///
"""RQ5 analysis: model performance, CWE heatmaps, temporal F1."""

from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set

import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns
from pydantic import BaseModel, Field, model_validator


# ── Data models ──────────────────────────────────────────────────────────────

def f1_calculate(precision: float, recall: float) -> float:
    if precision + recall == 0:
        return 0.0
    return 2 * (precision * recall) / (precision + recall)


class VulnPair(BaseModel):
    files: List[Path] = Field(default_factory=list)
    ground_truth: Set[str] = Field(default_factory=set)
    detected: Set[str] = Field(default_factory=set)
    score_board: Dict[str, float] = Field(default_factory=dict)
    score: float = 0.0

    @model_validator(mode="after")
    def calculate_score(self):
        self.score = sum(self.score_board.get(d, 0) for d in self.detected)
        return self


class Record(BaseModel):
    project_name: str
    report_path: str
    scan_time: float
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int
    tp_size: int = 0
    p_size: int = 0
    t_size: int = 0
    h_precision: float = 0
    h_recall: float = 0
    h_f1: float = 0
    score: float = 0
    ground_truth: List[Set[str]] = Field(default_factory=list)
    detected: List[Set[str]] = Field(default_factory=list)
    vuln_pairs: List[VulnPair] = Field(default_factory=list)

    @model_validator(mode="after")
    def calculate_metrics(self):
        self.h_precision = self.tp_size / self.p_size if self.p_size > 0 else 0.0
        self.h_recall = self.tp_size / self.t_size if self.t_size > 0 else 0.0
        self.h_f1 = f1_calculate(self.h_precision, self.h_recall)
        self.score = sum(vp.score for vp in self.vuln_pairs)
        return self


class Report(BaseModel):
    model: str
    total_scan_time: float = 0
    total_prompt_tokens: int = 0
    total_completion_tokens: int = 0
    total_tokens: int = 0
    h_precision: float = 0
    h_recall: float = 0
    h_f1: float = 0
    score: float = 0
    record: List[Record] = Field(default_factory=list)

    @model_validator(mode="after")
    def calculate_metrics(self):
        total_tp = total_p = total_t = 0
        score = 0.0
        for rec in self.record:
            total_tp += rec.tp_size
            total_p += rec.p_size
            total_t += rec.t_size
            score += rec.score
        self.score = score
        self.h_precision = total_tp / total_p if total_p > 0 else 0.0
        self.h_recall = total_tp / total_t if total_t > 0 else 0.0
        self.h_f1 = f1_calculate(self.h_precision, self.h_recall)
        return self


# ── Configuration ────────────────────────────────────────────────────────────

HEATMAP_X_IS_CWE = True
RESULTS_DIR = Path(__file__).parent / "results"
REPO_ROOT = Path(__file__).parent.parent.parent  # solana-security/
CWE_LABELS = [
    "CWE-284", "CWE-435", "CWE-691", "CWE-682", "CWE-697",
    "CWE-710", "CWE-664", "CWE-693", "CWE-707", "CWE-703",
]

MODEL_NAME_MAP = {
    "claude45": "Claude 4.5",
    "deepseek-r1": "DeepSeek R1",
    "gpt52": "GPT 5.2",
    "qwen3": "Qwen 3",
}

sns.set_theme(style="ticks")
plt.rcParams.update({
    "font.size": 14,
    "pdf.fonttype": 42,
    "ps.fonttype": 42,
})


# ── Helpers ──────────────────────────────────────────────────────────────────

def load_reports() -> Dict[str, Report]:
    report_files = sorted(RESULTS_DIR.glob("*.json"))
    print(f"Loading {len(report_files)} reports from {RESULTS_DIR}...")
    detail_reports: Dict[str, Report] = {}
    for f in report_files:
        try:
            data = json.loads(f.read_text())
            report = Report(**data)
            detail_reports[report.model] = report
        except Exception as e:
            print(f"Failed to load {f.name}: {e}")
    if not detail_reports:
        print("No reports loaded. Check directory.")
    else:
        print(f"Loaded models: {list(detail_reports.keys())}")
    return detail_reports


def print_summary(detail_reports: Dict[str, Report]) -> None:
    print("\n--- Model Performance Summary ---")
    for model, report in detail_reports.items():
        display_name = MODEL_NAME_MAP.get(model, model)
        total_tp = sum(rec.tp_size for rec in report.record)
        total_p = sum(rec.p_size for rec in report.record)
        total_t = sum(rec.t_size for rec in report.record)
        print(f"Model: {display_name}")
        print(f"  Token Usage:")
        print(f"    - Input Tokens:      {report.total_prompt_tokens:,}")
        print(f"    - Completion Tokens: {report.total_completion_tokens:,}")
        print(f"    - Total Tokens:      {report.total_tokens:,}")
        print(f"  Performance:")
        print(f"    - Scan Time (s):     {report.total_scan_time:.2f}")
        print(f"    - TP Size:           {total_tp}")
        print(f"    - P Size (Detected): {total_p}")
        print(f"    - T Size (Ground T): {total_t}")
        print(f"    - Precision:         {report.h_precision:.4f}")
        print(f"    - Recall:            {report.h_recall:.4f}")
        print(f"    - F1 Score:          {report.h_f1:.4f}")
        print(f"    - Severity Score:    {report.score:.2f}")
        print("-" * 40)


# ── Plot 1: Performance bar chart ────────────────────────────────────────────

def plot_performance(detail_reports: Dict[str, Report]) -> None:
    metrics_data = []
    score_data = []
    for model, report in detail_reports.items():
        name = MODEL_NAME_MAP.get(model, model)
        metrics_data.append({"Model": name, "Metric": "Precision", "Value": report.h_precision})
        metrics_data.append({"Model": name, "Metric": "Recall", "Value": report.h_recall})
        metrics_data.append({"Model": name, "Metric": "F1 Score", "Value": report.h_f1})
        score_data.append({"Model": name, "Score": report.score})

    df_metrics = pd.DataFrame(metrics_data)
    df_score = pd.DataFrame(score_data)
    models = sorted(set(df_metrics["Model"]) | set(df_score["Model"]))

    precisions, recalls, f1s, scores = [], [], [], []
    for m in models:
        p = df_metrics[(df_metrics["Model"] == m) & (df_metrics["Metric"] == "Precision")]["Value"].values
        r = df_metrics[(df_metrics["Model"] == m) & (df_metrics["Metric"] == "Recall")]["Value"].values
        f = df_metrics[(df_metrics["Model"] == m) & (df_metrics["Metric"] == "F1 Score")]["Value"].values
        s = df_score[df_score["Model"] == m]["Score"].values
        precisions.append(p[0] if len(p) else 0)
        recalls.append(r[0] if len(r) else 0)
        f1s.append(f[0] if len(f) else 0)
        scores.append(s[0] if len(s) else 0)

    x = np.arange(len(models))
    width = 0.15
    fig, ax1 = plt.subplots(figsize=(6, 2.8))
    colors = sns.color_palette("crest")

    rects1 = ax1.bar(x - 1.5 * width, precisions, width, label="Precision", color=colors[-5], alpha=0.9)
    rects2 = ax1.bar(x - 0.5 * width, recalls, width, label="Recall", color=colors[-3], alpha=0.9)
    rects3 = ax1.bar(x + 0.5 * width, f1s, width, label="F1", color=colors[-1], alpha=0.9)

    ax1.set_ylabel("Metrics (P, R, F)")
    ax1.set_ylim(0, 0.6)
    ax1.set_xticks(x)
    ax1.set_xticklabels(models, rotation=0)
    ax1.grid(True, axis="y", linestyle="--", alpha=0.5)

    ax2 = ax1.twinx()
    rects4 = ax2.bar(x + 1.5 * width, scores, width, label="Severity Score", color=colors[3], hatch="//", alpha=0.6)
    ax2.set_ylabel("Total Severity Score")
    ax2.set_ylim(0, 4200)
    ax2.grid(False)

    def autolabel(rects, ax, fmt="{:.2f}"):
        for rect in rects:
            height = rect.get_height()
            ax.annotate(
                fmt.format(height),
                xy=(rect.get_x() + rect.get_width() / 2 + 0.02, height),
                xytext=(0, 2),
                textcoords="offset points",
                ha="center", va="bottom", fontsize=10, rotation=90,
            )

    autolabel(rects1, ax1)
    autolabel(rects2, ax1)
    autolabel(rects3, ax1)
    autolabel(rects4, ax2, fmt="{:.0f}")

    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax1.legend(lines1 + lines2, labels1 + labels2, loc="upper center", bbox_to_anchor=(0.5, 1.18), ncol=4, frameon=False)

    plt.tight_layout()
    out = RESULTS_DIR.parent / "rq3_performance.pdf"
    plt.savefig(out, format="pdf", bbox_inches="tight", pad_inches=0)
    print(f"Saved {out}")
    plt.close()


# ── Plot 2 & 3: CWE heatmaps ────────────────────────────────────────────────

def get_cwe_heatmap_data(reports: Dict[str, Report], target_cwes: list, mode: str = "tp") -> pd.DataFrame:
    matrix_data = {model: {cwe: 0 for cwe in target_cwes} for model in reports}
    for model, report in reports.items():
        for rec in report.record:
            for vp in rec.vuln_pairs:
                cwes_found = vp.detected.intersection(vp.ground_truth) if mode == "tp" else vp.detected
                for cwe in cwes_found:
                    if cwe in matrix_data[model]:
                        matrix_data[model][cwe] += 1
    return pd.DataFrame(matrix_data).T


def plot_heatmaps(detail_reports: Dict[str, Report]) -> None:
    df_tp = get_cwe_heatmap_data(detail_reports, CWE_LABELS, mode="tp")
    df_p = get_cwe_heatmap_data(detail_reports, CWE_LABELS, mode="p")

    if not HEATMAP_X_IS_CWE:
        df_tp = df_tp.T
        df_p = df_p.T

    plt.figure(figsize=(8, 6))
    sns.heatmap(df_tp, annot=True, fmt="d", cmap="Greens", linewidths=0.5)
    plt.tight_layout()
    plt.close()

    plt.figure(figsize=(8, 6))
    sns.heatmap(df_p, annot=True, fmt="d", cmap="Blues", linewidths=0.5)
    plt.tight_layout()
    out = RESULTS_DIR.parent / "rq3_heatmap_det.pdf"
    plt.savefig(out, format="pdf", bbox_inches="tight", pad_inches=0)
    print(f"Saved {out}")
    plt.close()


# ── Plot 4: Per-CWE F1 heatmap ──────────────────────────────────────────────

def plot_cwe_f1_heatmap(detail_reports: Dict[str, Report]) -> None:
    print("Calculating F1 Scores per CWE...")
    cwe_metrics: Dict[str, Dict[str, Dict[str, int]]] = {
        model: {cwe: {"tp": 0, "fp": 0, "fn": 0} for cwe in CWE_LABELS}
        for model in detail_reports
    }
    for model, report in detail_reports.items():
        for rec in report.record:
            for vp in rec.vuln_pairs:
                for cwe in CWE_LABELS:
                    in_gt = cwe in vp.ground_truth
                    in_det = cwe in vp.detected
                    if in_gt and in_det:
                        cwe_metrics[model][cwe]["tp"] += 1
                    elif in_det and not in_gt:
                        cwe_metrics[model][cwe]["fp"] += 1
                    elif in_gt and not in_det:
                        cwe_metrics[model][cwe]["fn"] += 1

    f1_data = {}
    for model in detail_reports:
        f1_data[model] = {}
        for cwe in CWE_LABELS:
            m = cwe_metrics[model][cwe]
            tp, fp, fn = m["tp"], m["fp"], m["fn"]
            precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
            recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
            f1_data[model][cwe] = f1_calculate(precision, recall)

    df_f1 = pd.DataFrame(f1_data).T
    if not HEATMAP_X_IS_CWE:
        df_f1 = df_f1.T

    plt.figure(figsize=(8, 6))
    sns.heatmap(df_f1, annot=True, fmt=".2f", cmap="YlOrRd", linewidths=0.5, vmin=0, vmax=1)
    plt.tight_layout()
    out = RESULTS_DIR.parent / "rq3_heatmap_f1.pdf"
    plt.savefig(out, format="pdf", bbox_inches="tight", pad_inches=0)
    print(f"Saved {out}")
    plt.close()


# ── Plot 5: Temporal F1 ─────────────────────────────────────────────────────

def plot_temporal_f1(detail_reports: Dict[str, Report]) -> None:
    print("Extracting data for F1 Temporal Analysis...")
    audit_date_cache: Dict[str, Optional[datetime]] = {}

    def resolve_finding_path(report_path: str) -> Optional[Path]:
        """Resolve a report_path from result JSON to the actual findings file."""
        p = REPO_ROOT / report_path
        if p.exists():
            return p
        # Handle renamed files: .pdf.json → .json, .md.json → .json
        for ext in [".pdf.json", ".md.json"]:
            if p.name.endswith(ext):
                fixed = p.parent / (p.name[: -len(ext)] + ".json")
                if fixed.exists():
                    return fixed
        return None

    def get_audit_date(report_path: str) -> Optional[datetime]:
        if report_path in audit_date_cache:
            return audit_date_cache[report_path]
        result = None
        resolved = resolve_finding_path(report_path)
        if resolved:
            try:
                with open(resolved) as f:
                    data = json.load(f)
                    date_str = data.get("project_info", {}).get("audit_date")
                    if date_str:
                        result = datetime.strptime(date_str, "%Y-%m-%d")
            except Exception:
                pass
        audit_date_cache[report_path] = result
        return result

    temporal_data = []
    for model, report in detail_reports.items():
        for rec in report.record:
            d_date = get_audit_date(rec.report_path)
            if d_date:
                temporal_data.append({
                    "Model": model,
                    "Date": d_date,
                    "TP": rec.tp_size,
                    "Total_GT": rec.t_size,
                    "Total_P": rec.p_size,
                })

    df = pd.DataFrame(temporal_data)
    if df.empty:
        print("No temporal data extracted.")
        return

    df.set_index("Date", inplace=True)
    df_resampled = df.groupby("Model").resample("QE").sum(numeric_only=True)

    MIN_GT_THRESHOLD = 3

    def calculate_f1_period(row):
        tp, gt, p = row["TP"], row["Total_GT"], row["Total_P"]
        if gt < MIN_GT_THRESHOLD:
            return None
        precision = tp / p if p > 0 else 0.0
        recall = tp / gt if gt > 0 else 0.0
        if (precision + recall) == 0:
            return 0.0
        return 2 * (precision * recall) / (precision + recall)

    df_resampled["F1"] = df_resampled.apply(calculate_f1_period, axis=1)
    df_resampled = df_resampled.reset_index()

    print("Plotting F1 Temporal Analysis...")
    plt.figure(figsize=(10, 4))

    models_list = df_resampled["Model"].unique()
    palette = sns.color_palette("tab10", len(models_list))
    color_map = dict(zip(models_list, palette))

    for model in models_list:
        subset = df_resampled[(df_resampled["Model"] == model) & (df_resampled["F1"].notna())].sort_values("Date")
        if subset.empty:
            continue
        color = color_map[model]
        plt.scatter(subset["Date"], subset["F1"], color=color, s=50, zorder=5)
        plt.plot(subset["Date"], subset["F1"], color=color, linewidth=2.5, alpha=0.8, label=model)

        if len(subset) > 1:
            x_nums = mdates.date2num(subset["Date"])
            z = np.polyfit(x_nums, subset["F1"].values, 1)
            p = np.poly1d(z)
            plt.plot(subset["Date"], p(x_nums), linestyle="--", linewidth=1.5, alpha=0.6, color=color)

    plt.ylabel("F1 Score")
    plt.xlabel("Audit Date")
    plt.ylim(0, 1.05)
    plt.grid(True, linestyle="--", alpha=0.7)
    plt.legend(title="Model", loc="best")
    plt.tight_layout()
    out = RESULTS_DIR.parent / "rq3_temporal_f1.pdf"
    plt.savefig(out, format="pdf", bbox_inches="tight", pad_inches=0)
    print(f"Saved {out}")
    plt.close()


# ── Main ─────────────────────────────────────────────────────────────────────

def main() -> None:
    plt.switch_backend("Agg")
    detail_reports = load_reports()
    if not detail_reports:
        return
    print_summary(detail_reports)
    plot_performance(detail_reports)
    plot_heatmaps(detail_reports)
    plot_cwe_f1_heatmap(detail_reports)
    plot_temporal_f1(detail_reports)
    print("\nDone. All PDFs saved next to analyze.py.")


if __name__ == "__main__":
    main()
