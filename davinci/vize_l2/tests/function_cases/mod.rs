mod rejection;
mod semantics;
mod typed_parameters;
mod unwind;

use oxc_parser::{Parser, ProgramObservation};
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceBlock, SourceRoot};
use vize_l2::file::FileArtifact;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

fn parse<'a>(arena: &'a Allocator, source: &'a str) -> ProgramObservation<'a> {
    Parser::new(arena, source, SourceType::mjs()).parse_observed()
}

fn finish<'a>(
    arena: &'a Allocator,
    parsed: &ProgramObservation<'a>,
) -> Result<FileArtifact<'a>, &'static str> {
    let admitted = parsed.admitted().ok_or("original Program admission")?;
    let source = admitted.source();
    let block = SourceRoot::new(source)
        .map_err(|_| "source root")?
        .whole_block();
    finish_block(arena, parsed, block)
}

fn finish_block<'a>(
    arena: &'a Allocator,
    parsed: &ProgramObservation<'a>,
    block: SourceBlock<'a>,
) -> Result<FileArtifact<'a>, &'static str> {
    let mut producer = FileProducer::new(arena, block.root_source()).map_err(|_| "file source")?;
    producer
        .program(
            ProgramInput::checked(
                parsed.admitted().ok_or("original Program admission")?,
                block,
                7,
            )
            .map_err(|_| "checked original block")?,
            ProgramScope::Module,
        )
        .map_err(|_| "program admission")?;
    producer.finish().map_err(|_| "canonical finish")
}
