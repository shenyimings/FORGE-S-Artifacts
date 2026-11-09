from dataclasses import dataclass
from dataclasses import field
from typing import List, Dict, Set, Union, Optional, Literal, Tuple
from pydantic import BaseModel, RootModel, Field
from pathlib import Path
import hashlib


class CWE(BaseModel):
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
        return hash(self.ID)

    def __eq__(self, other):
        if not isinstance(other, CWE):
            return NotImplemented
        return self.ID == other.ID

    def add_child(self, child_cwe: "CWE"):
        self.Child.append(child_cwe)
        child_cwe.Parent.append(self)


class CWEDatabase(RootModel):
    root: Dict[str, CWE]

    def get_by_id(self, id: int | str):
        name = f"CWE-{id}"
        return self.root[name]

    def get_by_name(self, name: str):
        return self.root[name]


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
    files: List[str] = field(default_factory=list)
    location: Union[str, int, List] = ""


class Report(BaseModel):
    path: str = ""
    project_info: ProjectInfo = field(default_factory=ProjectInfo)
    findings: List[Finding] = field(default_factory=list)

    def append_finding(self, finding: Finding):
        self.findings.append(finding)


class CWEHandler:
    def __init__(self, cwe_database: CWEDatabase):
        self.db = cwe_database
        self.setup_relationships()

    @classmethod
    def load_from_file(cls, file_path: str) -> "CWEHandler":
        import json

        with open(file_path, "r") as f:
            data = json.load(f)
        # CWEDatabase expects a dict where keys are CWE IDs (strings) and values are CWE objects
        db = CWEDatabase.model_validate(data)
        return cls(db)

    def setup_relationships(self):
        """
        Build the parent-child relationships (DAG) based on the 'Child' field (List[int]).
        Populates the 'Parent' field (List[CWE]) for each CWE.
        """
        # First pass: Convert any integer IDs in Parent to CWE objects or clear if invalid
        for cwe in self.db.root.values():
            cleaned_parents = []
            for p in cwe.Parent:
                if isinstance(p, CWE):
                    cleaned_parents.append(p)
                elif isinstance(p, int):
                    try:
                        parent_cwe = self.db.get_by_id(p)
                        cleaned_parents.append(parent_cwe)
                    except KeyError:
                        pass
            cwe.Parent = cleaned_parents

        # Second pass: Build relationships from Child fields
        for cwe_key, cwe in self.db.root.items():
            for child_id in cwe.Child:
                try:
                    child_cwe = self.db.get_by_id(child_id)
                    # Add current cwe as parent to the child
                    # Check if already exists to avoid duplicates if run multiple times
                    if cwe not in child_cwe.Parent:
                        child_cwe.Parent.append(cwe)
                except KeyError:
                    # Child ID might not exist in the database
                    continue

    def get_cwe(self, cwe_id: Union[int, str]) -> Optional[CWE]:
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

    def get_direct_children(self, cwe_id: Union[int, str]) -> List[CWE]:
        cwe = self.get_cwe(cwe_id)
        if not cwe:
            return []
        children = []
        for child_id in cwe.Child:
            child = self.get_cwe(child_id)
            if child:
                children.append(child)
        return children

    def get_direct_parents(self, cwe_id: Union[int, str]) -> List[CWE]:
        cwe = self.get_cwe(cwe_id)
        if not cwe:
            return []
        return cwe.Parent

    def get_all_descendants(self, cwe_id: Union[int, str]) -> List[CWE]:
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

    def get_all_ancestors(self, cwe_id: Union[int, str]) -> Set[CWE]:
        """All ancestors over every parent path in the DAG (excluding the node itself)."""
        cwe = self.get_cwe(cwe_id)
        if not cwe:
            return set()
        ancestors = set()
        stack = [cwe]
        while stack:
            for parent in stack.pop().Parent:
                if parent not in ancestors:
                    ancestors.add(parent)
                    stack.append(parent)
        return ancestors

    def get_root_parents(self, cwe_id: Union[int, str]) -> List[CWE]:
        """
        Recursively find the most root parents (Pillars).
        """
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

            # If it's a Pillar, it's a root
            if current.Abstraction == "Pillar":
                roots.add(current)
                continue  # Stop traversing up from a Pillar? Usually yes.

            # If no parents, it's a root
            if not current.Parent:
                roots.add(current)
                continue

            # Otherwise, traverse up
            for parent in current.Parent:
                queue.append(parent)

        return list(roots)

    def is_related(self, id1: Union[int, str], id2: Union[int, str]) -> bool:
        """
        Check if there is any inheritance relationship between two CWEs.
        Returns True if id1 is ancestor of id2 OR id2 is ancestor of id1.
        """
        cwe1 = self.get_cwe(id1)
        cwe2 = self.get_cwe(id2)

        if not cwe1 or not cwe2:
            return False

        if cwe1 == cwe2:
            return True

        # Check if cwe1 is ancestor of cwe2
        if self._is_ancestor(cwe1, cwe2):
            return True

        # Check if cwe2 is ancestor of cwe1
        if self._is_ancestor(cwe2, cwe1):
            return True

        return False

    def _is_ancestor(self, ancestor: CWE, descendant: CWE) -> bool:
        # BFS up from descendant to find ancestor
        queue = [descendant]
        visited = set()
        while queue:
            current = queue.pop(0)
            if current in visited:
                continue
            visited.add(current)

            if current == ancestor:
                return True

            for parent in current.Parent:
                queue.append(parent)
        return False


