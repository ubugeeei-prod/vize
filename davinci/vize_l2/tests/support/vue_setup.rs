//! The repository's `<script setup>` corpus observed by `SetupObserver`.

#[path = "vue_corpus.rs"]
mod vue_corpus;
pub use vue_corpus::corpus;

use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::{
    Vue,
    vue::{DescriptorOptions, ScriptRole},
};
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l1::markup::SurfaceParseOptions;
use vize_l2::framework::vue::{SetupFacts, SetupFactsIncomplete, SetupObserver};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

pub fn slice(source: &str, base: u32, span: vize_l0::Span) -> Option<String> {
    source
        .get((span.start - base) as usize..(span.end - base) as usize)
        .map(String::from)
}

pub fn text(value: &str) -> String {
    String::from(value)
}

/// The facts of the file's `<script setup>` with its content and start, or
/// `None` when the native levels refuse the file first.
pub fn observe(source: &str) -> Option<(String, u32, Result<SetupFacts, SetupFactsIncomplete>)> {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = descriptor.admitted().ok()?;
    let script = view.setup()?;
    assert_eq!(script.role(), ScriptRole::Setup);
    let block = script.block();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(script.lang()),
    );
    let admitted = syntax.admitted_program()?;
    let mut producer = FileProducer::new(&arena, descriptor.source()).ok()?;
    let mut observer = SetupObserver::new();
    producer
        .program_observed(
            ProgramInput::checked(admitted, block, script.container_index()).ok()?,
            ProgramScope::Nested,
            &mut observer,
        )
        .ok()?;
    let facts = observer.finish();
    let content = source.get(block.span().start as usize..block.span().end as usize)?;
    Some((String::from(content), block.span().start, facts))
}
