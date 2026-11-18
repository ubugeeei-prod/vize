//! #13295: a Pug attribute value with `<`, `>`, `&` or `"` lost its exact
//! source mapping, because the lowering entity-escapes those bytes. The
//! content mapper dropped or misplaced errors after an escape and
//! `vize check` reported each one at several wrong columns. Every pair
//! below now reports what its HTML twin reports, once, at the authored
//! Pug column, under the mapper and `vize check` alike.

use std::path::PathBuf;

use super::pug_prop_expression_duplicates_13284::{PAIR_COMPONENTS, cli_lines, mapper_rows, write};
use super::{TSGO_ENV, cstr, install_mapper_manifest, install_vue_package, workspace_root};

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.vue", "src/**/*.ts"],
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }]
}
"#;

const SCRIPT: &str = r#"<script setup lang="ts">
import Child from "../Child.vue"

const model = { a: 1, b: 2 }
const flag = Math.random() > 0.5
void Child; void flag
</script>
"#;

/// `(name, errors, HTML template body, Pug template body)`.
const PAIRS: [(&str, usize, &str, &str); 16] = [
    (
        "issue",
        2,
        "<Child :title=\"model.a < 1 ? model.missing : ''\" />\n<div @click=\"() => model.other()\" />",
        "Child(:title=\"model.a < 1 ? model.missing : ''\")\ndiv(@click=\"() => model.other()\")",
    ),
    (
        "greater",
        1,
        "<div :title=\"model.a > 1 ? model.missing : ''\" />",
        "div(:title=\"model.a > 1 ? model.missing : ''\")",
    ),
    (
        "logical",
        1,
        "<div v-if=\"flag && model.missing\" />",
        "div(v-if=\"flag && model.missing\")",
    ),
    (
        "listener",
        1,
        "<Child title=\"t\" @pick=\"(id) => model.missing(id)\" />",
        "Child(title=\"t\" @pick=\"(id) => model.missing(id)\")",
    ),
    (
        "handler-block",
        1,
        "<div @click=\"() => { if (model.a > 1) model.missing() }\" />",
        "div(@click=\"() => { if (model.a > 1) model.missing() }\")",
    ),
    (
        "double-quotes",
        1,
        "<div :title='model.a ? \"x\" + model.missing : \"\"' />",
        "div(:title='model.a ? \"x\" + model.missing : \"\"')",
    ),
    (
        "single-quotes",
        1,
        "<div :title=\"model.a ? 'x' + model.missing : ''\" />",
        "div(:title=\"model.a ? 'x' + model.missing : ''\")",
    ),
    (
        "two-errors",
        2,
        "<div :title=\"model.a < 1 && model.x ? model.y : ''\" />",
        "div(:title=\"model.a < 1 && model.x ? model.y : ''\")",
    ),
    (
        "next-attribute",
        2,
        "<div :title=\"model.a < model.b\" :class=\"model.c\" />",
        "div(:title=\"model.a < model.b\" :class=\"model.c\")",
    ),
    (
        "shorthands",
        1,
        "<div id=\"i\" class=\"c\" :title=\"flag && model.missing\" />",
        "div#i.c(:title=\"flag && model.missing\")",
    ),
    (
        "unescaped",
        1,
        "<div :title=\"flag && model.missing\" />",
        "div(:title!=\"flag && model.missing\")",
    ),
    (
        "v-for",
        1,
        "<div v-for=\"i in [1, 2]\" :key=\"i\" :title=\"i > 1 && model.missing\" />",
        "div(v-for=\"i in [1, 2]\" :key=\"i\" :title=\"i > 1 && model.missing\")",
    ),
    (
        "multi-line",
        2,
        "<div\n  :title=\"model.a < 1 ? model.missing : ''\"\n  :class=\"flag && model.other\"\n/>",
        "div(\n  :title=\"model.a < 1 ? model.missing : ''\"\n  :class=\"flag && model.other\"\n)",
    ),
    (
        "interpolation",
        1,
        "<p :title=\"'v'\">v {{ model.a < 1 && model.missing }}</p>",
        "p(:title=\"'v'\") #{'v'} {{ model.a < 1 && model.missing }}",
    ),
    (
        "reference-start",
        1,
        "<div :title=\"model.a &&model.missing\" />",
        "div(:title=\"model.a &&model.missing\")",
    ),
    (
        "component-comparison",
        1,
        "<Child :title=\"model.a >= 1 ? 'x' : model.missing\" />",
        "Child(:title=\"model.a >= 1 ? 'x' : model.missing\")",
    ),
];

#[test]
fn pug_escaped_attribute_values_report_at_authored_columns() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("pug-escaped-attribute-values-13295-")
        .tempdir_in(cases_root)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    write(root, "tsconfig.json", TSCONFIG);
    for (path, contents) in PAIR_COMPONENTS {
        write(root, path, contents);
    }
    for (name, _, html, pug) in PAIRS {
        write(
            root,
            &cstr!("src/pairs/html/{name}.vue"),
            &cstr!("{SCRIPT}\n<template>\n{html}\n</template>\n"),
        );
        write(
            root,
            &cstr!("src/pairs/pug/{name}.vue"),
            &cstr!("{SCRIPT}\n<template lang=\"pug\">\n{pug}\n</template>\n"),
        );
    }
    let rows = mapper_rows(&tsgo, root);
    for (name, errors, _, _) in PAIRS {
        let messages = |lane: &str| {
            let file = cstr!("src/pairs/{lane}/{name}.vue");
            let mut messages = rows
                .iter()
                .filter(|row| row.0 == file.as_str())
                .map(|row| cstr!("{} {}", row.3, row.4))
                .collect::<Vec<_>>();
            messages.sort();
            messages
        };
        let html = messages("html");
        assert_eq!(html.len(), errors, "{name}: {html:?}");
        assert_eq!(messages("pug"), html, "{name}");
    }

    // Every Pug member error points at the authored member name.
    let mut members = 0;
    for (file, line, column, code, message) in &rows {
        if !file.starts_with("src/pairs/pug/") || code != "TS2339" {
            continue;
        }
        let member = message.split('\'').nth(1).unwrap();
        let source = std::fs::read_to_string(root.join(file.as_str())).unwrap();
        let authored = source.lines().nth(line - 1).unwrap();
        assert_eq!(
            authored.get(column - 1..column - 1 + member.len()),
            Some(member),
            "{file}({line},{column}): {message}"
        );
        members += 1;
    }
    assert_eq!(members, 19, "{rows:?}");

    // `vize check` agrees with the mapper on every row.
    let mut mapper = rows
        .iter()
        .map(|(file, line, column, code, _)| cstr!("{file}({line},{column}) {code}"))
        .collect::<Vec<_>>();
    mapper.sort();
    assert_eq!(cli_lines(&tsgo, root), mapper);
}
