//! The rule-fixture plane: every template the rule unit tests lint, read off
//! their sources.
//!
//! The rules' own fixtures are the templates rule authors chose to exercise
//! their rule, so they are the densest sample of what the facade must answer
//! for. Each `lint_template(<literal>` call and each
//! `run_over_template(<rule>, <literal>` call under `src/rules/` and
//! `src/markup/tests/` contributes its literal; calls whose argument is not a
//! literal are skipped (the literal census is pinned, so the plane cannot
//! shrink silently).

use std::path::{Path, PathBuf};
use vize_s0::{String, cstr};

/// `(file:line, template)` for every literal template, deduplicated by source
/// and sorted by it.
pub fn rule_fixture_templates() -> std::vec::Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = std::vec::Vec::new();
    collect_rs(&root.join("rules"), &mut files);
    collect_rs(&root.join("markup/tests"), &mut files);
    files.sort();
    let mut fixtures = std::vec::Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("rule source is readable");
        let label = file.strip_prefix(&root).unwrap_or(&file).display();
        for (marker, skip_first_arg) in [("lint_template(", false), ("run_over_template(", true)] {
            let mut from = 0;
            while let Some(found) = text[from..].find(marker) {
                let mut at = from + found + marker.len();
                from = at;
                if skip_first_arg {
                    let Some(comma) = text[at..].find(',') else {
                        continue;
                    };
                    at += comma + 1;
                }
                if let Some(literal) = string_literal(text[at..].trim_start()) {
                    let line = text[..from].matches('\n').count() + 1;
                    fixtures.push((cstr!("{label}:{line}"), literal));
                }
            }
        }
    }
    fixtures.sort_by(|left, right| left.1.cmp(&right.1));
    fixtures.dedup_by(|later, earlier| later.1 == earlier.1);
    fixtures
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

/// Parse one Rust string literal (raw `r#"…"#` or cooked `"…"`) at the start
/// of `text`.
fn string_literal(text: &str) -> Option<String> {
    if let Some(raw) = text.strip_prefix('r') {
        let hashes = raw.bytes().take_while(|byte| *byte == b'#').count();
        let body = raw[hashes..].strip_prefix('"')?;
        let mut search = 0;
        while let Some(quote) = body[search..].find('"') {
            let end = search + quote;
            let tail = &body.as_bytes()[end + 1..];
            if tail.len() >= hashes && tail[..hashes].iter().all(|byte| *byte == b'#') {
                return Some(String::from(&body[..end]));
            }
            search = end + 1;
        }
        return None;
    }
    let body = text.strip_prefix('"')?;
    let mut out = String::default();
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '0' => out.push('\0'),
                '\n' => chars = chars.as_str().trim_start().chars(),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}
