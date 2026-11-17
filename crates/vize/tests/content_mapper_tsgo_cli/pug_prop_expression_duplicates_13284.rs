//! #13284: under the content mapper, an error inside a component prop
//! expression in a Pug template was reported twice, once from the per-prop
//! assignment and once from the props literal. The HTML template it lowers
//! to marks the assignment as an echo; the Pug projection now gets the same
//! echo table, so each authored error reports once, as `vize check` does.

use std::{fs, path::Path, path::PathBuf, process::Command};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/pug-prop-expression-duplicates-13284/",
            $path
        ))
    };
}

/// The reporter's project, file for file.
const REPORTED: [(&str, &str); 4] = [
    ("package.json", reported!("package.json.txt")),
    ("tsconfig.json", reported!("tsconfig.json.txt")),
    ("src/child.vue", reported!("src/child.vue.txt")),
    ("src/parent.vue", reported!("src/parent.vue.txt")),
];

/// The issue's expected output.
const EXPECTED: &str = "src/parent.vue(9,19): error TS2339: Property 'inElement' does not exist on type '{ a: number; }'.\n\
src/parent.vue(8,21): error TS2339: Property 'inComponent' does not exist on type '{ a: number; }'.\n";

const PAIR_COMPONENTS: [(&str, &str); 2] = [
    (
        "src/pairs/Child.vue",
        r#"<script setup lang="ts">
defineProps<{ title: string; longTitle?: string; other?: number }>()
defineEmits<{ pick: [id: number] }>()
defineSlots<{ default(props: { item: { id: number } }): any }>()
defineModel<string>("model")
</script>

<template>
  <div><slot :item="{ id: 1 }" /></div>
</template>
"#,
    ),
    (
        "src/pairs/Gen.vue",
        r#"<script setup lang="ts" generic="T extends { id: number }">
defineProps<{ items: T[]; selected?: T }>()
</script>

<template>
  <div />
</template>
"#,
    ),
];

const PAIR_SCRIPT: &str = r#"<script setup lang="ts">
import Child from "../Child.vue"
import Gen from "../Gen.vue"

const model = { a: 1 }
const items = [{ id: 1 }]
const flag = Math.random() > 0.5
const key = "title"
void Child; void Gen; void items; void flag; void key
</script>
"#;

/// `(name, errors, HTML template body, Pug template body)`.
const PAIRS: [(&str, usize, &str, &str); 20] = [
    (
        "colon",
        1,
        r#"<Child :title="model.missing" />"#,
        r#"Child(:title="model.missing")"#,
    ),
    (
        "v-bind",
        1,
        r#"<Child v-bind:title="model.missing" />"#,
        r#"Child(v-bind:title="model.missing")"#,
    ),
    ("same-name", 1, "<Child :title />", "Child(:title)"),
    (
        "object",
        1,
        r#"<Child v-bind="model.missing" />"#,
        r#"Child(v-bind="model.missing")"#,
    ),
    (
        "kebab",
        1,
        r#"<Child title="t" :long-title="model.missing" />"#,
        r#"Child(title="t" :long-title="model.missing")"#,
    ),
    (
        "dynamic",
        1,
        r#"<Child title="t" :[key]="model.missing" />"#,
        r#"Child(title="t" :[key]="model.missing")"#,
    ),
    (
        "v-for",
        1,
        r#"<Child v-for="item in items" :key="item.id" :title="item.missing" />"#,
        r#"Child(v-for="item in items" :key="item.id" :title="item.missing")"#,
    ),
    (
        "v-if",
        1,
        r#"<Child v-if="flag" :title="model.missing" />"#,
        r#"Child(v-if="flag" :title="model.missing")"#,
    ),
    (
        "v-else",
        1,
        "<div v-if=\"flag\" />\n<Child v-else :title=\"model.missing\" />",
        "div(v-if=\"flag\")\nChild(v-else :title=\"model.missing\")",
    ),
    (
        "mismatch",
        1,
        r#"<Child :title="model.a" />"#,
        r#"Child(:title="model.a")"#,
    ),
    (
        "listener",
        1,
        r#"<Child title="t" @pick="model.missing" />"#,
        r#"Child(title="t" @pick="model.missing")"#,
    ),
    (
        "v-model",
        1,
        r#"<Child title="t" v-model:model="model.missing" />"#,
        r#"Child(title="t" v-model:model="model.missing")"#,
    ),
    (
        "several",
        2,
        r#"<Child :title="model.x" :other="model.y" />"#,
        r#"Child(:title="model.x" :other="model.y")"#,
    ),
    (
        "nested",
        1,
        "<Child title=\"t\">\n  <Child :title=\"model.inner\" />\n</Child>",
        "Child(title=\"t\")\n  Child(:title=\"model.inner\")",
    ),
    (
        "slot-props",
        1,
        "<Child title=\"t\" v-slot=\"{ item }\">\n  <Child :title=\"item.missing\" />\n</Child>",
        "Child(title=\"t\" v-slot=\"{ item }\")\n  Child(:title=\"item.missing\")",
    ),
    (
        "slot-outlet",
        1,
        r#"<slot :item="model.missing" />"#,
        r#"slot(:item="model.missing")"#,
    ),
    (
        "generic",
        1,
        r#"<Gen :items="model.missing" />"#,
        r#"Gen(:items="model.missing")"#,
    ),
    (
        "generic-inferred",
        1,
        r#"<Gen :items="[{ id: 1 }]" :selected="model.missing" />"#,
        r#"Gen(:items="[{ id: 1 }]" :selected="model.missing")"#,
    ),
    (
        "nested-expression",
        1,
        r#"<Child :title="flag ? String(model.missing) : 'x'" />"#,
        r#"Child(:title="flag ? String(model.missing) : 'x'")"#,
    ),
    (
        "class-style",
        2,
        r#"<Child title="t" :class="model.missing" :style="model.other" />"#,
        r#"Child(title="t" :class="model.missing" :style="model.other")"#,
    ),
];

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// `(file, line, column, code, message)` for every tsgo diagnostic.
fn mapper_rows(
    tsgo: &Path,
    root: &Path,
) -> Vec<(CompactString, usize, usize, CompactString, CompactString)> {
    let output = check_project(tsgo, root, "tsconfig.json");
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let parsed = (|| {
                let (file, rest) = line.split_once('(')?;
                let (position, rest) = rest.split_once("): error ")?;
                let (line, column) = position.split_once(',')?;
                let (code, message) = rest.split_once(": ")?;
                Some((
                    file.into(),
                    line.parse().ok()?,
                    column.parse().ok()?,
                    code.into(),
                    message.into(),
                ))
            })();
            parsed.unwrap_or_else(|| panic!("unexpected tsgo line: {line}"))
        })
        .collect()
}

