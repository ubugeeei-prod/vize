//! Conservative, demand-only free reads for expressions without a retained AST.

use super::IdentifierRef;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{CompactString, cstr};

/// Unknown syntax or direct eval cannot prove absence of a setup read.
pub(super) fn checked_reads(source: &str) -> Option<Vec<CompactString>> {
    checked_semantic_references(source).map(|references| {
        references
            .into_iter()
            .map(|reference| reference.name)
            .collect()
    })
}

pub(super) fn checked_references(source: &str) -> Option<Vec<IdentifierRef>> {
    let references = checked_semantic_references(source)?;
    references
        .iter()
        .all(|reference| {
            let start = reference.offset as usize;
            start
                .checked_add(reference.name.len())
                .is_some_and(|end| source.get(start..end) == Some(reference.name.as_str()))
        })
        .then_some(references)
}

fn checked_semantic_references(source: &str) -> Option<Vec<IdentifierRef>> {
    let allocator = Allocator::default();
    let wrapped = cstr!("({source})");
    let expression = Parser::new(&allocator, &wrapped, SourceType::tsx()).parse();
    let wrapped_view = !expression.panicked && expression.diagnostics.is_empty();
    let parsed = if !wrapped_view {
        // Inline event handlers may be statement bodies.
        Parser::new(&allocator, source, SourceType::tsx()).parse()
    } else {
        expression
    };
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_nodes(true)
        .build(&parsed.program);
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    if !built.diagnostics.is_empty()
        || scoping
            .scope_descendants_from_root()
            .any(|scope| scoping.scope_flags(scope).contains_direct_eval())
    {
        return None;
    }
    let mut references = Vec::new();
    for id in scoping.root_unresolved_references_ids().flatten() {
        let reference = scoping.get_reference(id);
        let flags = reference.flags();
        if !(flags.is_read() || flags.is_type() || flags.is_value_as_type()) {
            continue;
        }
        let span = semantic.reference_span(reference);
        let start = span.start.checked_sub(u32::from(wrapped_view))?;
        let name = semantic.reference_name(reference);
        references.push(IdentifierRef::new(name, start));
    }
    Some(references)
}
