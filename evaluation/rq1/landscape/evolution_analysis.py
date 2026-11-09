# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "pydantic>=2.0",
#     "pandas>=2.0",
#     "plotly>=5.0",
#     "kaleido",
# ]
# ///
"""Temporal evolution of Solana vulnerability disclosures vs. exploit incidents."""

from __future__ import annotations

from collections import namedtuple
from pathlib import Path

import pandas as pd
import plotly.express as px
import plotly.graph_objects as go

from landscape import JSONFileProcessor, Report

SCRIPT_DIR = Path(__file__).parent
DATASET_DIR = SCRIPT_DIR.parent.parent.parent / "dataset"

EXPLOIT_INCIDENTS = {
    "Wormhole Bridge": ("2022-02", 326, 1.0),
    "Cashio Stablecoin": ("2022-03", 52, 0.9),
    "Crema Finance": ("2022-07", 8.78, 1.0),
    "Nirvana Finance": ("2022-07", 3.5, 0.9),
    "OptiFi": ("2022-08", 0.661, 0.8),
    "Mango Markets": ("2022-10", 110, 1.0),
    "Tulip Protocol": ("2022-10", 2.5, 0.8),
    "UXD Protocol": ("2022-10", 20, 0.7),
    "Solend Oracle": ("2022-11", 1.26, 0.65),
    "Raydium DEX": ("2022-12", 5.5, 0.9),
    "Cypher Protocol": ("2023-08", 1, 1.0),
    "Loopscale Lending": ("2024-04", 5.8, 1.0),
    "Pump.fun DEX": ("2024-05", 2, 0.9),
    "DEXX Memecoin": ("2024-11", 30, 1.0),
    "Drift Protocol": ("2026-04", 285, 1.0),
}


def _is_valid_date(audit_date) -> bool:
    if audit_date is None or audit_date == "n/a":
        return False
    if isinstance(audit_date, (int, list)):
        return False
    ts = pd.to_datetime(audit_date, errors="coerce")
    return ts is not pd.NaT and not pd.isna(ts)


