"""Tests for ancestor-closure hierarchical metrics (Kiritchenko et al., LNAI'06).

Run: uv run python test_closure.py
"""

from model import CWE, CWEDatabase, CWEHandler, VulnPair


def make_handler() -> CWEHandler:
    # Mirrors the real CWE-346 situation in CWE-1000:
    #   284 (Pillar) -> 346, 284 -> 285
    #   693 (Pillar) -> 345 -> 346
    #   682 (Pillar) -> 190   (plain tree branch)
    def c(i, abst, child):
        return CWE(ID=i, Name=f"N{i}", Abstraction=abst, Mapping="Allowed", Child=child)

    db = CWEDatabase(
        root={
            "CWE-284": c(284, "Pillar", [346, 285]),
            "CWE-693": c(693, "Pillar", [345]),
            "CWE-345": c(345, "Class", [346]),
            "CWE-346": c(346, "Class", []),
            "CWE-285": c(285, "Class", []),
            "CWE-682": c(682, "Pillar", [190]),
            "CWE-190": c(190, "Base", []),
        }
    )
    return CWEHandler(db)


GT_346 = [{"1": ["CWE-693"], "2": ["CWE-345"], "3": ["CWE-346"]}]


def judge(handler, gt, det) -> VulnPair:
    vp = VulnPair(cwe_ground_truth=gt, cwe_detected=det)
    vp.judge_detection(handler)
    return vp


def sizes(vp):
    return vp.hier_tp_size, vp.hier_p_size, vp.hier_r_size


def test_get_all_ancestors():
    h = make_handler()
    assert {c.ID for c in h.get_all_ancestors(346)} == {284, 345, 693}
    assert {c.ID for c in h.get_all_ancestors(345)} == {693}
    assert h.get_all_ancestors(284) == set()
    assert h.get_all_ancestors(99999) == set()


def test_alternate_parent_path_gets_full_credit():
    # Predicted via 284 -> 346 instead of the GT path 693 -> 345 -> 346.
    vp = judge(make_handler(), GT_346, [{"1": ["CWE-284"], "2": ["CWE-346"]}])
    assert sizes(vp) == (4, 4, 4)
    assert vp.hierarchical_f1 == 1.0


def test_gt_path_gets_full_credit():
    vp = judge(make_handler(), GT_346, GT_346)
    assert sizes(vp) == (4, 4, 4)
    assert vp.hierarchical_f1 == 1.0


def test_both_paths_no_double_count():
    det = [{"1": ["CWE-284"], "2": ["CWE-346"]}] + GT_346
    vp = judge(make_handler(), GT_346, det)
    assert sizes(vp) == (4, 4, 4)


def test_wrong_sibling_is_still_false_positive():
    # 284 is a DAG ancestor of 346 (credited); 285 is wrong.
    vp = judge(make_handler(), GT_346, [{"1": ["CWE-284"], "2": ["CWE-285"]}])
    assert sizes(vp) == (1, 2, 4)


def test_unrelated_prediction_scores_zero():
    vp = judge(make_handler(), GT_346, [{"1": ["CWE-682"], "2": ["CWE-190"]}])
    assert sizes(vp) == (0, 2, 4)
    assert vp.hierarchical_f1 == 0.0


def test_tree_branch_unchanged():
    # Single-parent paths: closure adds nothing, same as the old path-node scoring.
    gt = [{"1": ["CWE-682"], "2": ["CWE-190"]}]
    vp = judge(make_handler(), gt, [{"1": ["CWE-682"]}])
    assert sizes(vp) == (1, 1, 2)


if __name__ == "__main__":
    tests = [v for k, v in dict(globals()).items() if k.startswith("test_")]
    failed = 0
    for t in tests:
        try:
            t()
            print(f"PASS {t.__name__}")
        except Exception as e:
            failed += 1
            print(f"FAIL {t.__name__}: {type(e).__name__}: {e}")
    print(f"{len(tests) - failed}/{len(tests)} passed")
    raise SystemExit(1 if failed else 0)
