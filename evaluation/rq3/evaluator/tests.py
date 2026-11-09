from model import CWE, CWEDatabase, CWEHandler, VulnPair


def test_metrics():
    print("\n--- Running Complex Metrics Test ---")
    # 1. Setup Mock DB
    # Hierarchy:
    # P1 (1) -> C1 (11) -> B1 (111)
    # P2 (2) -> C2 (22) -> B2 (222) -> V2 (2222)
    # P3 (3) -> C3 (33)
    # P4 (4) -> C4 (44)

    cwe_p1 = CWE(
        ID=1, Name="Pillar-1", Abstraction="Pillar", Mapping="Allowed", Child=[11]
    )
    cwe_c1 = CWE(
        ID=11, Name="Class-1", Abstraction="Class", Mapping="Allowed", Child=[111]
    )
    cwe_b1 = CWE(ID=111, Name="Base-1", Abstraction="Base", Mapping="Allowed", Child=[])

    cwe_p2 = CWE(
        ID=2, Name="Pillar-2", Abstraction="Pillar", Mapping="Allowed", Child=[22]
    )
    cwe_c2 = CWE(
        ID=22, Name="Class-2", Abstraction="Class", Mapping="Allowed", Child=[222]
    )
    cwe_b2 = CWE(
        ID=222, Name="Base-2", Abstraction="Base", Mapping="Allowed", Child=[2222]
    )
    cwe_v2 = CWE(
        ID=2222, Name="Variant-2", Abstraction="Variant", Mapping="Allowed", Child=[]
    )

    cwe_p3 = CWE(
        ID=3, Name="Pillar-3", Abstraction="Pillar", Mapping="Allowed", Child=[33]
    )
    cwe_c3 = CWE(
        ID=33, Name="Class-3", Abstraction="Class", Mapping="Allowed", Child=[]
    )

    cwe_p4 = CWE(
        ID=4, Name="Pillar-4", Abstraction="Pillar", Mapping="Allowed", Child=[44]
    )
    cwe_c4 = CWE(
        ID=44, Name="Class-4", Abstraction="Class", Mapping="Allowed", Child=[]
    )

    db_dict = {
        "CWE-1": cwe_p1,
        "CWE-11": cwe_c1,
        "CWE-111": cwe_b1,
        "CWE-2": cwe_p2,
        "CWE-22": cwe_c2,
        "CWE-222": cwe_b2,
        "CWE-2222": cwe_v2,
        "CWE-3": cwe_p3,
        "CWE-33": cwe_c3,
        "CWE-4": cwe_p4,
        "CWE-44": cwe_c4,
    }
    db = CWEDatabase(root=db_dict)
    handler = CWEHandler(db)

    # 2. Setup VulnPair
    # GT:
    # 1. 1 -> 11 -> 111
    # 2. 2 -> 22 -> 222 -> 2222
    # 3. 3 -> 33
    gt = [
        {"1": ["CWE-11"], "2": ["CWE-111"]},
        {"1": ["CWE-22"], "2": ["CWE-222"], "3": ["CWE-2222"]},
        {"1": ["CWE-33"]},
    ]

    # Det:
    # 1. 1 -> 11 (Shallower than GT)
    # 2. 2 -> 22 -> 222 (Shallower than GT)
    # 3. 3 -> 33 (Exact)
    # 4. 4 -> 44 (FP)
    det = [
        {"1": ["CWE-11"]},
        {"1": ["CWE-22"], "2": ["CWE-222"]},
        {"1": ["CWE-33"]},
        {"1": ["CWE-44"]},
    ]

    vp = VulnPair(cwe_ground_truth=gt, cwe_detected=det)
    vp.judge_detection(handler)

    # 3. Verify Level Metrics
    tp_lvl, fp_lvl, fn_lvl = vp.tfnp_level
    p_lvl, r_lvl, f1_lvl = vp.prf_level

    print("\n--- Level Metrics ---")
    for lvl in sorted(tp_lvl.keys()):
        print(f"Level {lvl}: TP={tp_lvl[lvl]}, FP={fp_lvl[lvl]}, FN={fn_lvl[lvl]}")
        print(f"         P={p_lvl[lvl]:.2f}, R={r_lvl[lvl]:.2f}, F1={f1_lvl[lvl]:.2f}")

    # Level 1: GT={11, 22, 33}, Det={11, 22, 33, 44} -> TP=3, FP=1, FN=0
    assert tp_lvl[1] == 3
    assert fp_lvl[1] == 1
    assert fn_lvl[1] == 0

    # Level 2: GT={111, 222}, Det={222} -> TP=1, FP=0, FN=1
    assert tp_lvl[2] == 1
    assert fp_lvl[2] == 0
    assert fn_lvl[2] == 1

    # Level 3: GT={2222}, Det={} -> TP=0, FP=0, FN=1
    assert tp_lvl[3] == 0
    assert fp_lvl[3] == 0
    assert fn_lvl[3] == 1

    # 4. Verify Pillar Metrics
    tp_cwe, fp_cwe, fn_cwe = vp.tfnp_cwe
    p_cwe, r_cwe, f1_cwe = vp.prf_cwe

    print("\n--- Pillar Metrics ---")
    # GT Roots: {1, 2, 3}
    # Det Roots: {1, 2, 3, 4}
    # TP=3 (1, 2, 3), FP=1 (4), FN=0

    for pillar in tp_cwe.keys():
        print(
            f"{pillar}: TP={tp_cwe[pillar]}, FP={fp_cwe[pillar]}, FN={fn_cwe[pillar]}"
        )
        print(
            f"         P={p_cwe[pillar]:.2f}, R={r_cwe[pillar]:.2f}, F1={f1_cwe[pillar]:.2f}"
        )

    assert tp_cwe["CWE-1"] == 1
    assert tp_cwe["CWE-2"] == 1
    assert tp_cwe["CWE-3"] == 1
    assert fp_cwe["CWE-4"] == 1

    # 5. Verify Leaf Metrics
    tp_leaf, fp_leaf, fn_leaf = vp.tfnp_leaf
    p_leaf, r_leaf, f1_leaf = vp.prf_leaf

    print("\n--- Leaf Metrics ---")
    # GT Leaves: {111, 2222, 33}
    # Det Leaves: {11, 222, 33, 44}
    # TP: 33 (1)
    # FP: 11, 222, 44 (3)
    # FN: 111, 2222 (2)

    for leaf in tp_leaf.keys():
        print(f"{leaf}: TP={tp_leaf[leaf]}, FP={fp_leaf[leaf]}, FN={fn_leaf[leaf]}")
        print(
            f"         P={p_leaf[leaf]:.2f}, R={r_leaf[leaf]:.2f}, F1={f1_leaf[leaf]:.2f}"
        )

    assert tp_leaf.get("CWE-33") == 1
    assert fp_leaf.get("CWE-11") == 1
    assert fp_leaf.get("CWE-222") == 1
    assert fp_leaf.get("CWE-44") == 1
    assert fn_leaf.get("CWE-111") == 1
    assert fn_leaf.get("CWE-2222") == 1

    print("\nComplex Metrics Test Passed!")


