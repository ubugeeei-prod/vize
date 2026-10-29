#![expect(
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "tests assert by panicking; fixtures use std strings"
)]
//! The P4-12c compile oracle: compiling an SFC with `<template
//! lang="pug">` equals, byte for byte, compiling the same SFC with its
//! template replaced by the pinned `pug` package's HTML rendering — in the
//! DOM, Vapor and SSR lanes, with and without a `<script setup>`. The
//! `.html` side is the committed fixture `tests/tooling/
//! davinci-pug-oracle.test.ts` proves is `pug@3.0.4`'s output; refused
//! pug fails the compile with its first diagnostic.

use std::path::PathBuf;

use oxc_sourcemap::SourceMap;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, compile_sfc,
    compile_sfc_with_template_syntax_and_codegen_options, parse_sfc,
};

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
    assert_eq!(matrix.len(), 33);
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
    assert_eq!(compared, 33 * 2 * 3);
}

#[test]
fn jade_alias_compiles_exactly_like_pug_in_every_lane() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/davinci-pug/matrix/slots-and-components.pug");
    let template = std::fs::read_to_string(path).expect("pug");
    for shell in SHELLS {
        let pug = ["<template lang=\"pug\">", &template, "</template>\n", shell].concat();
        for language in ["jade", "JADE"] {
            let jade = [
                "<template lang=\"",
                language,
                "\">",
                &template,
                "</template>\n",
                shell,
            ]
            .concat();
            for (lane, options) in lanes() {
                assert_eq!(
                    compile(&jade, options.clone()),
                    compile(&pug, options),
                    "{language} alias diverged from pug in {lane}"
                );
            }
        }
    }
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
        // The compile reads includes (the lowering alone refuses them), so
        // a missing file fails as pug fails.
        let message = match name.as_str() {
            "include" => "pug cannot include `partials/header.pug`: the file does not exist",
            _ => first.split_once(" — ").expect("message").1,
        };
        let sfc = ["<template lang=\"pug\">", &pug, "</template>\n"].concat();
        for (lane, options) in lanes() {
            let error = compile(&sfc, options).expect_err("refused pug must not compile");
            assert_eq!(error, ["pug template: ", message].concat(), "{name}/{lane}");
        }
    }
}

#[test]
fn refused_jade_fails_like_refused_pug() {
    let source = "include missing.pug";
    for (lane, options) in lanes() {
        let pug = ["<template lang=\"pug\">", source, "</template>"].concat();
        let jade = ["<template lang=\"jade\">", source, "</template>"].concat();
        assert_eq!(
            compile(&jade, options.clone()),
            compile(&pug, options),
            "{lane}"
        );
    }
}

fn offset(text: &str, line: u32, column: u32) -> Option<usize> {
    let start = if line == 0 {
        0
    } else {
        text.match_indices('\n').nth(line as usize - 1)?.0 + 1
    };
    let mut units = 0;
    for (at, ch) in text.get(start..)?.char_indices() {
        if units == column {
            return Some(start + at);
        }
        if ch == '\n' {
            return None;
        }
        units += ch.len_utf16() as u32;
        if units > column {
            return None;
        }
    }
    (units == column).then_some(text.len())
}

#[test]
fn template_only_map_navigates_generated_tokens_to_authored_pug() {
    let source = concat!(
        "<template lang=\"pug\">\n",
        "main\n",
        "  h1 Café 😀\n",
        "  p {{ account.name }}\n",
        "</template>\n",
    );
    let filename = "/src/PugMap.vue";
    let parse = SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("pug fixture parses");
    let compile = |source_map| {
        compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            SfcCompileOptions {
                parse: parse.clone(),
                ..Default::default()
            },
            TemplateSyntaxMode::Standard,
            CodegenOptions {
                source_map,
                filename: filename.into(),
                ..Default::default()
            },
        )
        .expect("pug fixture compiles")
    };

    let without = compile(false);
    let with = compile(true);
    assert_eq!(with.code, without.code, "source maps must be additive");
    assert!(without.map.is_none());
    let map = with.map.expect("mapped template-only module");
    assert_eq!(map["sources"], serde_json::json!([filename]));
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));

    let raw = serde_json::to_string(&map).expect("map JSON");
    let map = SourceMap::from_json_string(&raw).expect("valid source map");
    let generated = with
        .code
        .find("_ctx.account.name")
        .expect("generated expression");
    let authored = source.find("account.name").expect("authored expression");
    let token = map
        .get_tokens()
        .find(|token| {
            offset(&with.code, token.get_dst_line(), token.get_dst_col()) == Some(generated)
        })
        .expect("generated expression has a navigation anchor");
    assert_eq!(
        offset(source, token.get_src_line(), token.get_src_col()),
        Some(authored),
        "the runtime token must navigate to the Pug expression, not derived HTML",
    );
}