def temporal_evolution():
    print("\n=== Temporal Evolution ===")

    CWE_tuple = namedtuple("CWE", ["cwe_ids"])

    skipped = 0

    def extract_pillar_cwe_id_dates(report: Report):
        nonlocal skipped
        cwe_ids = []
        audit_date = report.project_info.audit_date
        if not _is_valid_date(audit_date):
            n = len(report.findings)
            if n:
                skipped += n
                print(f"  skipped {n} findings (invalid date) in {report.path}")
            return CWE_tuple(cwe_ids=[])
        for finding in report.findings:
            if finding.category:
                try:
                    cwe_value = finding.category.get("1", [""])[0]
                    if isinstance(cwe_value, str) and cwe_value.startswith("CWE-"):
                        cwe_ids.append((cwe_value, audit_date))
                except (ValueError, AttributeError):
                    continue
        return CWE_tuple(cwe_ids=cwe_ids)

    processor = JSONFileProcessor(str(DATASET_DIR / "findings"))
    time_results = processor.process_files(analysis_func=extract_pillar_cwe_id_dates)
    cwe_time_result = processor.operate_reduce(time_results, CWE_tuple)

    if skipped:
        print(f"  total findings skipped due to invalid dates: {skipped}")

    cwe_df = pd.DataFrame(cwe_time_result["cwe_ids"], columns=["cwe_id", "date"])
    cwe_df["date"] = pd.to_datetime(cwe_df["date"], errors="coerce")
    cwe_df = cwe_df.dropna(subset=["date"])

    cwe_df["quarter"] = cwe_df["date"].dt.to_period("Q").dt.start_time
    cwe_df_agg = cwe_df.groupby(["quarter", "cwe_id"]).size().reset_index(name="count")
    cwe_df_agg["total_quarterly_count"] = cwe_df_agg.groupby("quarter")[
        "count"
    ].transform("sum")
    cwe_df_agg["percentage"] = (
        cwe_df_agg["count"] / cwe_df_agg["total_quarterly_count"]
    ) * 100
    cwe_df_agg = cwe_df_agg.sort_values(by="quarter")
    cwe_df_agg = cwe_df_agg.dropna(subset=["quarter"])
    cwe_df_agg = cwe_df_agg[cwe_df_agg["quarter"] < "2026-05-01"]
    cwe_df_agg["quarter"] = cwe_df_agg["quarter"].dt.strftime("%Y-%m-%d")

    print(
        f"  quarters covered: {cwe_df_agg['quarter'].min()} – {cwe_df_agg['quarter'].max()}"
    )
    print(f"  total data points: {len(cwe_df_agg)}")

    exploit_df = pd.DataFrame(
        [
            {"incident": name, "date_str": date, "loss": loss, "y_pos": y_pos}
            for name, (date, loss, y_pos) in EXPLOIT_INCIDENTS.items()
        ]
    )
    exploit_df["date"] = pd.to_datetime(exploit_df["date_str"]).dt.strftime("%Y-%m-%d")
    exploit_df = exploit_df.sort_values(by=["date", "loss"], ascending=[True, False])

    fig1 = px.area(
        cwe_df_agg,
        x="quarter",
        y="count",
        color="cwe_id",
        line_shape="spline",
        labels={
            "quarter": "Quarter",
            "count": "Number of Disclosed Vulnerabilities",
            "cwe_id": "CWE Pillar",
        },
        color_discrete_sequence=px.colors.qualitative.Prism,
    )
    fig1.update_layout(
        yaxis=dict(showline=True, linewidth=1.4, linecolor="black"),
        yaxis2=dict(
            title="Incident Loss (Million $)",
            overlaying="y",
            side="right",
            showgrid=False,
        ),
        plot_bgcolor="white",
        xaxis=dict(
            tickformat="%Y-Q%q",
            dtick="M3",
            showline=True,
            linewidth=1.4,
            linecolor="black",
        ),
        xaxis_showgrid=False,
        yaxis_showgrid=False,
        xaxis_title="",
        legend=dict(yanchor="top", y=1, xanchor="left", x=1.04),
    )

    for _, row in exploit_df.iterrows():
        fig1.add_trace(
            go.Scatter(
                x=[row["date"]],
                y=[row["loss"]],
                mode="markers",
                marker=dict(
                    symbol="circle-open",
                    size=6,
                    color="indianred",
                    line=dict(width=1.5),
                ),
                name=f"{row['incident']} (${row['loss']}M)",
                yaxis="y2",
                hoverinfo="text",
                hovertext=f"{row['incident']}: ${row['loss']}M",
                showlegend=False,
            )
        )
        fig1.add_vline(
            x=row["date"],
            line_width=1,
            line_dash="dash",
            line_color="silver",
            layer="below",
        )
        fig1.add_annotation(
            x=row["date"],
            y=row["y_pos"],
            yref="paper",
            text=f"{row['incident']}<br>${row['loss']}M",
            showarrow=False,
            font=dict(size=11, color="grey"),
            xanchor="left",
            yanchor="bottom",
        )

    fig2 = px.area(
        cwe_df_agg,
        x="quarter",
        y="percentage",
        color="cwe_id",
        line_shape="spline",
        labels={
            "quarter": "Quarter",
            "percentage": "Percentage of Vulnerabilities (%)",
            "cwe_id": "CWE Pillar",
        },
        hover_data={"count": True, "percentage": ":.2f"},
        color_discrete_sequence=px.colors.qualitative.T10,
    )
    fig2.update_layout(yaxis_ticksuffix="%", xaxis_title="")

    out = SCRIPT_DIR / "vulnerability_evolution.html"
    fig1.write_html(str(out))
    print(f"Saved {out}")

    import copy

    fig_pdf = copy.deepcopy(fig1)
    for trace in fig_pdf.data:
        if hasattr(trace, "name") and trace.name and "435" in trace.name:
            trace.showlegend = False
    for i, trace in enumerate(fig_pdf.data):
        if hasattr(trace, "legendrank"):
            trace.legendrank = 1000 + i
    fig_pdf.add_trace(
        go.Scatter(
            x=[None],
            y=[None],
            mode="markers",
            marker=dict(
                symbol="circle-open", size=6, color="indianred", line=dict(width=1.5)
            ),
            name="Incidents",
            yaxis="y2",
            showlegend=True,
            legendrank=0,
        )
    )
    fig_pdf.update_layout(
        legend=dict(
            yanchor="top",
            y=0.98,
            xanchor="left",
            x=0.01,
            bgcolor="rgba(255,255,255,0.6)",
            borderwidth=0,
            font=dict(size=7),
            tracegroupgap=0,
            itemwidth=30,
            title=None,
        ),
        margin=dict(t=4, l=4, r=4, b=24),
        font=dict(size=8),
        yaxis=dict(title=dict(font=dict(size=8), standoff=2), tickfont=dict(size=7)),
        yaxis2=dict(title=dict(font=dict(size=8), standoff=2), tickfont=dict(size=7)),
        xaxis=dict(tickfont=dict(size=7), dtick="M12", tickformat="%Y"),
    )
    fig_pdf.layout.annotations = []
    pdf_path = SCRIPT_DIR / "vulnerability_evolution.pdf"
    fig_pdf.write_image(str(pdf_path), width=460, height=200, scale=2)
    print(f"Saved {pdf_path} (compact)")


if __name__ == "__main__":
    temporal_evolution()