class DetectedCWE(BaseModel):

    title: str = Field(
        description="Title of the vulnerability",
        examples=["Improper Access Control"],
    )
    description: str = Field(
        description="Detailed description of the vulnerability",
        examples=[
            "The function `withdraw` allows any user to withdraw funds because it lacks a check to ensure the caller is the owner."
        ],
    )
    category: str = Field(description="The CWE ID", examples=["CWE-284"])
    severity: Optional[str] = Field(
        description="Severity", examples=["critical"], default="n/a"
    )
    confidence: Union[float, int, str] = Field(
        description="Confidence of your finding", examples=["0.99"], default=0
    )


class DetectionResult(BaseModel):
    reasoning: Optional[str] = Field(
        description="Reasoning",
        examples=[
            "The function `withdraw` allows any user to withdraw funds because it lacks a check to ensure the caller is the owner."
        ],
        default="",
    )
    findings: List[DetectedCWE] = Field(default_factory=list)

    @classmethod
    def get_yaml_schema(cls) -> str:
        ex_reasoning = "The contract exists three kinds of CWE vulnerabilities. First, the function `withdraw` allows any user to withdraw funds because it lacks a check to ensure the caller is the owner...."
        ex_cwe_id = "CWE-1000"
        ex_severity = "critical"
        ex_confidence = 0.99
        ex_title = "Lack of Access Control in withdraw Function"
        ex_description = "The function `withdraw` does not verify the caller's identity, allowing unauthorized users to withdraw funds."

        return f"""
```yaml
reasoning: "{ex_reasoning}"
findings:
  - title: "{ex_title}" # (title of the vulnerability finding)
    description: "{ex_description}" # (detailed description of the vulnerability)
    category: "{ex_cwe_id}" # (CWE ID in the candidate list, e.g., CWE-1000)
    severity: "{ex_severity}" # (info, low, medium, high, critical)
    confidence: {ex_confidence} # (confidence score for your findings, float between 0 and 1)
```
"""


