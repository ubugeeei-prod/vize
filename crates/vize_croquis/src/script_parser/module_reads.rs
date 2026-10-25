//! Demand-only lexical reads of the original normal-script Program.

use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilder;
use vize_carton::{CompactString, FxHashSet};

#[derive(Debug)]
pub(crate) enum ModuleReadFacts {
    Unknown,
    Known(FxHashSet<CompactString>),
}

pub(super) fn observe(program: &Program<'_>) -> ModuleReadFacts {
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(program);
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    if !built.diagnostics.is_empty()
        || scoping
            .scope_descendants_from_root()
            .any(|scope| scoping.scope_flags(scope).contains_direct_eval())
    {
        return ModuleReadFacts::Unknown;
    }
    let reads = scoping
        .root_unresolved_references()
        .iter()
        .filter(|(_, references)| {
            references.iter().any(|id| {
                let flags = scoping.get_reference(*id).flags();
                flags.is_read() || flags.is_type() || flags.is_value_as_type()
            })
        })
        .map(|(name, _)| CompactString::new(name.as_str()))
        .collect();
    ModuleReadFacts::Known(reads)
}