/// `file(line,col) TScode` rows from `vize check`.
fn cli_lines(tsgo: &Path, root: &Path) -> Vec<CompactString> {
    let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", tsgo)
        .args([
            "check",
            "--quiet",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
            "src",
        ])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&cli.stdout)
        .unwrap_or_else(|error| panic!("{error}\n{}", output_text(&cli)));
    let mut observed = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(move |diagnostic| {
                    let text = diagnostic.as_str()?.strip_prefix("error:")?;
                    let (position, rest) = text.split_once(" [")?;
                    let (code, _) = rest.split_once(']')?;
                    let (line, column) = position.split_once(':')?;
                    Some(cstr!("{name}({line},{column}) {code}"))
                })
        })
        .collect::<Vec<_>>();
    observed.sort();
    observed
}

#[test]
fn pug_component_prop_expressions_report_each_error_once() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("pug-prop-expression-duplicates-13284-")
        .tempdir_in(cases_root)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    for (path, contents) in REPORTED {
        write(root, path, contents);
    }

    let output = check_project(&tsgo, root, "tsconfig.json");
    assert_eq!(
        output.stdout,
        EXPECTED.as_bytes(),
        "{}",
        output_text(&output)
    );
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        cli_lines(&tsgo, root),
        ["src/parent.vue(8,21) TS2339", "src/parent.vue(9,19) TS2339"]
    );

    // Every HTML/Pug pair reports the same diagnostics, once per error.
    for (path, contents) in PAIR_COMPONENTS {
        write(root, path, contents);
    }
    for (name, _, html, pug) in PAIRS {
        write(
            root,
            &cstr!("src/pairs/html/{name}.vue"),
            &cstr!("{PAIR_SCRIPT}\n<template>\n{html}\n</template>\n"),
        );
        write(
            root,
            &cstr!("src/pairs/pug/{name}.vue"),
            &cstr!("{PAIR_SCRIPT}\n<template lang=\"pug\">\n{pug}\n</template>\n"),
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

    // A Pug member error still points at the authored member name.
    for (file, line, column, code, message) in &rows {
        if !file.starts_with("src/pairs/pug/") || code != "TS2339" {
            continue;
        }
        let member = message.split('\'').nth(1).unwrap();
        let source = fs::read_to_string(root.join(file.as_str())).unwrap();
        let authored = source.lines().nth(line - 1).unwrap();
        assert_eq!(
            authored.get(column - 1..column - 1 + member.len()),
            Some(member),
            "{file}({line},{column}): {message}"
        );
    }

    // `vize check` agrees with the mapper on every row.
    let mut mapper = rows
        .iter()
        .map(|(file, line, column, code, _)| cstr!("{file}({line},{column}) {code}"))
        .collect::<Vec<_>>();
    mapper.sort();
    assert_eq!(cli_lines(&tsgo, root), mapper);
}
