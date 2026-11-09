import os
import sys
import json
import yaml
from pathlib import Path
from typing import List, Union
from loguru import logger

# Add parent directory to sys.path to import sibling modules
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.base import BaseProcessor
from core.models import Report, Finding
from localizer.react import ReActAgent, generate_audit_task
from localizer.vfs import VirtualFileSystem
from localizer.tool import (
    ToolRegistry,
    ListDirectoryTool,
    ReadFileTool,
    SearchFileTool,
    ReportTool,
)


class LocalizeProcessor(BaseProcessor):
    """
    Processor for localizing vulnerabilities in source code.
    Uses batch processing to handle multiple findings at once.
    """

    def _initialize(self) -> bool:
        """Initialize the localizer processor"""
        try:
            # Get batch size from config
            self.batch_size = self.config.get("localizer", {}).get("batch_size", 5)

            # Get model from config
            self.model = self.config.get("llm", {}).get("model")

            # Get max_steps and max_retries from config
            self.max_steps = self.config.get("localizer", {}).get("max_steps", 30)
            self.max_retries = self.config.get("localizer", {}).get("max_retries", 3)

            # Localize mode should always overwrite to avoid .json.json naming
            self.overwrite = True

            # Setup agent tools
            self.registry = ToolRegistry()
            self.registry.register(ListDirectoryTool())
            self.registry.register(ReadFileTool())
            self.registry.register(SearchFileTool())
            self.registry.register(ReportTool())

            # Initialize agent with max_steps and max_retries
            self.agent = ReActAgent(
                registry=self.registry,
                model=self.model,
                max_steps=self.max_steps,
                max_retries=self.max_retries,
            )

            logger.info(
                "Localizer initialized with batch_size={}, max_steps={}, max_retries={}",
                self.batch_size,
                self.max_steps,
                self.max_retries,
            )
            return True
        except Exception as e:
            logger.error(f"Failed to initialize localizer: {e}")
            return False

    def _parse_file(self, filepath: str) -> Union[Report, None]:
        """
        Parse the JSON file containing vulnerability findings

        Args:
            filepath: Path to the JSON file

        Returns:
            Report object or None if parsing failed
        """
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                data = json.load(f)

            # Convert to Report object
            report = Report(**data)
            return report

        except Exception as e:
            logger.error(f"Failed to parse file {filepath}: {e}")
            return None

    def _process(self, report: Report) -> Union[Report, None]:
        """
        Process the report and localize vulnerabilities in batches

        Args:
            report: Report object containing vulnerability findings

        Returns:
            Updated Report object or None if processing failed
        """
        if not report or not report.project_info or not report.findings:
            logger.warning("Empty report or missing project info/findings")
            return report

        # Extract project_path string from potentially complex types
        project_path_raw = report.project_info.project_path
        project_path_str = None

        if isinstance(project_path_raw, str):
            project_path_str = project_path_raw
        elif isinstance(project_path_raw, dict):
            # Try to extract a path from dict (e.g., {'path': '...'})
            project_path_str = project_path_raw.get("path") or project_path_raw.get(
                "value"
            )
            if not project_path_str and project_path_raw:
                # Use first value if no standard key found
                project_path_str = next(iter(project_path_raw.values()), None)
        elif isinstance(project_path_raw, list) and project_path_raw:
            # Use first item if it's a list
            project_path_str = project_path_raw[0]

        if not project_path_str or project_path_str in ["n/a", "", None]:
            logger.warning(
                "No valid project_path found in report, skipping localization"
            )
            return report

        # Construct absolute project path
        project_full_path = Path(project_path_str)

        # If project_path is relative, resolve it relative to output directory
        if not project_full_path.is_absolute():
            # Try to resolve relative to output directory
            # project_full_path = Path(self.output) / project_path_str
            project_full_path = Path(project_path_str)

        if not project_full_path.exists():
            logger.error(f"Project path {project_full_path} does not exist")
            return report

        # Filter findings that need localization (empty or invalid files)
        findings_to_localize = []
        for finding in report.findings:
            files = (
                finding.location
                if isinstance(finding.location, list)
                else ([finding.location] if finding.location else [])
            )

            # Check if files list is empty or contains invalid paths
            if not files or files == ["n/a"] or files == [""] or files == []:
                findings_to_localize.append(finding)
                continue

            # Check if files exist
            files_ok = True
            for file_rel_path in files:
                if file_rel_path in ["n/a", "", None]:
                    files_ok = False
                    break
                # The file path should be relative to the project root
                file_abs_path = project_full_path / file_rel_path
                if not file_abs_path.exists():
                    logger.debug(f"File not found: {file_abs_path}")
                    files_ok = False

            if not files_ok:
                findings_to_localize.append(finding)

        if not findings_to_localize:
            logger.info("All findings already have valid file locations")
            return report

        logger.info(
            f"Found {len(findings_to_localize)}/{len(report.findings)} findings "
            f"that need localization"
        )

        # Process findings in batches
        total_batches = (
            len(findings_to_localize) + self.batch_size - 1
        ) // self.batch_size

        for batch_idx in range(total_batches):
            start_idx = batch_idx * self.batch_size
            end_idx = min(start_idx + self.batch_size, len(findings_to_localize))
            batch = findings_to_localize[start_idx:end_idx]

            logger.info(
                f"Processing batch {batch_idx + 1}/{total_batches} "
                f"({len(batch)} findings)"
            )

            # Create VFS for the project directory
            try:
                vfs = VirtualFileSystem(project_full_path)
                self.registry.set_vfs(vfs)
                logger.debug(f"VFS initialized: {vfs}")
            except Exception as e:
                logger.error(f"Failed to initialize VFS: {e}")
                continue

            # Prepare findings for agent (convert to dict format)
            batch_findings_dict = []
            for finding in batch:
                batch_findings_dict.append(
                    {
                        "id": finding.id,
                        "title": finding.title,
                        "description": finding.description,
                        "location": (
                            finding.location
                            if isinstance(finding.location, str)
                            else (finding.location[0] if finding.location else "n/a")
                        ),
                    }
                )

            # Generate task for this batch
            # Use "." as the virtual root in the task description
            task = generate_audit_task(".", batch_findings_dict)

            # Run agent
            try:
                result_yaml = self.agent.run(task)

                # Parse result
                updates = yaml.safe_load(result_yaml)

                if isinstance(updates, dict):
                    updated_count = 0
                    for finding in batch:
                        f_id = finding.id
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
                                    # Files are already relative paths from VFS
                                    # Just validate them
                                    nf_path = Path(nf)

                                    # Ensure it's a relative path
                                    if nf_path.is_absolute():
                                        logger.warning(
                                            f"Received absolute path {nf}, "
                                            f"attempting to convert to relative"
                                        )
                                        try:
                                            rel_path = nf_path.relative_to(
                                                project_full_path
                                            )
                                            cleaned_files.append(str(rel_path))
                                        except ValueError:
                                            # Path is outside project, skip it
                                            logger.warning(
                                                f"Skipping path outside project: {nf}"
                                            )
                                            continue
                                    else:
                                        # Already relative, use as-is
                                        cleaned_files.append(str(nf_path))

                                logger.debug(
                                    f"Updating finding {f_id} files to: {cleaned_files}"
                                )
                                # Update finding location
                                finding.files = cleaned_files
                                updated_count += 1

                    if updated_count > 0:
                        logger.info(f"Updated {updated_count} findings in this batch")
                        # Save progress after each batch only if not running as subprocess
                        if not self.is_subprocess:
                            # Force overwrite=True to handle .json extension correctly
                            self.export_result(
                                report,
                                filename=self.file_name,
                                output_dir=self.output,
                                overwrite=True,
                            )
                    else:
                        logger.warning(
                            "Agent returned results but no matching IDs found"
                        )
                else:
                    logger.warning(f"Agent returned unexpected format: {type(updates)}")

            except Exception as e:
                logger.error(f"Failed to localize batch {batch_idx + 1}: {e}")
                continue

        return report

    def _validate_file(self, filename, valid_ext) -> bool:
        """Validate file extension"""
        if os.path.isfile(filename):
            _, ext = os.path.splitext(filename)
            if ext.lower() in valid_ext:
                return True
        return False
