import os
import json
import sys
import argparse
import yaml
from pathlib import Path
from typing import List, Dict
from dotenv import load_dotenv

# Add current directory to sys.path to import sibling modules
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from react import ReActAgent, generate_audit_task
from tool import ToolRegistry, ListDirectoryTool, ReadFileTool, SearchFileTool


def process_findings_file(file_path: Path, dataset_root: Path, agent: ReActAgent):
    print(f"Processing {file_path}...")

    try:
        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except json.JSONDecodeError:
        print(f"Error: Failed to decode JSON from {file_path}")
        return

    project_info = data.get("project_info", {})
    project_path_str = project_info.get("project_path")

    if not project_path_str:
        print(f"Skipping {file_path}: No project_path found.")
        return

    # Construct absolute project path
    # Assuming project_path in JSON is relative to dataset_root
    # We resolve it to be absolute or relative to CWD
    project_full_path = dataset_root / project_path_str

    if not project_full_path.exists():
        print(f"Error: Project path {project_full_path} does not exist.")
        return

    findings = data.get("findings", [])
    missing_findings = []

    for finding in findings:
        files = finding.get("files", [])

        # If files list is empty, we treat it as missing/needs fixing
        if not files:
            print(f"Finding {finding.get('id')} has no files.")
            missing_findings.append(finding)
            continue

        files_ok = True
        for file_rel_path in files:
            # The file path in 'files' should be relative to the project root
            file_abs_path = project_full_path / file_rel_path
            if not file_abs_path.exists():
                print(f"File not found: {file_abs_path}")
                files_ok = False

        if not files_ok:
            missing_findings.append(finding)

    if missing_findings:
        print(
            f"Found {len(missing_findings)} findings with missing/incorrect files. Invoking Agent..."
        )

        # Update whitelist to restrict agent to the project directory
        agent.registry.set_whitelist([str(project_full_path)])

        # Generate task
        # We pass the project_full_path so the agent knows where to search
        task = generate_audit_task(str(project_full_path), missing_findings)

        # Run Agent
        result_yaml = agent.run(task)

        try:
            # Parse result
            updates = yaml.safe_load(result_yaml)

            if isinstance(updates, dict):
                updated_count = 0
                for finding in findings:
                    f_id = finding.get("id")
                    # Handle both int and string keys
                    new_files = None
                    if f_id in updates:
                        new_files = updates[f_id]
                    elif str(f_id) in updates:
                        new_files = updates[str(f_id)]

                    if new_files is not None:
                        # Ensure it's a list
                        if isinstance(new_files, str):
                            new_files = [new_files]

                        if new_files:
                            cleaned_files = []
                            for nf in new_files:
                                # Normalize paths for comparison
                                nf_path = Path(nf)
                                try:
                                    # Try to make relative to project root if it's an absolute path or prefixed
                                    rel_path = nf_path.relative_to(project_full_path)
                                    cleaned_files.append(str(rel_path))
                                except ValueError:
                                    # If not relative to project_full_path, keep as is (or handle differently)
                                    cleaned_files.append(nf)

                            print(f"Updating finding {f_id} files to: {cleaned_files}")
                            finding["files"] = cleaned_files
                            updated_count += 1

                if updated_count > 0:
                    # Save updated JSON
                    with open(file_path, "w", encoding="utf-8") as f:
                        json.dump(data, f, indent=4, ensure_ascii=False)
                    print(f"Saved {updated_count} updates to {file_path}")
                else:
                    print(
                        "Agent returned results but no matching IDs found or empty lists."
                    )
            else:
                print(f"Agent returned unexpected format (not a dict): {type(updates)}")

        except Exception as e:
            print(f"Failed to parse agent output or update findings: {e}")
            print(f"Raw output: {result_yaml}")
    else:
        print("All files exist. No action needed.")


def main():
    load_dotenv()
    parser = argparse.ArgumentParser(description="Audit Finding Location Fixer")
    parser.add_argument(
        "--findings_dir",
        type=str,
        default="dataset/findings",
        help="Directory containing finding JSONs",
    )
    parser.add_argument(
        "--dataset_root",
        type=str,
        default="dataset",
        help="Root directory for datasets",
    )
    parser.add_argument(
        "--model", type=str, default=None, help="Model to use for the agent (default: llm.model in config.yaml)"
    )

    args = parser.parse_args()

    findings_dir = Path(args.findings_dir)
    dataset_root = Path(args.dataset_root)

    if not findings_dir.exists():
        print(f"Findings directory {findings_dir} does not exist.")
        return

    # Initialize Agent
    registry = ToolRegistry()
    registry.register(ListDirectoryTool())
    registry.register(ReadFileTool())
    registry.register(SearchFileTool())

    agent = ReActAgent(registry=registry, model=args.model)

    # Iterate over JSON files
    # We sort them to have a deterministic order
    json_files = sorted(list(findings_dir.glob("*.json")))
    print(f"Found {len(json_files)} JSON files in {findings_dir}")

    for file_path in json_files:
        process_findings_file(file_path, dataset_root, agent)
        print("-" * 50)


if __name__ == "__main__":
    main()


"""
python main.py --dataset_root src/update_251230 --findings_dir src/update_251230/findings --model qwen3
"""
