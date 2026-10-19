//! Monikers for definition targets, in the lsif-tsc spelling.
//!
//! An exported symbol is `tsc` / `<package>:<module path>:<exported name>`,
//! unique within its package group. It is an `export` of a workspace package
//! and an `import` when it lives under `node_modules`. Anything else is a
//! `local` moniker, unique to its document.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "tower-lsp Moniker identifiers are std String"
)]

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, ModuleExportName, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::{Moniker, MonikerKind, Range, UniquenessLevel};

use crate::ide::position_to_offset;

const SCHEME: &str = "tsc";

/// The moniker of the symbol a definition range names in `path`.
pub(crate) fn moniker_for_definition(path: &Path, text: &str, range: Range) -> Option<Moniker> {
    let is_vue = path.extension().is_some_and(|extension| extension == "vue");
    let start = position_to_offset(text, range.start.line, range.start.character)?;
    let end = position_to_offset(text, range.end.line, range.end.character)?;
    let name = text.get(start..end).unwrap_or_default();
    let (name, exported) = if is_vue && (name.is_empty() || name.starts_with('<')) {
        // A component definition lands on the SFC itself: its default export.
        ("default".to_owned(), Some("default".to_owned()))
    } else if is_identifier(name) {
        let exported = exported_name(module_source(path, text)?, source_type(path), name);
        (name.to_owned(), exported)
    } else {
        return None;
    };
    let package = nearest_package(path);
    match (exported, package) {
        (Some(exported), Some((package_name, package_root))) => {
            let module = module_path(path, &package_root)?;
            let in_dependency = path
                .components()
                .any(|component| component.as_os_str() == "node_modules");
            Some(Moniker {
                scheme: SCHEME.to_owned(),
                identifier: format!("{package_name}:{module}:{exported}"),
                unique: UniquenessLevel::Group,
                kind: Some(if in_dependency {
                    MonikerKind::Import
                } else {
                    MonikerKind::Export
                }),
            })
        }
        _ => {
            let file = path.file_name()?.to_string_lossy();
            Some(Moniker {
                scheme: SCHEME.to_owned(),
                identifier: format!(
                    "{file}:{name}:{}:{}",
                    range.start.line, range.start.character
                ),
                unique: UniquenessLevel::Document,
                kind: Some(MonikerKind::Local),
            })
        }
    }
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first == '$' || first.is_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_alphanumeric())
}

/// Module text to scan for exports: the whole file, or an SFC's plain
/// `<script>` (a `<script setup>` exports nothing but types it hoists).
fn module_source<'a>(path: &Path, text: &'a str) -> Option<&'a str> {
    if path.extension().is_none_or(|extension| extension != "vue") {
        return Some(text);
    }
    let mut rest = text;
    while let Some((_, after)) = rest.split_once("<script") {
        let (tag, body) = after.split_once('>')?;
        let (content, tail) = body.split_once("</script>")?;
        if !tag.contains("setup") {
            return Some(content);
        }
        rest = tail;
    }
    Some("")
}

fn source_type(path: &Path) -> SourceType {
    let name = path.to_string_lossy();
    if name.ends_with(".tsx") {
        SourceType::tsx()
    } else if name.ends_with(".jsx") {
        SourceType::jsx()
    } else if name.ends_with(".js") || name.ends_with(".mjs") || name.ends_with(".cjs") {
        SourceType::mjs()
    } else {
        SourceType::ts()
    }
}

/// The public name `local` is exported under, if any.
fn exported_name(source: &str, source_type: SourceType, local: &str) -> Option<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return None;
    }
    let module_name = |name: &ModuleExportName<'_>| name.name().to_string();
    for statement in &parsed.program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration
                    && declared_names(declaration).iter().any(|name| name == local)
                {
                    return Some(local.to_owned());
                }
                if export.source.is_none() {
                    for specifier in &export.specifiers {
                        if module_name(&specifier.local) == local {
                            return Some(module_name(&specifier.exported));
                        }
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                let id = match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                        function.id.as_ref().map(|id| id.name.as_str())
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        class.id.as_ref().map(|id| id.name.as_str())
                    }
                    _ => None,
                };
                if id == Some(local) {
                    return Some("default".to_owned());
                }
            }
            _ => {}
        }
    }
    None
}

fn declared_names(declaration: &Declaration<'_>) -> Vec<String> {
    match declaration {
        Declaration::VariableDeclaration(variables) => variables
            .declarations
            .iter()
            .filter_map(|declarator| match &declarator.id {
                BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
                _ => None,
            })
            .collect(),
        Declaration::FunctionDeclaration(function) => {
            function.id.iter().map(|id| id.name.to_string()).collect()
        }
        Declaration::ClassDeclaration(class) => {
            class.id.iter().map(|id| id.name.to_string()).collect()
        }
        Declaration::TSTypeAliasDeclaration(alias) => vec![alias.id.name.to_string()],
        Declaration::TSInterfaceDeclaration(interface) => vec![interface.id.name.to_string()],
        Declaration::TSEnumDeclaration(declaration) => vec![declaration.id.name.to_string()],
        _ => Vec::new(),
    }
}

/// The nearest `package.json` with a name, and its directory.
fn nearest_package(path: &Path) -> Option<(String, std::path::PathBuf)> {
    path.ancestors().skip(1).find_map(|dir| {
        let manifest = std::fs::read_to_string(dir.join("package.json")).ok()?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;
        let name = manifest.get("name")?.as_str()?.to_owned();
        Some((name, dir.to_path_buf()))
    })
}

/// The module path inside its package, `/`-separated, without a TypeScript
/// or JavaScript extension (`.vue` stays part of the specifier).
fn module_path(path: &Path, package_root: &Path) -> Option<String> {
    let relative = path.strip_prefix(package_root).ok()?;
    let relative = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    for extension in [
        ".d.ts", ".d.mts", ".d.cts", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
    ] {
        if let Some(stripped) = relative.strip_suffix(extension) {
            return Some(stripped.to_owned());
        }
    }
    Some(relative)
}

#[cfg(test)]
mod tests {
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
            moniker_for_definition(&path, text, range_of(text, "local"))
                .map(|moniker| moniker.unique),
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
}
