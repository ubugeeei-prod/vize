//! Exact pre-candidate value extractor; the type extractor remains unchanged.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet};

fn value_names(import_text: &str) -> Vec<&str> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, import_text, SourceType::ts()).parse();
    let Some(Statement::ImportDeclaration(declaration)) = parsed.program.body.first() else {
        return Vec::new();
    };
    if declaration.import_kind == ImportOrExportKind::Type {
        return Vec::new();
    }
    declaration
        .specifiers
        .iter()
        .flatten()
        .filter_map(|specifier| {
            let span = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if specifier.import_kind == ImportOrExportKind::Value =>
                {
                    specifier.local.span
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    specifier.local.span
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    specifier.local.span
                }
                ImportDeclarationSpecifier::ImportSpecifier(_) => return None,
            };
            import_text.get(span.start as usize..span.end as usize)
        })
        .collect()
}

pub(super) fn assert_facts(source: &str, actual: (Vec<&str>, FxHashSet<CompactString>)) {
    assert_eq!(actual.0, value_names(source), "{source}");
    let expected_types = super::super::extract_syntactic_type_only_import_names(source)
        .into_iter()
        .map(CompactString::new)
        .collect::<FxHashSet<_>>();
    assert_eq!(actual.1, expected_types, "{source}");
}
