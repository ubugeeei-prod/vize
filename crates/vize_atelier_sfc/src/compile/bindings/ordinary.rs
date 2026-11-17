//! The bindings a normal `<script>` exposes to the template of its paired
//! `<script setup>`, read from the native L2 normal-script facts (#6844, S7).
//!
//! The block's content is parsed once by L1 with its `lang`, as the setup
//! route does, and the L2 Program walk runs `OrdinaryObserver` over it. The
//! facts classify bindings as Vue's `compileScript` does for a normal script
//! beside `<script setup>`: runtime imports and top-level declarations, with
//! `ref`/`reactive` and their siblings recognized only when imported from
//! `vue`. A script the levels refuse exposes nothing; nothing falls back to
//! another analysis.

use vize_l0::{Allocator, SourceRoot, Span, ToCompactString, profile};
use vize_l1::embed::{EmbedSource, syntax::parse_program_once};
use vize_l2::framework::vue::OrdinaryObserver;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

use crate::compile::setup_facts::{binding_type, program_options};
use crate::types::BindingMetadata;

/// The normal script's runtime bindings; the first declaration of a name
/// wins, as in the module.
pub(in crate::compile) fn ordinary(content: &str, lang: Option<&str>) -> BindingMetadata {
    profile!(
        "atelier.sfc.normal_script.register_bindings",
        observe(content, lang).unwrap_or_default()
    )
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
    FileProducer::new(&arena, content)
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
    Some(bindings)
}
