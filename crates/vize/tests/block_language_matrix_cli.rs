#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::string_slice,
    reason = "whole-product fixtures use std strings"
)]

//! Every product reads a block's `lang` through the one block-language
//! registry (`vize_l1::container::lang`), so each alias of a language is
//! the same input to the compiler, `vize check`, the linter, the formatter
//! and the language server: `pug`, `jade`, `PUG` and `Jade`; `css`,
//! `postcss` and an absent `lang`; `stylus` and `styl`; any letter case.
//! Positions are compared as line/column so a longer spelling in the open
//! tag does not shift them.

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

use crate::support::{corsa_requirement, lsp_vue_project::Fixture};

const TEMPLATE_ALIASES: [&str; 4] = ["pug", "jade", "PUG", "Jade"];

/// One SFC whose pug template has a lint finding (`v-for` without a key) and
/// a type error (`count.trim()`), both inside the pug body.
fn pug_sfc(lang: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nconst items = [1, 2];\nconst count = 1;\n</script>\n\n\
         <template lang=\"{lang}\">\nul\n  li(v-for=\"item in items\") {{{{ item }}}}\n  \
         p(:title=\"count.trim()\") {{{{ count }}}}\n</template>\n"
    )
}

/// Style aliases by the language the registry resolves them to, each with
/// a body in that language holding one color literal.
const STYLE_GROUPS: [(&[Option<&str>], &str); 5] = [
    (
        &[
            None,
            Some("css"),
            Some("CSS"),
            Some("postcss"),
            Some("PostCSS"),
        ],
        ".a { color: #ff0000; }\n",
    ),
    (
        &[Some("scss"), Some("SCSS")],
        "$c: #ff0000;\n.a {\n  // note\n  color: $c;\n}\n",
    ),
    (&[Some("sass"), Some("Sass")], ".a\n  color: #ff0000\n"),
    (
        &[Some("less"), Some("LESS")],
        "@c: #ff0000;\n.a {\n  // note\n  color: @c;\n}\n",
    ),
    (
        &[Some("stylus"), Some("styl"), Some("Stylus"), Some("STYL")],
        ".a\n  color #ff0000\n",
    ),
];

fn style_sfc(lang: Option<&str>, body: &str) -> String {
    let lang = lang.map_or(String::new(), |lang| format!(" lang=\"{lang}\""));
    format!(
        "<script setup lang=\"ts\">\nconst ok = true;\n</script>\n\n\
         <template><div class=\"a\" :title=\"$style.a\" /></template>\n\n\
         <style module{lang}>\n{body}</style>\n"
    )
}

/// Line and column (both 0-based) of a byte offset.
fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.matches('\n').count();
    (
        line,
        prefix.len() - prefix.rfind('\n').map_or(0, |at| at + 1),
    )
}

/// The source with its open tag's `lang` attribute dropped, so outputs that
/// echo the authored tag compare equal across spellings.
fn without_lang(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(" lang=\"") {
        out.push_str(&rest[..at]);
        let value = &rest[at + 7..];
        let end = value.find('"').map_or(value.len(), |end| end + 1);
        let spelled = &value[..end.saturating_sub(1)];
        // Script `lang="ts"` is not part of the matrix; keep it.
        if spelled == "ts" {
            out.push_str(&rest[at..at + 7 + end]);
        }
        rest = &value[end..];
    }
    out.push_str(rest);
    out
}

fn compiled(source: &str) -> (String, Option<String>) {
    let parse = SfcParseOptions {
        filename: "App.vue".into(),
        ..SfcParseOptions::default()
    };
    let descriptor = parse_sfc(source, parse).expect("SFC parses");
    let mut options = SfcCompileOptions::default();
    options.parse.filename = "App.vue".into();
    options.scope_id = Some("data-v-matrix".into());
    options.style.id = "data-v-matrix".into();
    let result = compile_sfc(&descriptor, options).expect("SFC compiles");
    assert_eq!(result.errors.len(), 0, "{:?}", result.errors);
    (
        result.code.to_string(),
        result.css.map(|css| css.to_string()),
    )
}

fn linted(source: &str) -> Vec<(String, (usize, usize), (usize, usize))> {
    vize_patina::Linter::new()
        .lint_sfc(source, "App.vue")
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name.to_owned(),
                line_column(source, diagnostic.start as usize),
                line_column(source, diagnostic.end as usize),
            )
        })
        .collect()
}

fn formatted(source: &str) -> String {
    let result =
        vize_glyph::format_sfc(source, &vize_glyph::FormatOptions::default()).expect("SFC formats");
    without_lang(&result.code)
}

/// Assert every value equals the first, naming the spelling that diverged.
fn all_equal<T: PartialEq + std::fmt::Debug>(product: &str, values: &[(String, T)]) {
    let (first_spelling, first) = &values[0];
    for (spelling, value) in &values[1..] {
        assert_eq!(
            value, first,
            "{product}: lang {spelling} diverges from {first_spelling}"
        );
    }
}

