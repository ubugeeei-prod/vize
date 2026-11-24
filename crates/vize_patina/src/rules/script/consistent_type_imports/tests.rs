//! Tests for `script/consistent-type-imports`.

mod fixes;
mod options;
mod parity;
mod script;
mod sfc;
mod template_uses;

use serde_json::Value;
use vize_l0::config::ConfigLintRuleOptions;

use crate::{LintPreset, LintResult, Linter};

const RULE: &str = "script/consistent-type-imports";

const ALL_TYPES: &str = "All imports in the declaration are only used as types. Use `import type`.";

fn linter(options: Option<Value>) -> Linter {
    let linter =
        Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]));
    match options {
        Some(options) => {
            let options: ConfigLintRuleOptions =
                serde_json::from_value(serde_json::json!({ RULE: options })).unwrap();
            linter.with_config_rule_options(&options)
        }
        None => linter,
    }
}

fn lint(source: &str, filename: &str, options: Option<Value>) -> LintResult {
    let linter = linter(options);
    if filename.rsplit('.').next() == Some("vue") {
        linter.lint_sfc(source, filename)
    } else {
        linter.lint_script(source, filename)
    }
}

/// `(1-based line, message)` of each report, in source order.
fn reports(source: &str, filename: &str, options: Option<Value>) -> Vec<(usize, String)> {
    let result = lint(source, filename, options);
    let mut found: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, RULE);
            let line = source
                .bytes()
                .take(diagnostic.start as usize)
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            (diagnostic.start, line, diagnostic.message.to_string())
        })
        .collect();
    found.sort();
    found
        .into_iter()
        .map(|(_, line, message)| (line, message))
        .collect()
}

fn ts(source: &str) -> Vec<(usize, String)> {
    reports(source, "module.ts", None)
}

fn vue(source: &str) -> Vec<(usize, String)> {
    reports(source, "Component.vue", None)
}

fn report(line: usize, message: &str) -> (usize, String) {
    (line, message.to_string())
}

fn nothing() -> Vec<(usize, String)> {
    Vec::new()
}

/// The source after every fix is applied the way `vize lint --fix` applies
/// them: non-overlapping edits per pass, linted again until nothing changes.
fn fixed(source: &str, filename: &str, options: Option<Value>) -> String {
    let mut source = source.to_string();
    for _ in 0..10 {
        let result = lint(&source, filename, options.clone());
        let mut edits: Vec<_> = result
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.fix.as_ref())
            .flat_map(|fix| fix.edits.iter())
            .collect();
        edits.sort_by_key(|edit| (edit.start, edit.end));
        let mut next = source.clone();
        let mut applied = Vec::new();
        let mut last_end = 0;
        for edit in edits {
            if edit.start >= last_end {
                last_end = edit.end;
                applied.push(edit);
            }
        }
        for edit in applied.into_iter().rev() {
            next.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        }
        if next == source {
            break;
        }
        source = next;
    }
    source
}
