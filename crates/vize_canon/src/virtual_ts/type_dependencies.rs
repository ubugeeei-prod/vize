//! Free names in a type expression, respecting TypeScript lexical scopes.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, cstr};

pub(super) fn free_names(source: &str) -> Option<FxHashSet<CompactString>> {
    // An assertion gives the parser a type context without introducing a
    // synthetic declaration that could shadow an authored type name.
    let source = cstr!("null as {source};");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source.as_str(), SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let scoping = semantic.scoping();
    Some(
        scoping
            .root_unresolved_references_ids()
            .flatten()
            .map(|id| CompactString::new(semantic.reference_name(scoping.get_reference(id))))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::free_names;

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
