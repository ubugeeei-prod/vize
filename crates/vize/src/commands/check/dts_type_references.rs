//! References in copied declaration text, with TypeScript binding scopes.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};
use vize_l0::{String, ToCompactString, cstr};

pub(super) struct TypeReference {
    pub(super) name: String,
    pub(super) span: Span,
}

/// Names free in a type expression or a complete local declaration. Property
/// keys, strings, comments and nested type parameters are not references.
/// A declaration's own recursive references are retained as well.
pub(super) fn references(text: &str) -> Vec<TypeReference> {
    let trimmed = text.trim_start();
    let declaration = ["type ", "interface ", "declare ", "export "]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix));
    let prefix = if declaration { "" } else { "null as " };
    let source = cstr!("{prefix}{text};");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source.as_str(), SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let scoping = semantic.scoping();
    let root = scoping.root_scope_id();
    let declared = scoping
        .symbol_ids()
        .filter(|&symbol| scoping.symbol_scope_id(symbol) == root)
        .flat_map(|symbol| scoping.get_resolved_reference_ids(symbol).iter().copied());
    let mut references = scoping
        .root_unresolved_references_ids()
        .flatten()
        .chain(declared)
        .filter_map(|id| {
            let reference = scoping.get_reference(id);
            let span = semantic.reference_span(reference);
            let start = (span.start as usize).checked_sub(prefix.len())?;
            let end = (span.end as usize).checked_sub(prefix.len())?;
            (end <= text.len()).then(|| TypeReference {
                name: semantic.reference_name(reference).to_compact_string(),
                span: Span::new(start as u32, end as u32),
            })
        })
        .collect::<Vec<_>>();
    references.sort_unstable_by_key(|reference| reference.span.start);
    references.dedup_by_key(|reference| reference.span.start);
    references
}

/// A conservative lexical rejection before scoped parsing. Escaped identifiers
/// always take the parser path; strings/comments may cause harmless false hits.
pub(super) fn may_reference(text: &str, mut named: impl FnMut(&str) -> bool) -> bool {
    text.contains('\\')
        || text
            .split(|ch| !oxc_syntax::identifier::is_identifier_part(ch))
            .any(|word| !word.is_empty() && named(word))
}
