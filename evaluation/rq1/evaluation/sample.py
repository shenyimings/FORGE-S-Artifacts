#!/usr/bin/env python3
import os
import sys
import json
import random
import shutil
import argparse
from pathlib import Path
import yaml
from collections import defaultdict

sys.path.append(str(Path(__file__).resolve().parents[2]))

DATASET_ROOT = Path("dataset")
DATASET_FINDINGS = DATASET_ROOT / "findings"
DATASET_REPORTS = DATASET_ROOT / "reports"
DATASET_CONTRACTS = DATASET_ROOT / "contracts"

SAMPLE_ROOT = Path("evaluation/rq1/sample")
SAMPLE_FINDINGS = SAMPLE_ROOT / "findings"
SAMPLE_REPORTS = SAMPLE_ROOT / "reports"
SAMPLE_CONTRACTS = SAMPLE_ROOT / "contracts"

ANNOTATION_PROJECTS = SAMPLE_ROOT / "annotation_projects.yaml"
ANNOTATION_FINDINGS = SAMPLE_ROOT / "annotation_findings.yaml"


class FlowStyleList(list):
    pass


def represent_flow_style_list(dumper, data):
    return dumper.represent_list(data)


yaml.add_representer(FlowStyleList, represent_flow_style_list)


def setup_directories():
    SAMPLE_FINDINGS.mkdir(parents=True, exist_ok=True)
    SAMPLE_REPORTS.mkdir(parents=True, exist_ok=True)
    SAMPLE_CONTRACTS.mkdir(parents=True, exist_ok=True)


def sample_projects(num_samples=65, seed=42):
    """Stage 1: Sample project files for information extraction evaluation"""
    print(
        f"=== Stage 1: Sampling {num_samples} projects for information extraction evaluation ==="
    )
    random.seed(seed)
    setup_directories()

    findings_files = list(DATASET_FINDINGS.glob("*.json"))
    if len(findings_files) < num_samples:
        print(
            f"Warning: Available files ({len(findings_files)}) less than requested samples ({num_samples})"
        )
        num_samples = len(findings_files)

    sampled_files = random.sample(findings_files, num_samples)

    annotations = {
        "summary": {
            "total_samples": num_samples,
            "metrics": {
                "information_extraction": {
                    "url": [0, 0, 0],
                    "commit_id": [0, 0, 0],
                    "audit_date": [0, 0, 0],
                    "vuln_info": [0, 0, 0],
                    "severity": [0, 0, 0],
                }
            },
        },
        "samples": [],
    }

    for finding_file in sampled_files:
        dst_finding = SAMPLE_FINDINGS / finding_file.name
        shutil.copy2(finding_file, dst_finding)

        with open(finding_file, "r", encoding="utf-8") as f:
            finding_data = json.load(f)

        report_path = finding_data.get("path", "")
        if report_path:
            src_report = DATASET_ROOT / report_path
            if src_report.exists():
                dst_report = SAMPLE_ROOT / report_path
                dst_report.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src_report, dst_report)

        sample_entry = {
            "filename": finding_file.name,
            "report_path": report_path,
            "information_extraction": {
                "url": [0, 0, 0],
                "commit_id": [0, 0, 0],
                "audit_date": [0, 0, 0],
                "vuln_info": [0, 0, 0],
                "severity": [0, 0, 0],
            },
        }
        annotations["samples"].append(sample_entry)

    with open(ANNOTATION_PROJECTS, "w", encoding="utf-8") as f:
        yaml.dump(
            annotations, f, allow_unicode=True, default_flow_style=None, sort_keys=False
        )

    print(f"✓ Project sampling completed! Generated file: {ANNOTATION_PROJECTS}")


