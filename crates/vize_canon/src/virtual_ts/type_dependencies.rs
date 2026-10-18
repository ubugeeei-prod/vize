//! Free names in a type expression, respecting TypeScript lexical scopes.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, FxHashSet, cstr};

pub(super) fn free_names(source: &str) -> Option<FxHashSet<CompactString>> {
    // An assertion gives the parser a type context without introducing a
    // synthetic declaration that could shadow an authored type name.
    let source = cstr!("null as {source};");
    declaration_free_names(source.as_str())
}

/// Resolve an actual declaration, retaining its own generic binders. This is
/// used when deciding which hoisted local types capture an SFC parameter.
pub(super) fn declaration_free_names(source: &str) -> Option<FxHashSet<CompactString>> {
    analyze_declaration(source).map(|(names, _)| names)
}

/// The underlying type declaration excludes an `export` wrapper. Its exact
/// AST bytes can therefore also be retained inside a setup function.
pub(super) fn local_type_declaration(source: &str) -> Option<(&str, FxHashSet<CompactString>)> {
    let (names, range) = analyze_declaration(source)?;
    Some((source.get(range?)?, names))
}

fn analyze_declaration(
    source: &str,
) -> Option<(FxHashSet<CompactString>, Option<std::ops::Range<usize>>)> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let span = (parsed.program.body.len() == 1)
        .then(|| parsed.program.body.first())
        .flatten()
        .and_then(|statement| match statement {
            Statement::TSTypeAliasDeclaration(declaration) => Some(declaration.span()),
            Statement::TSInterfaceDeclaration(declaration) => Some(declaration.span()),
            Statement::ExportNamedDeclaration(export) => {
                export
                    .declaration
                    .as_ref()
                    .and_then(|declaration| match declaration {
                        Declaration::TSTypeAliasDeclaration(declaration) => {
                            Some(declaration.span())
                        }
                        Declaration::TSInterfaceDeclaration(declaration) => {
                            Some(declaration.span())
                        }
                        _ => None,
                    })
            }
            _ => None,
        });
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let scoping = semantic.scoping();
    Some((
        scoping
            .root_unresolved_references_ids()
            .flatten()
            .map(|id| CompactString::new(semantic.reference_name(scoping.get_reference(id))))
            .collect(),
        span.map(|span| span.start as usize..span.end as usize),
    ))
}

#[cfg(test)]
mod tests {
    use super::{declaration_free_names, free_names, local_type_declaration};

    #[test]
    fn setup_copy_uses_the_ast_declaration_inside_an_export_wrapper() {
        for source in [
            "export type Alias = Value;",
            "export interface Payload { value: Alias }",
        ] {
            let (declaration, _) = local_type_declaration(source).unwrap();
            assert_eq!(declaration, source.strip_prefix("export ").unwrap());
        }
    }

    #[test]
    fn declaration_dependencies_keep_local_generic_shadowing() {
        for (source, expected) in [
            (
                "type Value = T extends 'number' ? number : string",
                vec!["T"],
            ),
            ("type Alias = Value", vec!["Value"]),
            ("interface Payload { value: Alias; T: 'T' }", vec!["Alias"]),
            ("type Shadow<T> = T", vec![]),
            ("type Shadow<Value> = Value", vec![]),
            ("type Factory = <T>(value: T) => T", vec![]),
        ] {
            let mut actual: Vec<_> = declaration_free_names(source)
                .unwrap()
                .into_iter()
                .collect();
            actual.sort();
            assert_eq!(
                actual.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
                expected,
                "{source}"
            );
        }
    }

    #[test]
    fn type_dependencies_are_free_references_not_matching_text() {
        for (source, expected) in [
            ("{ Status: 'Status'; value: Status }", vec!["Status"]),
            ("{ 状態: '状態'; value: typeof 状態.member }", vec!["状態"]),
            ("<T>(value: T) => T", vec![]),
            ("{ [Key in keyof Input]: Input[Key] }", vec!["Input"]),
            ("Input extends infer Local ? Local : never", vec!["Input"]),
            ("typeof import('package').value", vec![]),
            ("Namespace.Type", vec!["Namespace"]),
            (
                "{ label: 'typeof state'; value: number /* State */ }",
                vec![],
            ),
        ] {
            let mut actual: Vec<_> = free_names(source).unwrap().into_iter().collect();
            actual.sort();
            assert_eq!(
                actual.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
                expected,
                "{source}"
            );
        }
        assert!(free_names("{ broken:").is_none());
    }
}
