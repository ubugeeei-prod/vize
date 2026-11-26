//! The top-level bindings a normal `<script>` exposes to the template of a
//! paired `<script setup>`, read from the native L2 ordinary facts (#6844, S7).
//!
//! The script content is parsed once by L1 with the block's `lang` and the L2
//! Program walk runs `OrdinaryObserver` over it. A script the levels refuse
//! forwards no bindings, as a script that did not parse did before.

use oxc_ast::ast::{Declaration, Statement, TSEnumDeclaration};
use vize_l0::{Allocator, SourceRoot, Span, ToCompactString, profile};
use vize_l1::embed::{EmbedSource, syntax::parse_program_once};
use vize_l2::framework::vue::OrdinaryObserver;
use vize_l2::lang::js::{L2IrProducer, ProgramInput, ProgramScope};

use crate::compile::setup_facts::{binding_type, program_options};
use crate::types::BindingMetadata;

/// Runtime imports and top-level declarations, classified as setup bindings
/// are. The first declaration of a name keeps it; a runtime enum takes the
/// compiler's literal classification, as in the setup route.
pub(in crate::compile) fn ordinary(content: &str, lang: Option<&str>) -> BindingMetadata {
    profile!(
        "atelier.sfc.normal_script.register_bindings",
        observe(content, lang)
    )
    .unwrap_or_default()
}

fn observe(content: &str, lang: Option<&str>) -> Option<BindingMetadata> {
    let arena = Allocator::default();
    let span = Span::new(0, u32::try_from(content.len()).ok()?);
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(content, span).ok()?,
        program_options(lang),
    );
    let block = SourceRoot::new(content).ok()?.whole_block();
    let input = ProgramInput::checked(syntax.admitted_program()?, block, 0).ok()?;
    let mut observer = OrdinaryObserver::new();
    L2IrProducer::new(&arena, content)
        .ok()?
        .program_observed(input, ProgramScope::Module, &mut observer)
        .ok()?;
    let facts = observer.finish().ok()?;
    let mut bindings = BindingMetadata::default();
    for binding in facts.bindings() {
        let name = content.get(binding.name.start as usize..binding.name.end as usize)?;
        bindings
            .bindings
            .entry(name.to_compact_string())
            .or_insert(binding_type(binding.kind));
    }
    for enumeration in syntax
        .admitted_program()?
        .program()
        .body
        .iter()
        .filter_map(runtime_enum)
    {
        crate::script::register_enum(&mut bindings, enumeration);
    }
    Some(bindings)
}

/// A top-level runtime `enum`, exported or not.
fn runtime_enum<'s, 'a>(statement: &'s Statement<'a>) -> Option<&'s TSEnumDeclaration<'a>> {
    let enumeration = match statement {
        Statement::TSEnumDeclaration(enumeration) => enumeration,
        Statement::ExportNamedDeclaration(export) => match &export.declaration {
            Some(Declaration::TSEnumDeclaration(enumeration)) => enumeration,
            _ => return None,
        },
        _ => return None,
    };
    (!enumeration.declare).then_some(enumeration.as_ref())
}

#[cfg(test)]
mod tests;
