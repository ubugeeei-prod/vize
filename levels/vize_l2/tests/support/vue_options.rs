//! A normal `<script>` observed by `OptionsObserver`.

use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::embed::{
    EmbedSource,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l1::markup::SurfaceParseOptions;
use vize_l2::framework::vue::{OptionsFacts, OptionsFactsIncomplete, OptionsObserver};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

pub type Observed = (String, u32, Result<OptionsFacts, OptionsFactsIncomplete>);

/// The facts of the file's normal `<script>` with its content and start,
/// or `None` when the native levels refuse the file first. JSX is not
/// enabled, so a script with JSX is refused here.
pub fn observe(source: &str) -> Option<Observed> {
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
    let script = view.ordinary()?;
    let block = script.block();
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(script.lang()),
    );
    let admitted = syntax.admitted_program()?;
    let mut producer = FileProducer::new(&arena, descriptor.source()).ok()?;
    let mut observer = OptionsObserver::new();
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
