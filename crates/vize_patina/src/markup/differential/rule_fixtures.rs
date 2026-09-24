//! The rule-fixture plane: every template the rule unit tests lint, read off
//! their sources.
//!
//! The rules' own fixtures are the templates rule authors chose to exercise
//! their rule, so they are the densest sample of what the facade must answer
//! for. Each `lint_template(<literal>` call and each
//! `run_over_template(<rule>, <literal>` call under `src/rules/` and
//! `src/markup/tests/` contributes its literal. A call whose template argument
//! is not a string literal is skipped, and both the call count and the skip
//! count are pinned, so a newly skipped call cannot hide behind dedup.

use std::path::{Path, PathBuf};

use vize_s0::{String, cstr};

use super::fixture_syntax;

/// Literal templates plus the call census the battery pins.
pub struct RuleFixtureScan {
    /// `(file:line, template)` deduplicated by template text.
    pub templates: std::vec::Vec<(String, String)>,
    /// Every discovered call, literal or not.
    pub calls: usize,
    /// Calls whose template argument is not a string literal.
    pub skipped: usize,
}

pub fn scan_rule_fixtures() -> RuleFixtureScan {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = std::vec::Vec::new();
    collect_rs(&root.join("rules"), &mut files);
    collect_rs(&root.join("markup/tests"), &mut files);
    files.sort();
    let mut templates = std::vec::Vec::new();
    let mut calls = 0;
    let mut skipped = 0;
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            skipped += 1;
            continue;
        };
        let label = file.strip_prefix(&root).unwrap_or(&file).display();
        for call in fixture_syntax::fixture_calls(&text) {
            calls += 1;
            let Some(literal) = call.template else {
                skipped += 1;
                continue;
            };
            templates.push((cstr!("{label}:{}", call.line), literal));
        }
    }
    templates.sort_by(|left, right| left.1.cmp(&right.1));
    templates.dedup_by(|later, earlier| later.1 == earlier.1);
    RuleFixtureScan {
        templates,
        calls,
        skipped,
    }
}

fn collect_rs(dir: &Path, out: &mut std::vec::Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
