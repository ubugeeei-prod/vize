//! Independent assurance parse; no production module-read storage is consumed.

use crate::sfc::SfcScriptBlock;
use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::collections::BTreeSet;
use vize_carton::CompactString;

pub(super) fn read(script: Option<&SfcScriptBlock<'_>>) -> Option<BTreeSet<CompactString>> {
    let Some(script) = script else {
        return Some(BTreeSet::new());
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        &script.content,
        if matches!(script.lang.as_deref(), Some("jsx" | "tsx")) {
            SourceType::tsx()
        } else {
            SourceType::ts()
        },
    )
    .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_nodes(true)
        .build(&parsed.program);
    let semantic = built.semantic;
    let scoping = semantic.scoping();
    if !built.diagnostics.is_empty()
        || scoping
            .scope_descendants_from_root()
            .any(|scope| scoping.scope_flags(scope).contains_direct_eval())
    {
        return None;
    }
    let mut reads = BTreeSet::new();
    for node in semantic.nodes().iter() {
        let AstKind::IdentifierReference(id) = node.kind() else {
            continue;
        };
        let Some(reference) = id.reference_id.get().map(|id| scoping.get_reference(id)) else {
            continue;
        };
        let flags = reference.flags();
        if reference.symbol_id().is_none()
            && (flags.is_read() || flags.is_type() || flags.is_value_as_type())
        {
            reads.insert(CompactString::new(id.name.as_str()));
        }
    }
    Some(reads)
}