@dataclass
class VulnPair:
    project: str = ""
    files: List[Path] = field(default_factory=list)
    cwe_detected: List[Dict[str, List[str]]] = field(default_factory=list)
    cwe_ground_truth: List[Dict[str, List[str]]] = field(default_factory=list)
    vuln_ground_truth: List[Dict[str, str]] = field(default_factory=list)
    # Hierarchical Metrics
    hierarchical_precision: float = 0.0
    hierarchical_recall: float = 0.0
    hierarchical_f1: float = 0.0

    # Hierarchical Set Sizes (for aggregation)
    hier_tp_size: int = 0  # |P ∩ T|
    hier_p_size: int = 0  # |P|
    hier_r_size: int = 0  # |T|

    # Leaf Metrics
    # tfnp_leaf: Tuple[Dict, Dict, Dict] = field(default_factory=lambda: ({}, {}, {}))
    # prf_leaf: Tuple[Dict, Dict, Dict] = field(default_factory=lambda: ({}, {}, {}))

    # LLM Reasoning
    candidate_cwes: List[List] = field(default_factory=list)
    results: List[DetectionResult] = field(default_factory=list)

    def _calc_prf(self, tp: int, fp: int, fn: int) -> tuple:
        if tp + fp == 0 or tp + fn == 0:
            return 0.0, 0.0, 0.0
        precision = tp / (tp + fp)
        recall = tp / (tp + fn)
        if precision + recall == 0:
            return 0.0, 0.0, 0.0
        f1 = 2 * (precision * recall) / (precision + recall)
        return precision, recall, f1

    def gen_pair_id(self) -> str:
        file_names = [str(f) for f in self.files]
        file_names.sort()
        proj_hex = hashlib.sha256(str(self.project).encode("utf-8")).hexdigest()[:6]
        files_hex = hashlib.sha256("__".join(file_names).encode("utf-8")).hexdigest()[
            -12:
        ]
        pair_id = f"{proj_hex}-{files_hex}"
        return pair_id

    def to_json(self) -> str:
        import json

        data = {
            "project": self.project,
            "files": [str(f) for f in self.files],
            "hierarchical_metrics": {
                "precision": self.hierarchical_precision,
                "recall": self.hierarchical_recall,
                "f1": self.hierarchical_f1,
                "tp_size": self.hier_tp_size,
                "p_size": self.hier_p_size,
                "r_size": self.hier_r_size,
            },
            "cwe_ground_truth": self.cwe_ground_truth,
            "cwe_detected": self.cwe_detected,
            "vuln_ground_truth": self.vuln_ground_truth,
            "results": [result.model_dump() for result in self.results],
            "candidate_cwes": self.candidate_cwes,
            # "leaf_metrics": {
            #     "tfnp_leaf": {
            #         "tp": self.tfnp_leaf[0],
            #         "fp": self.tfnp_leaf[1],
            #         "fn": self.tfnp_leaf[2],
            #     },
            #     "prf_leaf": {
            #         "precision": self.prf_leaf[0],
            #         "recall": self.prf_leaf[1],
            #         "f1": self.prf_leaf[2],
            #     },
            # },
        }
        return json.dumps(data, indent=4, ensure_ascii=False)

    def judge_detection(self, cwe_handler: CWEHandler):
        """
        Judge the detection results by comparing detected CWEs with ground truth CWEs.
        Updates hierarchical_precision, hierarchical_recall, hierarchical_f1,
        tfnp_leaf, prf_leaf attributes.
        """

        # Helper to extract CWE objects from chains
        def get_hierarchical_set(cwe_list: List[Dict[str, List[str]]]) -> set:
            cwes = set()
            # Extract all explicit CWEs from all chains
            for item in cwe_list:
                for cwe_strs in item.values():
                    if isinstance(cwe_strs, list) and cwe_strs:
                        cwe_str = cwe_strs[0]
                        cwe_obj = cwe_handler.get_cwe(cwe_str)
                        if cwe_obj:
                            cwes.add(cwe_obj)

            # Ancestor closure over all DAG parent paths (Kiritchenko et al., LNAI'06),
            # so a node is credited regardless of which parent path reached it.
            closed = set(cwes)
            for cwe in cwes:
                closed |= cwe_handler.get_all_ancestors(cwe.ID)
            return closed

        # 1. Hierarchical Performance Metrics
        P_set = get_hierarchical_set(self.cwe_detected)
        T_set = get_hierarchical_set(self.cwe_ground_truth)

        # print(f"Calc: Detected hierarchical CWEs: {[f'CWE-{c.ID}' for c in P_set]}")
        # print(f"Calc: Ground Truth hierarchical CWEs: {[f'CWE-{c.ID}' for c in T_set]}")
        intersection = P_set & T_set

        # Store set sizes for aggregation
        self.hier_tp_size = len(intersection)
        self.hier_p_size = len(P_set)
        self.hier_r_size = len(T_set)

        # Precision = |P ∩ T| / |P|
        if len(P_set) > 0:
            self.hierarchical_precision = len(intersection) / len(P_set)
        else:
            self.hierarchical_precision = 0.0

        # Recall = |P ∩ T| / |T|
        if len(T_set) > 0:
            self.hierarchical_recall = len(intersection) / len(T_set)
        else:
            self.hierarchical_recall = 0.0

        # F1
        if self.hierarchical_precision + self.hierarchical_recall > 0:
            self.hierarchical_f1 = (
                2
                * (self.hierarchical_precision * self.hierarchical_recall)
                / (self.hierarchical_precision + self.hierarchical_recall)
            )
        else:
            self.hierarchical_f1 = 0.0

        # 2. Leaf-based evaluation
        # def get_leaf_cwes(cwe_list: List[Dict[str, List[str]]]) -> set:
        #     leaves = set()
        #     for item in cwe_list:
        #         if not item:
        #             continue
        #         try:
        #             max_level = max(int(k) for k in item.keys())
        #             cwe_strs = item[str(max_level)]
        #             if isinstance(cwe_strs, list) and cwe_strs:
        #                 leaf_cwe_str = cwe_strs[0]
        #                 cwe_obj = cwe_handler.get_cwe(leaf_cwe_str)
        #                 if cwe_obj:
        #                     leaves.add(cwe_obj)
        #         except ValueError:
        #             continue
        #     return leaves

        # gt_leaves = get_leaf_cwes(self.cwe_ground_truth)
        # det_leaves = get_leaf_cwes(self.cwe_detected)

        # all_leaves = gt_leaves | det_leaves
        # tp_leaf, fp_leaf, fn_leaf = {}, {}, {}
        # p_leaf, r_leaf, f1_leaf = {}, {}, {}

        # for leaf in all_leaves:
        #     leaf_name = f"CWE-{leaf.ID}"
        #     in_gt = leaf in gt_leaves
        #     in_det = leaf in det_leaves

        #     is_tp = False
        #     if in_gt and in_det:
        #         is_tp = True
        #     elif in_det and not in_gt:
        #         # Check if this detection corresponds to a coarser GT (GT is ancestor of Det)
        #         # If GT has an ancestor of this leaf, we count it as TP for this leaf
        #         if any(cwe_handler._is_ancestor(g, leaf) for g in gt_leaves):
        #             is_tp = True
        #     elif in_gt and not in_det:
        #         # Check if this GT is covered by a more specific detection (GT is ancestor of Det)
        #         # If Det has a descendant of this leaf, we count it as TP for this leaf
        #         if any(cwe_handler._is_ancestor(leaf, d) for d in det_leaves):
        #             is_tp = True

        #     tp = 1 if is_tp else 0
        #     fp = 1 if in_det and not is_tp else 0
        #     fn = 1 if in_gt and not is_tp else 0

        #     tp_leaf[leaf_name] = tp
        #     fp_leaf[leaf_name] = fp
        #     fn_leaf[leaf_name] = fn

        #     p_leaf[leaf_name], r_leaf[leaf_name], f1_leaf[leaf_name] = self._calc_prf(
        #         tp, fp, fn
        #     )

        # self.tfnp_leaf = (tp_leaf, fp_leaf, fn_leaf)
        # self.prf_leaf = (p_leaf, r_leaf, f1_leaf)


