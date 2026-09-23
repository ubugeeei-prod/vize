//! TS-39 for the script-registry installment (P4-8a).
//!
//! Each directory under `tests/fixtures/parity/` is one rule. `sfc.vue` and
//! `jsx.tsx` carry the same program. `lint_sfc` and `lint_jsx` must report
//! that rule with the same messages.

use std::fs;
use std::path::{Path, PathBuf};

use super::Linter;
use crate::LintPreset;
use vize_atelier_jsx::JsxLang;
use vize_s0::String as CompactString;

fn parity_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parity")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("parity entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if path.join("sfc.vue").is_file() {
                out.push(path);
            } else {
                collect(&path, out);
            }
        }
    }
}

fn reports(diagnostics: &[crate::diagnostic::LintDiagnostic], rule: &str) -> Vec<String> {
    let mut messages = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == rule)
        .map(|diagnostic| diagnostic.message.to_string())
        .collect::<Vec<_>>();
    messages.sort();
    messages
}

#[test]
fn script_parity_fixtures_report_the_same_diagnostics() {
    let root = parity_root();
    let mut dirs = Vec::new();
    collect(&root, &mut dirs);
    assert!(!dirs.is_empty(), "parity fixtures are missing");
    let mut failures = Vec::new();
    for dir in dirs {
        let rule = dir
            .strip_prefix(&root)
            .expect("fixture path")
            .to_string_lossy()
            .replace('\\', "/");
        let sfc = fs::read_to_string(dir.join("sfc.vue")).expect("sfc.vue");
        let jsx = fs::read_to_string(dir.join("jsx.tsx")).expect("jsx.tsx");
        let enabled = Some(vec![CompactString::from(rule.as_str())]);
        let sfc_result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(enabled.clone())
            .lint_sfc(&sfc, "parity.vue");
        let jsx_result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(enabled)
            .lint_jsx(&jsx, "parity.tsx", JsxLang::Tsx);
        let sfc_messages = reports(&sfc_result.diagnostics, &rule);
        let jsx_messages = reports(&jsx_result.diagnostics, &rule);
        if sfc_messages.is_empty() {
            failures.push(format!("{rule} did not fire"));
            continue;
        }
        if sfc_messages != jsx_messages {
            failures.push(format!(
                "{rule} diverged\n  sfc {sfc_messages:?}\n  jsx {jsx_messages:?}"
            ));
            continue;
        }
        let actual = sfc_messages.join("\n") + "\n";
        let expected = fs::read_to_string(dir.join("expected.txt")).unwrap_or_default();
        if expected != actual {
            failures.push(format!("{rule} snapshot\n{actual}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
