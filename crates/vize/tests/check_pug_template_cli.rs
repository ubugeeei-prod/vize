#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "whole CLI fixtures use std strings"
)]

//! `vize check` reads a Pug template through the same lane as HTML: attribute
//! and directive expressions are type-checked and reported at their authored
//! Pug positions (#8908).

use crate::support::corsa_requirement;

use std::{fs, path::Path, process::Command};

const SCRIPT: &str = "<script setup lang=\"ts\">\nconst count = 1;\nfunction inc(step: number) { return step; }\n</script>\n\n";

const PUG: &str = "<template lang=\"pug\">\ndiv\n  p(:title=\"count.trim()\") {{ count.trim() }}\n  button(@click=\"inc('x')\") add\n</template>\n";

const HTML: &str = "<template>\n<div>\n  <p :title=\"count.trim()\">{{ count.trim() }}</p>\n  <button @click=\"inc('x')\">add</button>\n</div>\n</template>\n";

fn link_workspace_vue(root: &Path) {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/node_modules");
    fs::create_dir_all(root.join("node_modules")).unwrap();
    for package in ["vue", "@vue"] {
        let source = workspace.join(package);
        if source.exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source, root.join("node_modules").join(package)).unwrap();
        }
    }
}

/// `(line, column, code)` of every error, with the column of its token's start.
fn errors(root: &Path, corsa: &str, file: &str) -> Vec<(usize, usize, String)> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .args(["check", file, "--format", "json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    report["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .filter_map(|diagnostic| {
            let rest = diagnostic.as_str()?.strip_prefix("error:")?;
            let (line, rest) = rest.split_once(':')?;
            let (column, rest) = rest.split_once(' ')?;
            let code = rest.split_once(']')?.0.trim_start_matches('[');
            Some((line.parse().ok()?, column.parse().ok()?, code.to_owned()))
        })
        .collect()
}

fn position(source: &str, needle: &str, nth: usize) -> (usize, usize) {
    let offset = source.match_indices(needle).nth(nth).unwrap().0;
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    (
        source[..offset].matches('\n').count() + 1,
        offset - line_start + 1,
    )
}

#[test]
fn pug_attribute_expressions_report_like_html_at_authored_positions() {
    let Some(corsa) = corsa_requirement::required_or_skip(None::<String>) else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    link_workspace_vue(root.path());
    for (name, template) in [("Pug.vue", PUG), ("Html.vue", HTML)] {
        let source = format!("{SCRIPT}{template}");
        fs::write(root.path().join(name), &source).unwrap();
        let expected = [
            ("trim", 0, "TS2339"),
            ("trim", 1, "TS2339"),
            ("'x'", 0, "TS2345"),
        ]
        .map(|(needle, nth, code)| {
            let (line, column) = position(&source, needle, nth);
            (line, column, code.to_owned())
        })
        .to_vec();
        assert_eq!(errors(root.path(), &corsa, name), expected, "{name}");
    }
}