class ProjectRecord(BaseModel):
    project_name: str
    report_path: str
    scan_time: float = 0.0
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0

    # Metrics (Leaf Level)
    # tp: int = 0
    # fp: int = 0
    # fn: int = 0
    # precision: float = 0.0
    # recall: float = 0.0
    # f1: float = 0.0

    # Hierarchical Metrics
    hierarchical_precision: float = 0.0
    hierarchical_recall: float = 0.0
    hierarchical_f1: float = 0.0

    # Hierarchical Counts (for aggregation)
    hier_tp_size: int = 0
    hier_p_size: int = 0
    hier_r_size: int = 0

    # Detailed metrics aligned with VulnPair
    # tfnp_leaf: Tuple[Dict, Dict, Dict] = Field(default_factory=lambda: ({}, {}, {}))
    # candidate_cwes: List[str] = Field(default_factory=list)
    # llm_results: List[DetectionResult] = Field(default_factory=list)
    # Detailed metrics can be stored if needed
    # metrics_detail: Dict = Field(default_factory=dict)


class DetectorModel(BaseModel):
    total_scan_time: float = 0.0
    total_prompt_tokens: int = 0
    total_completion_tokens: int = 0
    total_tokens: int = 0

    # # Aggregate Metrics (Leaf Level)
    # total_tp: int = 0
    # total_fp: int = 0
    # total_fn: int = 0
    hier_precision: float = 0.0
    hier_recall: float = 0.0
    hier_f1: float = 0.0
    # Aggregate Metrics (Hierarchical)
    total_hier_tp: int = 0
    total_hier_p: int = 0
    total_hier_r: int = 0

    history: List[ProjectRecord] = Field(default_factory=list)

    def add_record(self, record: ProjectRecord):
        self.history.append(record)
        self.total_scan_time += record.scan_time
        self.total_prompt_tokens += record.prompt_tokens
        self.total_completion_tokens += record.completion_tokens
        self.total_tokens += record.total_tokens

        # self.total_tp += record.tp
        # self.total_fp += record.fp
        # self.total_fn += record.fn

        self.total_hier_tp += record.hier_tp_size
        self.total_hier_p += record.hier_p_size
        self.total_hier_r += record.hier_r_size

        hierarchical_metrics = self.aggregate_metrics
        self.hier_precision = hierarchical_metrics["hierarchical"]["precision"]
        self.hier_recall = hierarchical_metrics["hierarchical"]["recall"]
        self.hier_f1 = hierarchical_metrics["hierarchical"]["f1"]

    @property
    def aggregate_metrics(self):
        # Leaf Metrics
        # tp = self.total_tp
        # fp = self.total_fp
        # fn = self.total_fn

        # precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
        # recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
        # f1 = (
        #     2 * (precision * recall) / (precision + recall)
        #     if (precision + recall) > 0
        #     else 0.0
        # )

        # Hierarchical Metrics
        h_tp = self.total_hier_tp
        h_p = self.total_hier_p
        h_r = self.total_hier_r

        h_precision = h_tp / h_p if h_p > 0 else 0.0
        h_recall = h_tp / h_r if h_r > 0 else 0.0
        h_f1 = (
            2 * (h_precision * h_recall) / (h_precision + h_recall)
            if (h_precision + h_recall) > 0
            else 0.0
        )

        return {
            "hierarchical": {
                "tp_size": h_tp,
                "p_size": h_p,
                "r_size": h_r,
                "precision": h_precision,
                "recall": h_recall,
                "f1": h_f1,
            },
        }

    def save(self, path: str):
        import json

        with open(path, "w") as f:
            f.write(self.model_dump_json(indent=2))

    @classmethod
    def load(cls, path: str) -> "DetectorModel":
        import json

        if not Path(path).exists():
            return cls()
        with open(path, "r") as f:
            data = json.load(f)
        return cls.model_validate(data)


if __name__ == "__main__":
    # Example usage
    print("Loading CWE database...")
    handler = CWEHandler.load_from_file("cwe_dict.json")
    cwe_id = 384  # Example CWE ID
    cwe = handler.get_cwe(cwe_id)
    if cwe:
        print(f"CWE-{cwe.ID}: {cwe.Name}")
        children = handler.get_direct_children(cwe_id)
        print(f"Direct Children: {[child.ID for child in children]}")
        parents = handler.get_direct_parents(cwe_id)
        print(f"Direct Parents: {[parent.ID for parent in parents]}")
        descendants = handler.get_all_descendants(cwe_id)
        print(f"All Descendants: {[desc.ID for desc in descendants]}")
        roots = handler.get_root_parents(cwe_id)
        print(f"Root Parents: {[root.ID for root in roots]}")
