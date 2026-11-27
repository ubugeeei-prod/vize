//! A component's named type imports bind the export of their own module: an
//! aliased import resolves through the name it imports, and a same-named
//! declaration of another imported module never takes its place. Every case
//! compiles alike alone and in a batch.

use super::super::super::batch_epoch::tests::lock_epoch_state;
use super::referenced::{compile_alone_and_in_batch, write};

/// Compile `component` beside `files` alone and in a batch, and return the
/// one output both give.
fn compile(files: &[(&str, &str)], component: &str) -> std::string::String {
    let _lock = lock_epoch_state();
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    write(&dir, files);
    let path = dir.join("Component.vue");
    write(&dir, &[("Component.vue", component)]);
    let (alone, in_batch) = compile_alone_and_in_batch(&path, component);
    assert_eq!(in_batch, alone, "{component}");
    alone
}

/// The runtime props of a compiled component, one trimmed line each, and
/// its errors: exactly `props` and none.
fn assert_props(output: &str, props: &[&str]) {
    let lines: Vec<&str> = output.lines().map(str::trim).collect();
    let errors = lines.iter().find_map(|line| line.strip_prefix("errors: "));
    assert_eq!(errors, Some("[]"), "{output}");
    let start = lines.iter().position(|line| *line == "props: {").unwrap() + 1;
    let length = lines
        .iter()
        .skip(start)
        .position(|line| *line == "}," || *line == "}")
        .unwrap();
    let found: Vec<&str> = lines.iter().skip(start).take(length).copied().collect();
    assert_eq!(found, props, "{output}");
}

#[test]
fn an_aliased_import_resolves_through_the_name_it_imports() {
    let output = compile(
        &[("types.ts", "export type Public = { value: string }\n")],
        "<script setup lang=\"ts\">\nimport type { Public as Alias } from './types'\ndefineProps<Alias>()\n</script>\n<template><div /></template>\n",
    );
    assert_props(&output, &["value: { type: String, required: true }"]);
}

#[test]
fn an_aliased_import_resolves_through_renaming_and_star_re_exports() {
    let files = [
        (
            "types.ts",
            "export interface Public { value: string; nested?: Inner }\nexport type Inner = { deep: number }\n",
        ),
        (
            "renamed.ts",
            "export type { Public as Exposed } from './types'\n",
        ),
        ("index.ts", "export * from './renamed'\n"),
    ];
    let output = compile(
        &files,
        "<script setup lang=\"ts\">\nimport type { Exposed as Mine } from './index'\ndefineProps<Mine>()\n</script>\n<template><div /></template>\n",
    );
    assert_props(
        &output,
        &[
            "value: { type: String, required: true },",
            "nested: { type: Object, required: false }",
        ],
    );
}

#[test]
fn a_named_import_reads_its_own_module_over_a_same_named_declaration() {
    let files = [
        (
            "list.ts",
            "type Unwrap<T> = T extends (infer U)[] ? U : never;\n\nexport type Options = { label: string };\nexport type Item = Unwrap<string[]>;\n",
        ),
        (
            "detail.ts",
            "export type Item = { id: number; name: string };\n",
        ),
    ];
    for imports in [
        "import type { Options } from \"./list\";\nimport type { Item } from \"./detail\";\n",
        "import type { Item } from \"./detail\";\nimport type { Options } from \"./list\";\n",
    ] {
        let output = compile(
            &files,
            &format!(
                "<script setup lang=\"ts\">\n{imports}\ndefineProps<{{ item: Item; options?: Options }}>();\n</script>\n\n<template><p>{{{{ item.name }}}}</p></template>\n"
            ),
        );
        assert_props(
            &output,
            &[
                "item: { type: Object, required: true },",
                "options: { type: Object, required: false }",
            ],
        );
    }
}

#[test]
fn a_same_named_boolean_of_another_module_does_not_cast_the_prop() {
    let files = [
        (
            "flags.ts",
            "export type Options = { label: string }\nexport type Flag = boolean\n",
        ),
        ("detail.ts", "export type Flag = string\n"),
    ];
    let output = compile(
        &files,
        "<script setup lang=\"ts\">\nimport type { Options } from './flags'\nimport type { Flag } from './detail'\ndefineProps<{ flag: Flag; options?: Options }>()\n</script>\n<template><div /></template>\n",
    );
    assert_props(
        &output,
        &[
            "flag: { type: String, required: true },",
            "options: { type: Object, required: false }",
        ],
    );
}

#[test]
fn a_local_declaration_still_wins_over_a_bound_import() {
    let output = compile(
        &[("types.ts", "export type Public = { value: string }\n")],
        "<script lang=\"ts\">\nexport type Alias = { local: number }\n</script>\n<script setup lang=\"ts\">\nimport type { Public as Alias } from './types'\ndefineProps<Alias>()\n</script>\n<template><div /></template>\n",
    );
    assert_props(&output, &["local: { type: Number, required: true }"]);
}

/// Compile the pinned compiler project `root`, whose `inputs` are copied to
/// their project paths, for the server and compare it with its reference.
fn compile_pinned_project(root: &str, inputs: &[(&str, &str)]) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler")
        .join(root);
    let project = tempfile::tempdir().unwrap();
    for (input, output) in inputs {
        let target = project.path().join(output);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(input), target).unwrap();
    }
    let filename = project.path().join("Layout.vue");
    let source = std::fs::read_to_string(&filename).unwrap();
    let mut options = crate::SfcCompileOptions::default();
    options.parse.filename = filename.to_str().unwrap().into();
    options.template.ssr = true;
    let descriptor = crate::parse_sfc(&source, options.parse.clone()).unwrap();
    let result = crate::compile_sfc(&descriptor, options).unwrap();
    assert_eq!(
        serde_json::to_value(&result.errors).unwrap(),
        serde_json::json!([])
    );
    assert_eq!(
        serde_json::to_value(&result.warnings).unwrap(),
        serde_json::json!([])
    );
    assert_eq!(
        result.code.as_str(),
        std::fs::read_to_string(fixture.join("module.expected.txt")).unwrap(),
    );
}

#[test]
fn compiles_the_pinned_aliased_specifier_project() {
    compile_pinned_project(
        "imported-props-aliased-specifier",
        &[
            ("Layout.vue.txt", "Layout.vue"),
            ("types.ts.txt", "types.ts"),
        ],
    );
}

#[test]
fn compiles_the_pinned_module_scoped_names_project() {
    compile_pinned_project(
        "imported-props-module-scoped-names",
        &[
            ("Layout.vue.txt", "Layout.vue"),
            ("list.ts.txt", "list.ts"),
            ("detail.ts.txt", "detail.ts"),
        ],
    );
}
