use super::*;
use serde_json::json;
use std::{hint::black_box, time::Instant};

fn run(label: &str, n: usize, ignored: bool) {
    let root = std::env::current_dir().unwrap();
    let path = root.join(if ignored {
        "ignored/Panel.vue"
    } else {
        "src/part0/Panel.vue"
    });
    let entries: Vec<_> = (0..n)
        .map(|i| {
            json!({
                "files": [crate::cstr!("src/part{i}/**"), "shared/**"],
                "compiler": {"whitespace": if i % 2 == 0 { "preserve" } else { "condense" }},
                "linter": {"rules": {"vue/v-bind-style": "warn"}}
            })
        })
        .collect();
    let document: ConfigDocument =
        serde_json::from_value(json!({"entries":entries, "ignores":["ignored/**"]})).unwrap();
    let rules = document.linter_plan_with_config_rule_options();
    let modes = document.compiler_entries();
    let mode = document.compiler_whitespace();
    let separate = || {
        (
            FormatterLinterPlan::new(rules.clone(), &root),
            CompilerWhitespacePlan::new(mode, &modes, &root),
        )
    };
    let combined = || ProjectFilePlan::new(rules.clone(), mode, &modes, &root);
    let (lint, whitespace) = separate();
    let plan = combined();
    let expected = FilePolicy {
        lint: lint.resolve(&path),
        preserves_text: whitespace.preserves_text_for(&path),
    };
    assert_eq!(expected, plan.resolve(&path));
    assert_eq!(plan.scopes.len(), n);
    measure(
        label,
        "prepare",
        if n < 2 { 32 } else { 4 },
        || {
            black_box(separate());
        },
        || {
            black_box(combined());
        },
    );
    measure(
        label,
        "resolve",
        if n < 2 { 128 } else { 32 },
        || {
            black_box((
                lint.resolve(black_box(&path)),
                whitespace.preserves_text_for(black_box(&path)),
            ));
        },
        || {
            black_box(plan.resolve(black_box(&path)));
        },
    );
}

fn measure(label: &str, operation: &str, iterations: u128, old: impl Fn(), new: impl Fn()) {
    let time = |f: &dyn Fn()| {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        start.elapsed().as_nanos() / iterations
    };
    old();
    new();
    let mut original = Vec::new();
    let mut shared = Vec::new();
    for pair in 0..31 {
        let (a, b) = if pair % 2 == 0 {
            (time(&old), time(&new))
        } else {
            let b = time(&new);
            (time(&old), b)
        };
        original.push(a);
        shared.push(b);
    }
    let mut ratios = Vec::new();
    for (pair, (a, b)) in original.iter().zip(&shared).enumerate() {
        let first = if pair % 2 == 0 {
            "independent"
        } else {
            "shared"
        };
        let ratio = *b as f64 / *a as f64;
        ratios.push(ratio);
        eprintln!(
            "sample {label} {operation} pair={pair} first={first} iterations={iterations} independent_ns={a} shared_ns={b} shared_over_independent={ratio:.6}"
        );
    }
    original.sort_unstable();
    shared.sort_unstable();
    ratios.sort_by(f64::total_cmp);
    eprintln!(
        "{label} {operation} p50_ns={}/{} p95_ns={}/{} paired={:.3}",
        original[15], shared[15], original[29], shared[29], ratios[15]
    );
}

#[test]
#[ignore = "paired optimized configuration planning and file lookup"]
fn paired_project_file_policy() {
    run("root", 0, false);
    run("one", 1, false);
    run("24", 24, false);
    run("24-ignored", 24, true);
}
