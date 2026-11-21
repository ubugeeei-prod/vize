use super::*;
use std::{hint::black_box, time::Instant};
use vize_patina::{Fix, LintDiagnostic, TextEdit};

fn fixture(edits: Vec<TextEdit>) -> Vec<LintDiagnostic> {
    vec![
        LintDiagnostic::warn("test/selection", "fixture", 0, 0)
            .with_fix(Fix::with_edits("edits", edits)),
    ]
}

fn profile(label: &str, source: &str, messages: Vec<LintDiagnostic>) {
    let result = LintResult {
        filename: "a.vue".into(),
        warning_count: messages.len(),
        error_count: 0,
        diagnostics: messages,
    };
    compare(source, &result);
    let time = |operation: fn(&str, &LintResult) -> Option<String>| {
        let start = Instant::now();
        for _ in 0..256 {
            black_box(operation(black_box(source), black_box(&result)));
        }
        start.elapsed().as_nanos() / 256
    };
    let mut old = vec![];
    let mut new = vec![];
    let mut ratios = vec![];
    time(apply_lint_fixes);
    time(selected);
    for i in 0..31 {
        let (a, b) = if i % 2 == 0 {
            (time(apply_lint_fixes), time(selected))
        } else {
            let b = time(selected);
            (time(apply_lint_fixes), b)
        };
        old.push(a);
        new.push(b);
        ratios.push(b as f64 / a as f64);
    }
    for (pair, (a, b)) in old.iter().zip(&new).enumerate() {
        let first = if pair % 2 == 0 {
            "original"
        } else {
            "selected"
        };
        eprintln!(
            "sample {label} pair={pair} first={first} iterations=256 original_ns={a} selected_ns={b} selected_over_original={:.6}",
            *b as f64 / *a as f64
        );
    }
    old.sort_unstable();
    new.sort_unstable();
    ratios.sort_by(f64::total_cmp);
    eprintln!(
        "{label}: original_p50_ns={} selected_p50_ns={} original_p95_ns={} selected_p95_ns={} median_ratio={:.3}",
        old[15], new[15], old[29], new[29], ratios[15]
    );
}

#[test]
#[ignore = "paired optimized comparison of the real CLI application with a versioned selection"]
fn paired_fix_batches() {
    profile("empty", "one short source", vec![]);
    profile(
        "no-fix",
        "one short source",
        vec![LintDiagnostic::warn("test/no-fix", "fixture", 0, 3)],
    );
    profile("invalid", "αβ", fixture(vec![TextEdit::replace(0, 1, "a")]));
    profile(
        "one",
        "one short source",
        fixture(vec![TextEdit::replace(0, 3, "a")]),
    );
    profile(
        "two",
        "one short source",
        fixture(vec![
            TextEdit::insert(0, "!"),
            TextEdit::replace(4, 9, "longer"),
        ]),
    );
    let source = "0123456789".repeat(128);
    for count in [16, 128] {
        let mut edits = Vec::new();
        for i in (0..count).rev() {
            edits.push(TextEdit::replace(i * 10, i * 10 + 1, "updated"));
        }
        profile(&vize_l0::cstr!("{count} reversed"), &source, fixture(edits));
    }
}