def run_tests():
    # Create dummy CWEs
    # Structure:
    # 1 (Pillar) -> 2 (Class) -> 3 (Base) -> 5 (Variant)
    # 1 (Pillar) -> 4 (Class) -> 3 (Base)

    cwe1 = CWE(
        ID=1, Name="CWE-1", Abstraction="Pillar", Mapping="Allowed", Child=[2, 4]
    )
    cwe2 = CWE(ID=2, Name="CWE-2", Abstraction="Class", Mapping="Allowed", Child=[3])
    cwe4 = CWE(ID=4, Name="CWE-4", Abstraction="Class", Mapping="Allowed", Child=[3])
    cwe3 = CWE(ID=3, Name="CWE-3", Abstraction="Base", Mapping="Allowed", Child=[5])
    cwe5 = CWE(ID=5, Name="CWE-5", Abstraction="Variant", Mapping="Allowed", Child=[])

    # Create DB
    db_dict = {
        "CWE-1": cwe1,
        "CWE-2": cwe2,
        "CWE-4": cwe4,
        "CWE-3": cwe3,
        "CWE-5": cwe5,
    }
    db = CWEDatabase(root=db_dict)

    # Initialize Handler
    handler = CWEHandler(db)

    # Test get_direct_children
    children_1 = [c.ID for c in handler.get_direct_children(1)]
    print(f"Children of 1: {children_1}")
    assert set(children_1) == {2, 4}

    # Test get_direct_parents
    parents_3 = [c.ID for c in handler.get_direct_parents(3)]
    print(f"Parents of 3: {parents_3}")
    assert set(parents_3) == {2, 4}

    # Test get_all_descendants
    descendants_1 = [c.ID for c in handler.get_all_descendants(1)]
    print(f"Descendants of 1: {descendants_1}")
    assert set(descendants_1) == {2, 4, 3, 5}

    # Test get_root_parents
    roots_5 = [c.ID for c in handler.get_root_parents(5)]
    print(f"Roots of 5: {roots_5}")
    assert set(roots_5) == {1}

    # Test is_related
    print(f"Is 1 related to 5? {handler.is_related(1, 5)}")
    assert handler.is_related(1, 5) == True

    print(f"Is 5 related to 1? {handler.is_related(5, 1)}")
    assert handler.is_related(5, 1) == True

    print(f"Is 2 related to 4? {handler.is_related(2, 4)}")
    assert handler.is_related(2, 4) == False

    print("All tests passed!")
    test_metrics()


if __name__ == "__main__":
    run_tests()
