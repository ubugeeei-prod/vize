//! Authenticate `import Component …; export default Component` barrel edges.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ExportDefaultDeclarationKind, ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::String;

pub(super) fn import_target(content: &str, path: &Path) -> Option<(String, String)> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, content, SourceType::from_path(path).ok()?).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let local = parsed.program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        let ExportDefaultDeclarationKind::Identifier(identifier) = &export.declaration else {
            return None;
        };
        Some(identifier.name.as_str())
    })?;
    parsed.program.body.iter().find_map(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return None;
        };
        if import.import_kind.is_type() {
            return None;
        }
        let specifier = import
            .specifiers
            .as_ref()?
            .iter()
            .find(|specifier| specifier.local().name == local)?;
        let exported = match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => "default",
            ImportDeclarationSpecifier::ImportSpecifier(named) if !named.import_kind.is_type() => {
                named.imported.name().as_str()
            }
            _ => return None,
        };
        Some((import.source.value.as_str().into(), exported.into()))
    })
}

#[cfg(test)]
mod tests {
    use super::import_target;
    use std::path::Path;

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
}
