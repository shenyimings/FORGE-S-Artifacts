import argparse
import time
import os
from pathlib import Path
from typing import List
from dotenv import load_dotenv

from model import DetectorModel, ProjectRecord, CWEHandler, VulnPair
from reasoner import HierarchicalAuditor
from processor import process_reports
from dataclasses import asdict
import json


def main():
    parser = argparse.ArgumentParser(description="Solana Security Auditor CLI")
    parser.add_argument(
        "--findings-path",
        type=str,
        required=True,
        help="Path to findings directory or file",
    )
    parser.add_argument(
        "--detector-path", type=str, required=True, help="Path to detector history file"
    )
    parser.add_argument(
        "--cwe-db-path", type=str, required=True, help="Path to CWE database file"
    )
    parser.add_argument(
        "--dataset-path", type=str, default="dataset/", help="Path to dataset directory"
    )
    parser.add_argument("--model", type=str, default="qwen3", help="Model name to use")

    args = parser.parse_args()

    # Load Detector
    detector = DetectorModel.load(args.detector_path)
    # print(f"Loaded detector history with {len(detector.history)} records.")

    # Load CWE Handler
    cwe_handler = CWEHandler.load_from_file(args.cwe_db_path)
    # print("Loaded CWE database.")

    # Initialize Auditor
    auditor = HierarchicalAuditor(cwe_handler, model_name=args.model)

    # Get list of files to process
    findings_path = Path(args.findings_path)
    if findings_path.is_file():
        files = [findings_path]
    else:
        files = sorted(list(findings_path.glob("*.json")))

    print(f"Found {len(files)} finding files to process.")

    processed_paths = {r.report_path for r in detector.history}

    for file_path in files:
        abs_path = str(file_path.resolve())
        if abs_path in processed_paths:
            print(f"Skipping {file_path.name} (already processed).")
            continue

        print(f"Processing {file_path.name}...")
        start_time = time.time()

        try:
            vuln_pairs = process_reports(file_path, dataset_path=args.dataset_path)
        except Exception as e:
            print(f"Error processing report {file_path}: {e}")
            continue

        if not vuln_pairs:
            print(f"No valid vuln pairs found in {file_path.name}.")
            # Record as processed but empty
            record = ProjectRecord(
                project_name=file_path.stem,
                report_path=abs_path,
                scan_time=time.time() - start_time,
            )
            detector.add_record(record)
            detector.save(args.detector_path)
            continue

        # Audit each pair
        file_prompt_tokens = 0
        file_completion_tokens = 0
        file_total_tokens = 0

        # Metrics accumulators for the file (Hierarchical)
        file_hier_p_num = 0
        file_hier_p_den = 0
        file_hier_r_num = 0
        file_hier_r_den = 0


        for vp in vuln_pairs:
            # Audit
            vp, usage = auditor.audit(vp)

            # Accumulate tokens
            file_prompt_tokens += usage.get("prompt_tokens", 0)
            file_completion_tokens += usage.get("completion_tokens", 0)
            file_total_tokens += usage.get("total_tokens", 0)

            # Judge
            vp.judge_detection(cwe_handler)

            # save to json file
            filename = vp.gen_pair_id()
            model_id = args.model.replace("/", "-")
            if not os.path.exists(f"evaluation/rq3/evaluator/output/pairs/{model_id}"):
                os.makedirs(
                    f"evaluation/rq3/evaluator/output/pairs/{model_id}", exist_ok=True
                )

            with open(
                os.path.join(
                    f"evaluation/rq3/evaluator/output/pairs/{model_id}",
                    f"{filename}.json",
                ),
                "w",
            ) as f:

                # f.write()
                # json.dump(vp.to_json(), f, ensure_ascii=False, indent=4, default=str)
                f.write(vp.to_json())

            # Collect Hierarchical Metrics Data (directly sum from vp)
            # print(
            #     f"VulnPair {filename} - Hier P Num: {vp.hier_tp_size}, Hier P Den: {vp.hier_p_size}, Hier R Den: {vp.hier_r_size}"
            # )
            file_hier_p_num += vp.hier_tp_size
            file_hier_p_den += vp.hier_p_size
            file_hier_r_num += vp.hier_tp_size
            file_hier_r_den += vp.hier_r_size

            # all_tfnp_leaf.append(vp.tfnp_leaf)
        print(
            f"tp_size: {file_hier_p_num}, p_size: {file_hier_p_den}, r_size: {file_hier_r_den}"
        )
        end_time = time.time()
        duration = end_time - start_time

        def merge_tfnp(list_of_tuples):
            total_tp = {}
            total_fp = {}
            total_fn = {}

            for tp, fp, fn in list_of_tuples:
                for k, v in tp.items():
                    total_tp[k] = total_tp.get(k, 0) + v
                for k, v in fp.items():
                    total_fp[k] = total_fp.get(k, 0) + v
                for k, v in fn.items():
                    total_fn[k] = total_fn.get(k, 0) + v

            return total_tp, total_fp, total_fn

        # merged_leaf = merge_tfnp(all_tfnp_leaf)

        # # Calculate aggregate leaf metrics
        # tp_leaf, fp_leaf, fn_leaf = merged_leaf
        # file_tp = sum(tp_leaf.values())
        # file_fp = sum(fp_leaf.values())
        # file_fn = sum(fn_leaf.values())

        # Calculate P/R/F1 for this file (Leaf)
        # precision = file_tp / (file_tp + file_fp) if (file_tp + file_fp) > 0 else 0.0
        # recall = file_tp / (file_tp + file_fn) if (file_tp + file_fn) > 0 else 0.0
        # f1 = (
        #     2 * (precision * recall) / (precision + recall)
        #     if (precision + recall) > 0
        #     else 0.0
        # )

        # Calculate Hierarchical Metrics for this file
        h_prec = file_hier_p_num / file_hier_p_den if file_hier_p_den > 0 else 0.0
        h_rec = file_hier_r_num / file_hier_r_den if file_hier_r_den > 0 else 0.0
        h_f1 = 2 * (h_prec * h_rec) / (h_prec + h_rec) if (h_prec + h_rec) > 0 else 0.0

        record = ProjectRecord(
            project_name=vuln_pairs[0].project if vuln_pairs else file_path.stem,
            report_path=abs_path,
            scan_time=duration,
            prompt_tokens=file_prompt_tokens,
            completion_tokens=file_completion_tokens,
            total_tokens=file_total_tokens,
            hierarchical_precision=h_prec,
            hierarchical_recall=h_rec,
            hierarchical_f1=h_f1,
            hier_tp_size=file_hier_p_num,  # Intersection size (Wait, p_num is intersection)
            hier_p_size=file_hier_p_den,
            hier_r_size=file_hier_r_den,
        )

        detector.add_record(record)
        detector.save(args.detector_path)

        print(f"Finished {file_path.name}. Time: {duration:.2f}s. Hier F1: {h_f1:.4f}")


if __name__ == "__main__":
    load_dotenv()
    main()


"""
python evaluation/rq3/evaluator/main.py \
  --findings-path dataset/findings/ \
  --detector-path evaluation/rq3/evaluator/detector_history.json \
  --cwe-db-path evaluation/rq3/evaluator/cwe_dict.json \
  --dataset-path dataset/ \
  --model qwen3
"""

"""
python evaluation/rq3/evaluator/main.py \
  --findings-path "dataset/findings/Sec3_2023.pdf.json" \
  --detector-path evaluation/rq3/evaluator/gpt52.json \
  --cwe-db-path evaluation/rq3/evaluator/cwe_dict.json \
  --dataset-path dataset/ \
  --model gpt-5.2
"""
