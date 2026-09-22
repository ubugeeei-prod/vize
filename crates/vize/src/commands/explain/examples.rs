//! Fixture examples from each rule's own module docs.
//!
//! The first `### Invalid` fence and the first `### Valid` fence (a
//! parenthetical such as `### Invalid (Vue 3)` counts) are the examples the
//! page shows. The files under `vize_patina/src/rules` are the source; nothing
//! is copied per rule.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use vize_s0::{FxHashMap, String};

pub(crate) struct Fixtures {
    pub(crate) invalid: Option<String>,
    pub(crate) valid: Option<String>,
}

pub(crate) fn get(rule: &str) -> Option<&'static Fixtures> {
    fixtures().get(rule)
}

fn fixtures() -> &'static FxHashMap<String, Fixtures> {
    static FIXTURES: OnceLock<FxHashMap<String, Fixtures>> = OnceLock::new();
    FIXTURES.get_or_init(load)
}

fn load() -> FxHashMap<String, Fixtures> {
    let mut found = FxHashMap::default();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vize_patina/src/rules");
    walk(&root, &mut found);
    found
}

fn walk(dir: &Path, found: &mut FxHashMap<String, Fixtures>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if name != "tests" {
                walk(&path, found);
            }
            continue;
        }
        if name == "tests.rs" || !name.ends_with(".rs") {
            continue;
        }
        let Some(source) = read_source(&path) else {
            continue;
        };
        let Some(rule) = rule_name(&source) else {
            continue;
        };
        let doc = module_doc(&source);
        let invalid = first_example(&doc, "Invalid");
        let valid = first_example(&doc, "Valid");
        if invalid.is_none() && valid.is_none() {
            continue;
        }
        found.insert(rule, Fixtures { invalid, valid });
    }
}

#[allow(clippy::disallowed_types)]
fn read_source(path: &Path) -> Option<String> {
    let owned = fs::read_to_string(path).ok()?;
    Some(String::from(owned.as_str()))
}

/// The `name: "namespace/rule"` a `RuleMeta`-family declaration carries.
fn rule_name(source: &str) -> Option<String> {
    let mut rest = source;
    while let Some(at) = rest.find("name:") {
        rest = &rest["name:".len() + at..];
        let trimmed = rest.trim_start();
        let Some(body) = trimmed.strip_prefix('"') else {
            continue;
        };
        let Some(end) = body.find('"') else {
            continue;
        };
        let name = &body[..end];
        if name.contains('/') && !name.contains(' ') && !name.contains('\\') {
            return Some(String::from(name));
        }
    }
    None
}

fn module_doc(source: &str) -> String {
    let mut doc = String::new("");
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("//!") {
            if !doc.is_empty() {
                doc.push('\n');
            }
            doc.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        } else if trimmed.is_empty() && doc.is_empty() {
            continue;
        } else {
            break;
        }
    }
    doc
}

fn first_example(doc: &str, label: &str) -> Option<String> {
    let mut in_section = false;
    let mut body = String::new("");
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("### ") {
            if in_section {
                break;
            }
            let word = rest.split_whitespace().next().unwrap_or("");
            if word.eq_ignore_ascii_case(label) {
                in_section = true;
            }
            continue;
        }
        if in_section {
            body.push_str(line);
            body.push('\n');
        }
    }
    fence(&body)
}

fn fence(section: &str) -> Option<String> {
    let mut body = String::new("");
    let mut inside = false;
    for line in section.lines() {
        if line.trim().starts_with("```") {
            if inside {
                let trimmed = body.trim_end();
                return (!trimmed.is_empty()).then(|| String::from(trimmed));
            }
            inside = true;
            continue;
        }
        if inside {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(line);
        }
    }
    None
}
