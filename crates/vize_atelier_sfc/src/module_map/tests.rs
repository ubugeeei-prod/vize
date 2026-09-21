//! Exact provenance algebra and map emission for the SFC module map.

use super::{Run, Runs, apply_edits, build_module_map, edit_runs, replace_traced, token_starts};

fn copies(runs: &Runs) -> Vec<(usize, usize, usize)> {
    runs.runs()
        .iter()
        .map(|run| (run.out, run.src, run.len))
        .collect()
}

#[test]
fn contiguous_copies_merge_and_slices_rebase() {
    let mut runs = Runs::default();
    runs.copy(0, 10, 3);
    runs.copy(3, 13, 2);
    runs.copy(7, 40, 4);
    runs.point(5, 99);
    assert_eq!(copies(&runs), [(0, 10, 5), (7, 40, 4)]);
    let sliced = runs.slice(2, 7);
    assert_eq!(copies(&sliced), [(0, 12, 3), (5, 40, 2)]);
    assert_eq!(sliced.points(), [(3, 99)]);
    assert_eq!(
        [0, 4, 5, 8, 11].map(|pos| runs.lookup(pos)),
        [Some(10), Some(14), Some(99), Some(41), None]
    );
}

#[test]
fn composition_follows_bytes_through_both_stages() {
    // C[2..6] is a copy of B[0..4], and B[2..6] a copy of A[0..4], so C[4..6]
    // copies A[0..2]. B[0..2] is synthesized, with B[1] anchored at A[30].
    let mut upper = Runs::default();
    upper.copy(2, 0, 4);
    upper.point(0, 1);
    upper.point(1, 5);
    let mut lower = Runs::default();
    lower.copy(2, 0, 4);
    lower.point(1, 30);
    let composed = upper.compose(&lower);
    assert_eq!(copies(&composed), [(4, 0, 2)]);
    // C[3] copies B[1]; C[0] points at B[1]; C[1] points into B's copy of A.
    assert_eq!(composed.points(), [(3, 30), (0, 30), (1, 3)]);
}

#[test]
fn edits_copy_untouched_text_and_anchor_replacements() {
    let (text, runs) = apply_edits("let a = f(b)", [(4, 5, "__props.a"), (10, 11, "")]);
    assert_eq!(text, "let __props.a = f()");
    assert_eq!(copies(&runs), [(0, 0, 4), (13, 5, 5), (18, 11, 1)]);
    assert_eq!(runs.points(), [(4, 4)]);
    assert_eq!(edit_runs(12, &[(4, 5, 9), (10, 11, 0)]), runs);
    let (replaced, replace_runs) = replace_traced("\tx\t", "\t", "  ");
    assert_eq!(replaced, "  x  ");
    assert_eq!(copies(&replace_runs), [(2, 1, 1)]);
    assert_eq!(replace_runs.points(), [(0, 0), (3, 2)]);
}

#[test]
fn verification_drops_runs_whose_bytes_differ() {
    let mut runs = Runs::default();
    runs.copy(0, 0, 3);
    runs.copy(4, 4, 3);
    runs.point(3, 3);
    runs.point(9, 3);
    let kept = runs.verified("abc_xyz", "abc-XYZ");
    assert_eq!(copies(&kept), [(0, 0, 3)]);
    assert_eq!(kept.points(), [(3, 3)]);
}

#[test]
fn verbatim_runs_map_every_token_start() {
    let generated = "  count.value += step(2)";
    let run = Run {
        out: 0,
        src: 100,
        len: generated.len(),
    };
    let starts: Vec<_> = token_starts(generated, run).collect();
    assert_eq!(starts, [2, 7, 8, 14, 15, 17, 21, 22, 23]);
}

#[test]
fn module_map_emits_verified_token_segments_and_points() {
    let source = "<script>\nconst a = 1\n</script>";
    let generated = "import x\nconst a = 1\n";
    let mut runs = Runs::default();
    runs.copy(9, 9, 11);
    runs.point(0, 0);
    let json = build_module_map(generated, runs, "A.vue", source).unwrap();
    let map: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(map["sources"], serde_json::json!(["A.vue"]));
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    // `import` -> `<script>`; `const`, `a`, `=`, `1` -> the authored tokens.
    assert_eq!(map["mappings"], "AAAA;AACA,MAAM,EAAE,EAAE");
    assert_eq!(
        build_module_map(generated, Runs::default(), "A.vue", source),
        None
    );
}
