import os
from pydantic import BaseModel
from typing import List
from extractor.document_handler import DocumentHandler
from extractor.map_reducer import MapReducer
from classifier.tot_reasoner import TotReasoner
from classifier.cwe_handler import CWEHandler
from fetcher.fetch_processor import _fetcher
from localizer.localize_processor import LocalizeProcessor
from core.models import Report
from core.base import BaseProcessor
from core.sol_invoker import CONFIG, MAX_RETRIES, INTERVAL, CHUNK_LENGTH
from loguru import logger


class BuildProcessor(BaseProcessor):
    def _process(self, input: List[str]) -> Report | None:

        map_reducer = MapReducer()
        reasoner = TotReasoner()
        cwe_handler = CWEHandler()
        cwe_handler.load_dict(CONFIG["classifier"].get("cwe_data", "cwe_dict.json"))
        map_reduce_result = map_reducer.map_reduce(input, self.context_length)
        logger.info(
            "Finished.  {} finding(s) extracted", len(map_reduce_result.findings)
        )
        report = Report(
            path=self.filepath,
            project_info=map_reduce_result.project_info,
            findings=map_reduce_result.findings,
        )

        # Export initial findings to findings directory
        findings_dir = os.path.join(self.dataset_root, "findings")
        os.makedirs(findings_dir, exist_ok=True)
        self.export_result(
            report,
            filename=self.file_name,
            output_dir=findings_dir,
            overwrite=True,
        )

        # Fetch source code to contracts directory
        contracts_dir = os.path.join(self.dataset_root, "contracts")
        os.makedirs(contracts_dir, exist_ok=True)
        report.project_info, fetch_success = _fetcher(
            report, self.output, contracts_dir
        )

        if not fetch_success:
            # Fetch failed - export to manual_verification and skip classify/localize
            logger.warning(
                "Fetch failed for {}: no targets or all fetch attempts failed. "
                "Exporting to manual_verification folder.",
                self.file_name,
            )
            manual_verification_dir = os.path.join(
                self.dataset_root, "manual_verification"
            )
            os.makedirs(manual_verification_dir, exist_ok=True)
            self.export_result(
                report,
                filename=self.file_name,
                output_dir=manual_verification_dir,
                overwrite=True,
            )
            return report

        # Fetch succeeded - update findings with project_info
        self.export_result(
            report,
            filename=self.file_name,
            output_dir=findings_dir,
            overwrite=True,
        )

        # Classify vulnerabilities
        for finding in report.findings:
            vuln_info = {"title": finding.title, "description": finding.description}
            try:
                category = reasoner.analyze(
                    vuln_info=vuln_info, cwe_handler=cwe_handler
                )
                finding.category = category
                self.export_result(
                    report,
                    filename=self.file_name,
                    output_dir=findings_dir,
                    overwrite=True,
                )
                logger.info("Classified {} as {}", finding.title, category)
            except Exception as e:
                logger.error(e)
                continue

        # Localize vulnerabilities
        logger.info("Starting vulnerability localization...")
        try:
            localizer = LocalizeProcessor(
                "localize",
                self.target,
                self.output,
                self.log_dir,
                self.config_path,
                is_subprocess=True,
            )
            localizer._initialize()
            # Share file_name and overwrite settings to ensure consistent file naming
            localizer.file_name = self.file_name
            localizer.overwrite = self.overwrite
            report = localizer._process(report)
            self.export_result(
                report,
                filename=self.file_name,
                output_dir=findings_dir,
                overwrite=True,
            )
            logger.info("Vulnerability localization completed")
        except Exception as e:
            logger.error(f"Localization failed: {e}")

        return report

    def _parse_file(self, filepath: str) -> List[str] | None:
        self.filepath = filepath
        self.context_length = CHUNK_LENGTH
        max_heading_level = CONFIG.get("extractor", {}).get("max_heading_level", 3)
        doc_handler = DocumentHandler(
            max_level=max_heading_level, max_tokens=self.context_length
        )
        return doc_handler.process(filepath=self.filepath)

    def _initialize(self) -> bool:
        # Set overwrite to ensure consistent file naming (.pdf -> .json, not .pdf -> .pdf.json.json)
        self.overwrite = True

        # Set up dataset_root directory structure
        self.dataset_root = self.output
        os.makedirs(os.path.join(self.dataset_root, "reports"), exist_ok=True)
        os.makedirs(os.path.join(self.dataset_root, "findings"), exist_ok=True)
        os.makedirs(os.path.join(self.dataset_root, "contracts"), exist_ok=True)

        valid_ext = CONFIG.get("extractor", {}).get("valid_ext", None)
        if valid_ext:
            self.valid_ext = valid_ext
            return True
        else:
            return False

    def run(self) -> bool:
        """
        Override run() to prevent duplicate exports.
        BuildProcessor manages its own export logic in _process().
        """
        result = None
        if not self._initialize():
            logger.error("Failed to initialize!")
        if not self.validate_target():
            logger.error("Invalid path or file!")
            return False
        if self.isdir:
            for root, dirs, files in os.walk(self.target):
                for f in files:
                    file_path = os.path.join(root, f)
                    if not self._validate_file(file_path, self.valid_ext):
                        continue
                    if file_path in self.history.finished:
                        logger.info("Skip analyzed file {}", file_path)
                        continue
                    self.file_name = f
                    logger.info("Processing {}...", file_path)
                    input = self._parse_file(file_path)
                    result = self._process(input)
                    is_failed = False
                    if not result:
                        is_failed = True
                    # Skip export here - BuildProcessor handles it internally
                    self._save_history(file_path, self.log_dir, is_failed)
            return True
        else:
            file_path = self.target
            f = os.path.basename(file_path)
            self.file_name = f
            logger.info("Processing {}...", file_path)
            input = self._parse_file(file_path)
            result = self._process(input)
            is_failed = False
            if not result:
                is_failed = True
            # Skip export here - BuildProcessor handles it internally
            self._save_history(file_path, self.log_dir, is_failed)
            return True


# if __name__ == "__main__":
#     extractor = ExtractProcessor(
#         task="extract",
#         target="tests/test_cases/test_pdf1.pdf",
#         output="tests/test_output",
#         log_dir="src/logs",
#         config_path="src/config.yaml",
#     )
#     extractor.run()
