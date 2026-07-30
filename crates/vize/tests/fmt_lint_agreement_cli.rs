//! `vize fmt` must not change which lint findings a file produces.
//!
//! The corpus-wide `glyph lint-agreement` property guards this over hydrated
//! fixtures; this test pins the specific way it broke in #3343 — the formatter
//! split a line covered by `eslint-disable-next-line`, so the suppression stopped
//! applying and `a11y/form-control-has-label` went from 2 findings to 3.

use std::{collections::BTreeMap, fs, path::Path, process::Command};

/// The reduced `gogocode` keycode-modifiers fixture from the report: three
/// unlabelled `<input>` elements, the third suppressed by a comment on the line
/// above, sharing its line with the `custom space:` text.
const SOURCE: &str = r#"<template>
  <div class="mt20 text-left">
    <div>space: <input type="text" v-on:keyup.space="keys('space')" /></div>
    <div class="mt20">
      space number 32:
      <input type="text" v-on:keyup.32="keys('keycode 32 space')" />
    </div>
    <div class="mt20">
      <!--eslint-disable-next-line-->
      custom space: <input v-on:keyup.custom="keys('custom keycode space')" />
    </div>
  </div>
</template>
"#;

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Findings per rule id, the same multiset the corpus property compares.
fn findings_by_rule(dir: &Path, file: &str) -> BTreeMap<String, usize> {
    let output = run(
        dir,
        &[
            "lint",
            file,
            "--format",
            "json",
            "--preset",
            "ecosystem",
            "--no-config",
        ],
    );
    let code = output.status.code();
    assert!(
        code == Some(0) || code == Some(1),
        "vize lint exited with {code:?}: {}",
        details(&output)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("lint JSON: {error}: {}", details(&output)));
    let mut counts = BTreeMap::new();
    for entry in report.as_array().expect("lint report is an array") {
        for message in entry["messages"].as_array().into_iter().flatten() {
            let rule = message["ruleId"].as_str().unwrap_or_default().to_owned();
            *counts.entry(rule).or_insert(0) += 1;
        }
    }
    counts
}

#[test]
fn formatting_does_not_introduce_lint_findings_for_a_suppressed_line() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::write(root.join("Comp.vue"), SOURCE).unwrap();

    let before = findings_by_rule(root, "Comp.vue");
    assert_eq!(
        before.get("a11y/form-control-has-label"),
        Some(&2),
        "the suppressed third <input> must not be reported in the source: {before:?}"
    );

    let formatted = run(root, &["fmt", "Comp.vue", "--write", "--no-config"]);
    assert_eq!(formatted.status.code(), Some(0), "{}", details(&formatted));

    let after = findings_by_rule(root, "Comp.vue");
    // Counts may shrink (formatting legitimately fixes findings such as
    // `vue/v-on-style`); any rule whose count grows is the defect.
    let introduced: Vec<_> = after
        .iter()
        .filter(|(rule, count)| *count > before.get(rule.as_str()).unwrap_or(&0))
        .collect();
    assert!(
        introduced.is_empty(),
        "vize fmt introduced findings: {introduced:?}\nbefore: {before:?}\nafter:  {after:?}\n{}",
        fs::read_to_string(root.join("Comp.vue")).unwrap()
    );

    // The suppression still sits on the line it covers, and formatting again
    // changes nothing.
    let output = fs::read_to_string(root.join("Comp.vue")).unwrap();
    assert!(
        output
            .contains("<!--eslint-disable-next-line-->\n      custom space: <input @keyup.custom="),
        "suppressed line was split:\n{output}"
    );
    let again = run(root, &["fmt", "Comp.vue", "--write", "--no-config"]);
    assert_eq!(again.status.code(), Some(0), "{}", details(&again));
    assert_eq!(
        fs::read_to_string(root.join("Comp.vue")).unwrap(),
        output,
        "formatting the joined line must be idempotent"
    );
}