def sample_individual_findings(num_samples=91, seed=42):
    """Stage 2: Sample individual findings for classification and location evaluation"""
    print(
        f"=== Stage 2: Sampling {num_samples} findings for classification and location evaluation ==="
    )
    random.seed(seed)
    setup_directories()

    all_findings_pool = []

    findings_files = list(DATASET_FINDINGS.glob("*.json"))
    for f_path in findings_files:
        with open(f_path, "r", encoding="utf-8") as f:
            data = json.load(f)
            findings_list = data.get("findings", [])
            for finding in findings_list:
                all_findings_pool.append(
                    {"file_path": f_path, "finding": finding, "project_data": data}
                )

    if len(all_findings_pool) < num_samples:
        print(
            f"Warning: Available findings ({len(all_findings_pool)}) less than requested samples ({num_samples})"
        )
        num_samples = len(all_findings_pool)

    sampled_items = random.sample(all_findings_pool, num_samples)

    grouped_samples = defaultdict(list)
    for item in sampled_items:
        grouped_samples[item["file_path"]].append(item)

    annotations = {
        "summary": {
            "total_samples": num_samples,
            "metrics": {
                "category_classification": {
                    "total_items": 0,
                    "krippendorff_alpha": None,
                },
                "location_detection": {"TP": 0, "FP": 0, "FN": 0, "TN": 0},
            },
        },
        "samples": [],
    }

    for f_path, items in grouped_samples.items():
        project_data = items[0]["project_data"]

        dst_finding = SAMPLE_FINDINGS / f_path.name
        shutil.copy2(f_path, dst_finding)

        report_path = project_data.get("path", "")
        if report_path:
            src_report = DATASET_ROOT / report_path
            if src_report.exists():
                dst_report = SAMPLE_ROOT / report_path
                dst_report.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src_report, dst_report)

        project_path = project_data.get("project_info", {}).get("project_path", "")
        if project_path:
            src_contract_dir = DATASET_ROOT / project_path
            if src_contract_dir.exists():
                dst_contract_dir = SAMPLE_ROOT / project_path
                if src_contract_dir.is_dir():
                    shutil.copytree(
                        src_contract_dir, dst_contract_dir, dirs_exist_ok=True
                    )
                else:
                    dst_contract_dir.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(src_contract_dir, dst_contract_dir)

        sample_entry = {
            "filename": f_path.name,
            "report_path": report_path,
            "contract_path": project_path,
            "findings": [],
        }

        for item in items:
            finding = item["finding"]
            category = finding.get("category", {})
            category = category.values() if isinstance(category, dict) else []
            category = [cate[0] for cate in category]

            finding_annotation = {
                "finding_id": finding.get("id", 0),
                "finding_title": finding.get("title", ""),
                "finding_description": finding.get("description", ""),
                "finding_files": finding.get("files", []),
                "category": [
                    FlowStyleList(category),
                    FlowStyleList([]),
                    FlowStyleList([]),
                ],
                "location": [0, 0, 0, 0],
            }
            sample_entry["findings"].append(finding_annotation)

        annotations["samples"].append(sample_entry)

    with open(ANNOTATION_FINDINGS, "w", encoding="utf-8") as f:
        yaml.dump(
            annotations, f, allow_unicode=True, default_flow_style=None, sort_keys=False
        )
    print(f"✓ Findings sampling completed! Generated file: {ANNOTATION_FINDINGS}")


def calculate_metrics(tp, fp, fn):
    """Calculate precision, recall, f1"""
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    f1 = (
        2 * precision * recall / (precision + recall)
        if (precision + recall) > 0
        else 0.0
    )
    return precision, recall, f1


def summary_annotations(annotation_file):
    """Auto-select summary calculation based on file type"""
    print(f"Reading annotation file: {annotation_file}")
    with open(annotation_file, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f)

    metrics_config = data.get("summary", {}).get("metrics", {})

    if "information_extraction" in metrics_config:
        _summary_project_ie(data, annotation_file, metrics_config)
    elif "category_classification" in metrics_config:
        _summary_finding_classification(data, annotation_file, metrics_config)
    else:
        print("Error: Unrecognized annotation file format")


def _summary_project_ie(data, annotation_file, metrics_config):
    """Calculate information extraction metrics"""
    ie_metrics = {
        "url": [0, 0, 0],
        "commit_id": [0, 0, 0],
        "audit_date": [0, 0, 0],
        "vuln_info": [0, 0, 0],
        "severity": [0, 0, 0],
    }

    for sample in data.get("samples", []):
        for key in ie_metrics:
            ie_values = sample.get("information_extraction", {}).get(key, [0, 0, 0])
            for i in range(3):
                ie_metrics[key][i] += ie_values[i]

    print("\n=== 信息提取准确性 (Information Extraction) ===")
    ie_summary = {}
    for key, values in ie_metrics.items():
        tp, pred_total, gt_total = values
        fp = pred_total - tp
        fn = gt_total - tp
        precision, recall, f1 = calculate_metrics(tp, fp, fn)
        ie_summary[key] = {
            "TP": tp,
            "FP": fp,
            "FN": fn,
            "precision": round(precision, 4),
            "recall": round(recall, 4),
            "f1": round(f1, 4),
        }
        print(
            f"{key:15s}: P={precision:.4f}, R={recall:.4f}, F1={f1:.4f} (correct entities={tp}, wrong entities={fp}, omitted entities={fn})"
        )

    data["summary"]["metrics"]["information_extraction"] = ie_summary

    macro_metrics = {"precision": 0.0, "recall": 0.0, "f1": 0.0}
    for metric in ["precision", "recall", "f1"]:
        macro_metrics[metric] = round(
            sum(ie_summary[key][metric] for key in ie_summary) / len(ie_summary), 4
        )
    data["summary"]["metrics"]["information_extraction_macro"] = macro_metrics
    print(
        f"\Marco: P={macro_metrics['precision']:.4f}, R={macro_metrics['recall']:.4f}, F1={macro_metrics['f1']:.4f}"
    )
    with open(annotation_file, "w", encoding="utf-8") as f:
        yaml.dump(data, f, allow_unicode=True, default_flow_style=None, sort_keys=False)
    print(f"\n✓ Summary updated to: {annotation_file}")


