#![allow(clippy::disallowed_types)] // Fixture files arrive through `std::fs` as std strings.
//! The P4-12c compile oracle: compiling an SFC with `<template
//! lang="pug">` equals, byte for byte, compiling the same SFC with its
//! template replaced by the pinned `pug` package's HTML rendering — in the
//! DOM, Vapor and SSR lanes, with and without a `<script setup>`. The
//! `.html` side is the committed fixture `tests/tooling/
//! davinci-pug-oracle.test.ts` proves is `pug@3.0.4`'s output; refused
//! pug fails the compile with its first diagnostic.

use std::path::PathBuf;

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

const SHELLS: [&str; 2] = [
    "",
    "\n<script setup>\nimport { ref } from \"vue\";\nconst query = ref(\"\");\nconst items = ref([]);\n</script>\n<style scoped>\n.card { color: red }\n</style>\n",
];

fn fixtures(dir: &str) -> Vec<(String, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/davinci-pug")
        .join(dir);
    let mut paths: Vec<_> = std::fs::read_dir(&root)
        .expect("pug fixture directory")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "pug"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            (
                path.file_stem().unwrap().to_string_lossy().into_owned(),
                path,
            )
        })
        .collect()
}

fn lanes() -> [(&'static str, SfcCompileOptions); 3] {
    let mut ssr = SfcCompileOptions::default();
    ssr.template.ssr = true;
    let vapor = SfcCompileOptions {
        vapor: true,
        ..SfcCompileOptions::default()
    };
    [
        ("dom", SfcCompileOptions::default()),
        ("ssr", ssr),
        ("vapor", vapor),
    ]
}

/// Compile `source` in one lane: the code, CSS and diagnostics as text.
fn compile(source: &str, options: SfcCompileOptions) -> Result<String, String> {
    let parse = SfcParseOptions {
        filename: "Fixture.vue".into(),
        ..SfcParseOptions::default()
    };
    let descriptor =
        parse_sfc(source, parse).map_err(|error| String::from(error.message.as_str()))?;
    let result =
        compile_sfc(&descriptor, options).map_err(|error| String::from(error.message.as_str()))?;
    let messages = |list: &[vize_atelier_sfc::SfcError]| {
        list.iter()
            .map(|error| String::from(error.message.as_str()))
            .collect::<Vec<_>>()
    };
    Ok([
        String::from(result.code.as_str()),
        result
            .css
            .map(|css| String::from(css.as_str()))
            .unwrap_or_default(),
        messages(&result.errors).join("\n"),
        messages(&result.warnings).join("\n"),
    ]
    .join("\n/*--*/\n"))
}

#[test]
fn pug_compiles_exactly_like_its_pug_rendered_html_in_every_lane() {
    let matrix = fixtures("matrix");
    assert_eq!(matrix.len(), 26);
    let mut compared = 0;
    for (name, path) in &matrix {
        let pug = std::fs::read_to_string(path).expect("pug");
        let html = std::fs::read_to_string(path.with_extension("html")).expect("html");
        for shell in SHELLS {
            let pug_sfc = ["<template lang=\"pug\">", &pug, "</template>\n", shell].concat();
            let html_sfc = ["<template>", &html, "</template>\n", shell].concat();
            for (lane, options) in lanes() {
                let from_pug = compile(&pug_sfc, options.clone());
                let from_html = compile(&html_sfc, options);
                assert!(from_html.is_ok(), "{name}/{lane}: the HTML side compiles");
                assert_eq!(
                    from_pug, from_html,
                    "{name}/{lane}: pug diverged from its HTML"
                );
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 26 * 2 * 3);
}

#[test]
fn refused_pug_fails_the_compile_with_its_first_diagnostic() {
    let refused = fixtures("refused");
    assert_eq!(refused.len(), 14);
    for (name, path) in &refused {
        let pug = std::fs::read_to_string(path).expect("pug");
        let first =
            std::fs::read_to_string(path.with_extension("diagnostics")).expect("diagnostics");
        let first = first.lines().next().expect("a diagnostic");
        let message = first.split_once(" — ").expect("message").1;
        let sfc = ["<template lang=\"pug\">", &pug, "</template>\n"].concat();
        for (lane, options) in lanes() {
            let error = compile(&sfc, options).expect_err("refused pug must not compile");
            assert_eq!(error, ["pug template: ", message].concat(), "{name}/{lane}");
        }
    }
}
