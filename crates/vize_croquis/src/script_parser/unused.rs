//! Demand-only population of unread setup declarations from the retained AST.

use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilder;
use oxc_syntax::symbol::SymbolFlags;
use vize_carton::CompactString;

use super::ScriptParseResult;

mod generic_reads;

pub(crate) fn unused_setup_bindings(
    program: &Program<'_>,
    result: &ScriptParseResult,
    generic: Option<&str>,
) -> Vec<CompactString> {
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(program);
    let scoping = built.semantic.scoping();
    // Invalid semantics and direct eval make absence of a read unprovable.
    if !built.diagnostics.is_empty()
        || scoping
            .scope_descendants_from_root()
            .any(|scope| scoping.scope_flags(scope).contains_direct_eval())
    {
        return Vec::new();
    }
    let mut unused = Vec::new();
    for (name, symbol) in scoping.get_bindings(scoping.root_scope_id()) {
        let flags = scoping.symbol_flags(*symbol);
        if flags.intersects(SymbolFlags::TypeImport | SymbolFlags::Ambient)
            || (!flags.is_value() && !flags.contains(SymbolFlags::Import))
            || !result.bindings.contains(name.as_str())
            || !result.binding_spans.contains_key(name.as_str())
        {
            continue;
        }
        // Closures, defineExpose, JSX and typeof queries resolve to the same
        // symbol. A shadow with the same spelling never satisfies this read.
        if !scoping.get_resolved_references(*symbol).any(|reference| {
            let flags = reference.flags();
            flags.is_read() || flags.is_type() || flags.is_value_as_type()
        }) {
            unused.push(CompactString::new(name.as_str()));
        }
    }
    if !unused.is_empty()
        && let Some(generic) = generic
    {
        let Some(reads) = generic_reads::free_reads(generic) else {
            // An invalid attribute cannot prove any binding is unread.
            return Vec::new();
        };
        unused.retain(|name| !reads.contains(name));
    }
    unused.sort_unstable();
    unused
}

#[cfg(test)]
mod tests;