#[test]
fn every_template_alias_compiles_lints_and_formats_as_pug() {
    let mut compiles = Vec::new();
    let mut lints = Vec::new();
    let mut formats = Vec::new();
    for lang in TEMPLATE_ALIASES {
        let source = pug_sfc(lang);
        compiles.push((lang.to_owned(), compiled(&source)));
        lints.push((lang.to_owned(), linted(&source)));
        formats.push((lang.to_owned(), formatted(&source)));
    }
    all_equal("compile", &compiles);
    all_equal("lint", &lints);
    all_equal("format", &formats);

    // The pug grammar read the body: the key finding sits on the `li` head
    // of the pug source, which an HTML reading of the body would not have.
    let source = pug_sfc("pug");
    let li = line_column(&source, source.find("li(").unwrap());
    assert_eq!(
        lints[0].1,
        [("vue/require-v-for-key".to_owned(), li, (li.0, li.1 + 2))]
    );
    // A lang no grammar reads is never read as pug or as HTML: the linter
    // skips the template, the compiler refuses it at the authored value.
    let haml = pug_sfc("haml");
    assert_eq!(linted(&haml), []);
    let descriptor = parse_sfc(&haml, SfcParseOptions::default()).expect("SFC parses");
    let error = compile_sfc(&descriptor, SfcCompileOptions::default()).expect_err("refused");
    assert_eq!(error.code.as_deref(), Some("TEMPLATE_LANG_UNSUPPORTED"));
    let loc = error.loc.expect("located");
    assert_eq!(&haml[loc.start..loc.end], "haml");
}

#[test]
fn every_style_alias_compiles_lints_and_formats_as_its_language() {
    for (aliases, body) in STYLE_GROUPS {
        let mut compiles = Vec::new();
        let mut lints = Vec::new();
        let mut formats = Vec::new();
        for lang in aliases {
            let spelling = lang.unwrap_or("(absent)").to_owned();
            let source = style_sfc(*lang, body);
            compiles.push((spelling.clone(), compiled(&source)));
            lints.push((spelling.clone(), linted(&source)));
            formats.push((spelling, formatted(&source)));
        }
        all_equal("compile", &compiles);
        all_equal("lint", &lints);
        all_equal("format", &formats);
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

/// `vize check --format json` diagnostics of one file, as authored text.
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
fn every_template_and_style_alias_checks_the_same() {
    let Some(corsa) = corsa_requirement::required_or_skip(None::<String>) else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    link_workspace_vue(root.path());
    let mut templates = Vec::new();
    for (index, lang) in TEMPLATE_ALIASES.into_iter().enumerate() {
        let name = format!("Pug{index}.vue");
        fs::write(root.path().join(&name), pug_sfc(lang)).unwrap();
        templates.push((lang.to_owned(), checked(root.path(), &corsa, &name)));
    }
    all_equal("check", &templates);
    assert_eq!(templates[0].1.len(), 2, "{:?}", templates[0].1);

    for (group, (aliases, body)) in STYLE_GROUPS.into_iter().enumerate() {
        let mut styles = Vec::new();
        for (index, lang) in aliases.iter().enumerate() {
            let name = format!("Style{group}x{index}.vue");
            let source = style_sfc(*lang, body).replace("$style.a", "$style.missing");
            fs::write(root.path().join(&name), source).unwrap();
            let spelling = lang.unwrap_or("(absent)").to_owned();
            styles.push((spelling, checked(root.path(), &corsa, &name)));
        }
        all_equal("check", &styles);
    }
}

#[test]
fn every_template_and_style_alias_serves_the_same_editor_answers() {
    let first = pug_sfc(TEMPLATE_ALIASES[0]);
    let mut fixture = Fixture::new_with_vue(&first);
    let mut version = 1;
    let mut templates = Vec::new();
    for lang in TEMPLATE_ALIASES {
        let source = pug_sfc(lang);
        let diagnostics = if version == 1 {
            fixture.open(&source)
        } else {
            fixture.change(&source, version)
        };
        version += 1;
        let hover = fixture.request("textDocument/hover", &source, "count }}");
        let definition = fixture.request("textDocument/definition", &source, "items\")");
        templates.push((lang.to_owned(), json!([diagnostics, hover, definition])));
    }
    all_equal("lsp", &templates);
    let answers = &templates[0].1;
    assert_eq!(answers[0].as_array().map(Vec::len), Some(1), "{answers:#}");
    assert_eq!(
        answers[1]["contents"]["value"],
        "```typescript\nconst count: 1\n```"
    );

    for (aliases, body) in STYLE_GROUPS {
        let mut styles = Vec::new();
        for lang in aliases {
            let source = style_sfc(*lang, body);
            let diagnostics = fixture.change(&source, version);
            version += 1;
            let colors = fixture.request("textDocument/documentColor", &source, "ff0000");
            let folding = fixture.request("textDocument/foldingRange", &source, ".a");
            let spelling = lang.unwrap_or("(absent)").to_owned();
            styles.push((spelling, json!([diagnostics, colors, folding])));
        }
        all_equal("lsp", &styles);
        assert_eq!(styles[0].1[1].as_array().map(Vec::len), Some(1), "{body}");
    }
    fixture.shutdown();
}
