//! Candidate histories belong to freshness tracking, not ordinary graph walks.

use super::{probe_candidates_inner, resolve_dependency_inner, resolve_dependency_with_inputs};

#[test]
fn runtime_substitution_retains_exact_tracking_history_without_history_on_the_fast_path() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let runtime = root.join("model.js");
    let typed = root.join("model.ts");
    std::fs::write(&runtime, "export const value = 'runtime';").unwrap();
    std::fs::write(&typed, "export const value: number = 1;").unwrap();
    assert_eq!(
        probe_candidates_inner::<true>(&runtime),
        (Some(typed.clone()), vec![runtime.clone(), typed.clone()])
    );
    assert_eq!(
        probe_candidates_inner::<false>(&runtime),
        (Some(typed), Vec::new())
    );
}

#[test]
fn missing_module_keeps_every_ordered_candidate_only_for_tracking() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().join("missing");
    let mut expected = vec![base.clone()];
    for suffix in [
        "ts", "tsx", "d.ts", "mts", "d.mts", "cts", "d.cts", "vue", "js", "jsx", "mjs", "cjs",
    ] {
        expected.push(base.with_extension(suffix));
    }
    for name in [
        "index.ts",
        "index.tsx",
        "index.d.ts",
        "index.mts",
        "index.d.mts",
        "index.cts",
        "index.d.cts",
        "index.vue",
        "index.js",
        "index.jsx",
        "index.mjs",
        "index.cjs",
    ] {
        expected.push(base.join(name));
    }
    assert_eq!(probe_candidates_inner::<true>(&base), (None, expected));
    assert_eq!(probe_candidates_inner::<false>(&base), (None, Vec::new()));
}

#[test]
fn alias_fallback_and_vue_fold_preserve_resolution_and_tracking_order() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir_all(root.join("first")).unwrap();
    std::fs::create_dir_all(root.join("second")).unwrap();
    let target = root.join("second/Component.vue");
    std::fs::write(&target, "<template />").unwrap();
    let aliases = vec![
        ("@/*".into(), "first/*".into()),
        ("@/*".into(), "second/*".into()),
    ];
    let expected_inputs = vec![root.join("first/Component.vue"), target.clone()];
    assert_eq!(
        resolve_dependency_with_inputs("@/Component.vue.ts", root, root, &aliases),
        (Some(target.clone()), expected_inputs)
    );
    assert_eq!(
        resolve_dependency_inner::<false>("@/Component.vue.ts", root, root, &aliases),
        (Some(target), Vec::new())
    );
}