#[test]
fn plain_template_only_map_request_preserves_unmapped_default_packet() {
    let source = "<template><slot v-pre></slot></template>\n";
    let filename = "/src/PlainMapBoundary.vue";
    let parse = SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("plain fixture parses");
    let compile = |source_map| {
        compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            SfcCompileOptions {
                parse: parse.clone(),
                ..Default::default()
            },
            TemplateSyntaxMode::Standard,
            CodegenOptions {
                source_map,
                filename: filename.into(),
                ..Default::default()
            },
        )
        .expect("plain fixture compiles")
    };
    let plain = serde_json::to_value(compile(false)).expect("plain whole packet");
    let requested = serde_json::to_value(compile(true)).expect("requested whole packet");
    assert_eq!(
        serde_json::json!({ "plainMap": &plain["map"], "requested": requested }),
        serde_json::json!({ "plainMap": null, "requested": plain }),
        "a source-map request must not change an unmapped plain template packet"
    );
}

#[test]
fn template_error_location_points_at_authored_pug() {
    let source = "<template lang=\"pug\">\ndiv\n  p(title=\"😀\" v-else) orphan\n</template>\n";
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "/src/PugError.vue".into(),
            ..Default::default()
        },
    )
    .expect("pug fixture parses");
    let error = compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect_err("an orphan v-else is rejected");
    let loc = error.loc.expect("exact diagnostic location");
    assert_eq!(source.get(loc.start..loc.end), Some("v-else"));
    assert_eq!((loc.start_line, loc.start_column), (3, 18));
    assert_eq!((loc.end_line, loc.end_column), (3, 24));
}

#[test]
fn pug_includes_resolve_against_the_sfc_and_compile_like_their_html() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/davinci-pug/includes")
        .canonicalize()
        .expect("include fixtures");
    let pug = std::fs::read_to_string(root.join("layout.pug")).expect("pug");
    let html = std::fs::read_to_string(root.join("layout.html")).expect("html");
    let filename = root.join("Layout.vue").to_string_lossy().into_owned();
    let parse = SfcParseOptions {
        filename: filename.as_str().into(),
        ..SfcParseOptions::default()
    };
    let pug_sfc = ["<template lang=\"pug\">", &pug, "</template>\n"].concat();
    let html_sfc = ["<template>", &html, "</template>\n"].concat();
    for (lane, options) in lanes() {
        let from_pug = parse_sfc(&pug_sfc, parse.clone()).unwrap();
        let from_html = parse_sfc(&html_sfc, parse.clone()).unwrap();
        let from_pug = compile_sfc(&from_pug, options.clone()).unwrap();
        let from_html = compile_sfc(&from_html, options).unwrap();
        assert_eq!(from_pug.code, from_html.code, "{lane}");
        let read: Vec<_> = from_pug
            .dependencies
            .iter()
            .map(|file| {
                std::path::Path::new(file.as_str())
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(
            read,
            [
                "partials/header.pug",
                "partials/nested/logo.pug",
                "partials/card.pug",
                "partials/list.pug",
                "partials/note.txt",
            ],
            "{lane}: includes are dependencies"
        );
        assert!(from_html.dependencies.is_empty());
    }
}
