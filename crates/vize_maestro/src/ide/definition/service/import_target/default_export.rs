//! Authenticate default-export declarations and imported component barrel edges.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier, Statement,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};
use vize_l0::String;

pub(super) enum Target {
    Import { specifier: String, exported: String },
    Declaration(Span),
}

pub(super) fn target(content: &str, path: &Path) -> Option<Target> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, content, SourceType::from_path(path).ok()?).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let export = parsed.program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        Some(export)
    })?;
    match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            return Some(Target::Declaration(
                function.id.as_ref().map_or(function.span, |id| id.span),
            ));
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            return Some(Target::Declaration(
                class.id.as_ref().map_or(class.span, |id| id.span),
            ));
        }
        _ => {}
    }
    let Expression::Identifier(identifier) =
        export.declaration.as_expression()?.get_inner_expression()
    else {
        return None;
    };
    let local = identifier.name.as_str();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let Some(specifier) = import.specifiers.as_ref().and_then(|specifiers| {
            specifiers
                .iter()
                .find(|specifier| specifier.local().name == local)
        }) else {
            continue;
        };
        if import.import_kind.is_type() {
            return None;
        }
        let exported = match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => "default",
            ImportDeclarationSpecifier::ImportSpecifier(named) if !named.import_kind.is_type() => {
                named.imported.name().as_str()
            }
            _ => return None,
        };
        return Some(Target::Import {
            specifier: import.source.value.as_str().into(),
            exported: exported.into(),
        });
    }
    // Resolve an exported identifier to its lexical declaration, never to the
    // first same-spelling token (which may be a comment or a nested shadow).
    let semantic = SemanticBuilder::new().build(&parsed.program);
    if !semantic.diagnostics.is_empty() {
        return None;
    }
    let symbol = semantic
        .semantic
        .scoping()
        .get_reference(identifier.reference_id.get()?)
        .symbol_id()?;
    Some(Target::Declaration(
        semantic.semantic.scoping().symbol_span(symbol),
    ))
}

#[cfg(test)]
fn import_target(content: &str, path: &Path) -> Option<(String, String)> {
    match target(content, path)? {
        Target::Import {
            specifier,
            exported,
        } => Some((specifier, exported)),
        Target::Declaration(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::import_target;
    use crate::ide::{IdeContext, definition::DefinitionService};
    use crate::server::ServerState;
    use std::fs;
    use std::path::Path;
    use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

    #[test]
    fn original_n8n_barrel_follows_the_default_component_identity() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/n8n-authored-editor/index.ts.txt"
        ));
        assert_eq!(
            import_target(source, Path::new("index.ts")),
            Some(("./BlockUi.vue".into(), "default".into()))
        );
    }

    #[test]
    fn only_a_real_value_import_export_edge_is_followed() {
        for (source, expected) in [
            (
                "// import Fake from './fake'; export default Fake;\nimport Real from './real'; const decoy = 'export default Fake'; export default Real;",
                Some(("./real", "default")),
            ),
            (
                "import { Widget as Real } from './real'; export default Real;",
                Some(("./real", "Widget")),
            ),
            ("import type Real from './real'; export default Real;", None),
            ("import Real from './real'; export default Unknown;", None),
            (
                "import Real from './real'; export default wrap(Real);",
                None,
            ),
            ("import Real from './real'; export default", None),
        ] {
            assert_eq!(
                import_target(source, Path::new("index.ts")),
                expected.map(|(path, export)| (path.into(), export.into())),
                "{source}"
            );
        }
    }
    #[test]
    fn default_imports_preserve_local_function_variable_and_class_definitions() {
        for (target_source, anchor) in [
            ("export default function Widget() {}", "function Widget"),
            (
                "const Widget = () => null; export default Widget;",
                "const Widget",
            ),
            (
                "const Widget = () => null; export default (Widget as unknown);",
                "const Widget",
            ),
            ("export default class Widget {}", "class Widget"),
            (
                "// const Widget = decoy;\nfunction inner() { const Widget = 0; }\nconst Widget = () => null; export default Widget;",
                "const Widget = ()",
            ),
        ] {
            for local in ["Widget", "Renamed"] {
                let project = tempfile::tempdir().unwrap();
                let target = project.path().join("index.ts");
                fs::write(&target, target_source).unwrap();
                let source: std::string::String = vize_l0::cstr!(
                    "<script setup lang=\"ts\">\nimport {local} from './index';\n</script>\n<template><{local} /></template>"
                ).into();
                let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
                let state = ServerState::new();
                state
                    .documents
                    .open(uri.clone(), source.clone(), 1, "vue".into());
                state.update_virtual_docs(&uri, &source);
                let ctx = IdeContext::new(&state, &uri, source.rfind(local).unwrap()).unwrap();
                let GotoDefinitionResponse::Scalar(location) =
                    DefinitionService::definition(&ctx).expect("default definition")
                else {
                    panic!("default definition must be scalar");
                };
                let expected = target_source.find(anchor).unwrap() + anchor.find("Widget").unwrap();
                let (line, character) = crate::ide::offset_to_position(target_source, expected);
                assert_eq!(
                    location.uri,
                    Url::from_file_path(&target).unwrap(),
                    "{target_source}"
                );
                assert_eq!(
                    location.range,
                    tower_lsp::lsp_types::Range::new(
                        tower_lsp::lsp_types::Position::new(line, character),
                        tower_lsp::lsp_types::Position::new(line, character + 6)
                    ),
                    "{target_source}"
                );
            }
        }
    }
}
