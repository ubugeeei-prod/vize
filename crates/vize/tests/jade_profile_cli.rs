#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::string_slice,
    reason = "whole-product fixtures use std strings"
)]

//! The jade profile (`<template lang="jade">`) row by row, across products:
//! the compiler accepts what jade 1.x accepts and warns (`JADE_PROFILE`) at
//! the authored construct, the checker and the language server read the same
//! template the compiler does, the formatter keeps the block opaque, and a
//! `lang="pug"` template keeps pug 3's behavior byte for byte.

use std::{fs, path::Path, process::Command};

use serde_json::Value;
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

use crate::support::{corsa_requirement, lsp_vue_project::Fixture};

/// `(pug body, authored construct jade 1.x reads differently, pug 3 refuses it)`.
const ROWS: [(&str, &str, bool); 4] = [
    ("ul\n  - each n in [1, 2]\n    li {{ count }}\n", "- ", true),
    (
        "mixin item(x)\n  p= x\nmixin item('a')\n",
        "mixin item('a')",
        true,
    ),
    (
        "ul\n  each n of [1, 2]\n    li= n\n",
        "each n of [1, 2]",
        false,
    ),
    ("a(href=\"/users/#{count}\") x\n", "#{count}", false),
];

fn sfc(lang: &str, body: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nconst count = 1;\n</script>\n\n<template lang=\"{lang}\">\n{body}</template>\n"
    )
}

fn compiled(source: &str) -> Result<(String, Vec<(String, String)>), String> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC parses");
    match compile_sfc(&descriptor, SfcCompileOptions::default()) {
        Ok(result) => Ok((
            result.code.to_string(),
            result
                .warnings
                .iter()
                .map(|warning| {
                    let loc = warning.loc.as_ref().expect("located");
                    (
                        warning.code.as_deref().unwrap_or_default().to_owned(),
                        source[loc.start..loc.end].to_owned(),
                    )
                })
                .collect(),
        )),
        Err(error) => Err(error.code.as_deref().unwrap_or_default().to_owned()),
    }
}

#[test]
fn every_row_compiles_under_jade_with_a_located_warning() {
    for (body, construct, refused_by_pug) in ROWS {
        let (_, warnings) = compiled(&sfc("jade", body)).expect(body);
        assert_eq!(
            warnings,
            [("JADE_PROFILE".to_owned(), construct.to_owned())],
            "{body}"
        );
        let pug = compiled(&sfc("pug", body));
        if refused_by_pug {
            assert_eq!(pug, Err("PUG_TEMPLATE_ERROR".to_owned()), "{body}");
        } else {
            let (code, warnings) = pug.expect(body);
            assert_eq!(warnings, [], "{body}");
            // Where pug 3 accepts the row, jade compiles to the same code.
            assert_eq!(compiled(&sfc("jade", body)).expect(body).0, code, "{body}");
        }
    }
}

#[test]
fn the_formatter_keeps_both_profiles_opaque_alike() {
    for (body, _, _) in ROWS {
        let format = |lang: &str| {
            vize_glyph::format_sfc(&sfc(lang, body), &vize_glyph::FormatOptions::default())
                .expect("formats")
                .code
                .replace("lang=\"jade\"", "lang=\"pug\"")
        };
        assert_eq!(format("jade"), format("pug"), "{body}");
    }
}

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

fn checked(root: &Path, corsa: &str, file: &str) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .args(["check", file, "--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
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
        .filter_map(|diagnostic| diagnostic.as_str().map(str::to_owned))
        .collect()
}

#[test]
fn the_checker_reads_the_template_the_compiler_reads() {
    let Some(corsa) = corsa_requirement::required_or_skip(None::<String>) else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    link_workspace_vue(root.path());
    for (index, (body, _, refused_by_pug)) in ROWS.into_iter().enumerate() {
        for lang in ["jade", "pug"] {
            let name = format!("Row{index}{lang}.vue");
            fs::write(root.path().join(&name), sfc(lang, body)).unwrap();
            let diagnostics = checked(root.path(), &corsa, &name);
            let refused = lang == "pug" && refused_by_pug;
            assert_eq!(
                diagnostics.is_empty(),
                !refused,
                "{lang} {body}: {diagnostics:?}"
            );
        }
    }
}

#[test]
fn the_language_server_reads_the_template_the_compiler_reads() {
    let (body, _, _) = ROWS[0];
    let jade = sfc("jade", body);
    let mut fixture = Fixture::new_with_vue(&jade);
    assert_eq!(fixture.open(&jade), serde_json::json!([]), "jade iterates");
    let hover = fixture.request("textDocument/hover", &jade, "count }}");
    assert_eq!(
        hover["contents"]["value"], "```typescript\nconst count: 1\n```",
        "{hover:#}"
    );
    // pug 3 refuses the dashed loop: the checker has no template there, so
    // the hover is the editor's syntactic binding summary, not a type.
    let pug = sfc("pug", body);
    fixture.change(&pug, 2);
    let refused = fixture.request("textDocument/hover", &pug, "count }}");
    assert_ne!(
        refused["contents"]["value"], "```typescript\nconst count: 1\n```",
        "{refused:#}"
    );
    fixture.shutdown();
}
