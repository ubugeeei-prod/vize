#![expect(
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "tests build fixtures with format! and slice their own ASCII-indexed text"
)]

use super::moniker_for_definition;
use tower_lsp::lsp_types::{Moniker, MonikerKind, Position, Range, UniquenessLevel};

fn range_of(text: &str, needle: &str) -> Range {
    let start = text.find(needle).unwrap();
    let position = |offset: usize| {
        let before = &text[..offset];
        let line = before.matches('\n').count() as u32;
        let column = before.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
        Position::new(line, column)
    };
    Range::new(position(start), position(start + needle.len()))
}

fn package(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().to_path_buf();
    std::fs::write(path.join("package.json"), format!(r#"{{"name":"{name}"}}"#)).unwrap();
    (root, path)
}

fn moniker(identifier: &str, unique: UniquenessLevel, kind: MonikerKind) -> Moniker {
    Moniker {
        scheme: "tsc".to_owned(),
        identifier: identifier.to_owned(),
        unique,
        kind: Some(kind),
    }
}

#[test]
fn exported_declarations_and_aliases_name_their_public_identity() {
    let (_root, root) = package("app");
    let path = root.join("src/lib.ts");
    let text = "export const count = 1;\nconst inner = 2;\nexport { inner as renamed };\nexport default function make() {}\nexport interface Shape { x: number }\n";
    for (needle, identifier) in [
        ("count", "app:src/lib:count"),
        ("inner", "app:src/lib:renamed"),
        ("make", "app:src/lib:default"),
        ("Shape", "app:src/lib:Shape"),
    ] {
        assert_eq!(
            moniker_for_definition(&path, text, range_of(text, needle)),
            Some(moniker(
                identifier,
                UniquenessLevel::Group,
                MonikerKind::Export
            )),
            "{needle}"
        );
    }
}

#[test]
fn unexported_bindings_are_document_local() {
    let (_root, root) = package("app");
    let path = root.join("src/lib.ts");
    let text = "// 日本語 😀\nconst hidden = 1;\nexport const shown = hidden;\n";
    assert_eq!(
        moniker_for_definition(&path, text, range_of(text, "hidden")),
        Some(moniker(
            "lib.ts:hidden:1:6",
            UniquenessLevel::Document,
            MonikerKind::Local
        ))
    );
}

#[test]
fn component_files_export_their_default_and_plain_script_exports() {
    let (_root, root) = package("@scope/ui");
    let path = root.join("src/Button.vue");
    let text = "<script lang=\"ts\">\nexport const size = 1;\n</script>\n<script setup lang=\"ts\">\nconst local = 2;\n</script>\n<template><b /></template>\n";
    assert_eq!(
        moniker_for_definition(&path, text, Range::default()),
        Some(moniker(
            "@scope/ui:src/Button.vue:default",
            UniquenessLevel::Group,
            MonikerKind::Export
        ))
    );
    assert_eq!(
        moniker_for_definition(&path, text, range_of(text, "size")),
        Some(moniker(
            "@scope/ui:src/Button.vue:size",
            UniquenessLevel::Group,
            MonikerKind::Export
        ))
    );
    assert_eq!(
        moniker_for_definition(&path, text, range_of(text, "local")).map(|moniker| moniker.unique),
        Some(UniquenessLevel::Document)
    );
}

#[test]
fn dependency_declarations_are_imports_named_by_their_own_package() {
    let (_root, root) = package("app");
    let dependency = root.join("node_modules/lib-pkg");
    std::fs::create_dir_all(dependency.join("dist")).unwrap();
    std::fs::write(dependency.join("package.json"), r#"{"name":"lib-pkg"}"#).unwrap();
    let path = dependency.join("dist/index.d.ts");
    let text = "export declare function format(value: string): string;\n";
    assert_eq!(
        moniker_for_definition(&path, text, range_of(text, "format")),
        Some(moniker(
            "lib-pkg:dist/index:format",
            UniquenessLevel::Group,
            MonikerKind::Import
        ))
    );
}

#[test]
fn ranges_that_are_not_identifiers_have_no_moniker() {
    let (_root, root) = package("app");
    let path = root.join("src/lib.ts");
    let text = "export const count = 1;\n";
    assert_eq!(
        moniker_for_definition(&path, text, range_of(text, "= 1")),
        None
    );
}