def _summary_finding_classification(data, annotation_file, metrics_config):
    """Calculate classification and location metrics"""
    location_metrics = {"TP": 0, "FP": 0, "FN": 0, "TN": 0}
    category_data = []

    for sample in data.get("samples", []):
        for finding in sample.get("findings", []):
            category = finding.get("category", [[], [], []])
            if len(category) >= 3:
                original = category[0]
                annotator_a = category[1]
                annotator_b = category[2]
                category_data.append((original, annotator_a, annotator_b))

            location = finding.get("location", [0, 0, 0, 0])
            location_metrics["TP"] += location[0]
            location_metrics["FP"] += location[1]
            location_metrics["FN"] += location[2]
            location_metrics["TN"] += location[3]

    print("\n=== Location Detection Accuracy ===")
    loc_tp = location_metrics["TP"]
    loc_fp = location_metrics["FP"]
    loc_fn = location_metrics["FN"]
    loc_tn = location_metrics["TN"]
    precision, recall, f1 = calculate_metrics(loc_tp, loc_fp, loc_fn)
    print(f"Precision: {precision:.4f}")
    print(f"Recall:    {recall:.4f}")
    print(f"F1-Score:  {f1:.4f}")

    location_summary = {
        "TP": loc_tp,
        "FP": loc_fp,
        "FN": loc_fn,
        "TN": loc_tn,
        "precision": round(precision, 4),
        "recall": round(recall, 4),
        "f1": round(f1, 4),
    }

    print("\n=== Category Classification Consistency ===")
    print(f"Annotated samples: {len(category_data)}")

    csv_file = Path(annotation_file).parent / "category_agreement.csv"
    with open(csv_file, "w", encoding="utf-8") as f:
        for original, annotator_a, annotator_b in category_data:

            def to_str(val):
                if isinstance(val, list):
                    if val:
                        return str(val[-1]).strip("CWE-")
                return ""

            f.write(
                f"{to_str(original)}, {to_str(annotator_a)}, {to_str(annotator_b)}\n"
            )

    print(f"✓ Classification consistency data exported: {csv_file}")
    data["summary"]["metrics"]["location_detection"] = location_summary
    data["summary"]["metrics"]["category_classification"] = {
        "total_items": len(category_data),
        "krippendorff_alpha": None,
    }

    with open(annotation_file, "w", encoding="utf-8") as f:
        yaml.dump(data, f, allow_unicode=True, default_flow_style=None, sort_keys=False)
    print(f"\n✓ Summary updated to: {annotation_file}")


def main():
    parser = argparse.ArgumentParser(
        description="Dataset random sampling and evaluation tool"
    )
    subparsers = parser.add_subparsers(dest="command", help="Subcommands")

    sample_parser = subparsers.add_parser("sample", help="Random sampling")
    sample_parser.add_argument(
        "--stage",
        type=int,
        choices=[1, 2],
        help="Sampling stage: 1=Projects(IE), 2=Findings(Cat/Loc). Default runs both",
    )
    sample_parser.add_argument(
        "-np",
        "--num-projects",
        type=int,
        default=65,
        help="Stage 1 project sample count (default: 65)",
    )
    sample_parser.add_argument(
        "-nf",
        "--num-findings",
        type=int,
        default=91,
        help="Stage 2 findings sample count (default: 91)",
    )
    sample_parser.add_argument(
        "-s", "--seed", type=int, default=42, help="Random seed (default: 42)"
    )

    summary_parser = subparsers.add_parser(
        "summary", help="Calculate annotation statistics summary"
    )
    summary_parser.add_argument(
        "annotation_file", type=str, help="Path to annotation.yaml file"
    )

    args = parser.parse_args()

    if args.command == "sample":
        if args.stage is None or args.stage == 1:
            sample_projects(num_samples=args.num_projects, seed=args.seed)
        if args.stage is None or args.stage == 2:
            sample_individual_findings(num_samples=args.num_findings, seed=args.seed)

    elif args.command == "summary":
        summary_annotations(args.annotation_file)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
