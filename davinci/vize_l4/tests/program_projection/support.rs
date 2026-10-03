use vize_l0::{Allocator, SourceBlock, Span};
use vize_l1::embed::syntax::{NativeSyntax, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l2::file::FileArtifact;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

pub fn syntax<'a>(
    arena: &'a Allocator,
    block: SourceBlock<'a>,
    lang: Lang,
) -> Option<NativeSyntax<'a>> {
    Some(parse_program_once(
        arena,
        EmbedSource::authored(block.root_source(), block.span()).ok()?,
        ProgramOptions::module(lang),
    ))
}

pub fn file<'a>(arena: &'a Allocator, source: &'a str, lang: Lang) -> Option<FileArtifact<'a>> {
    let block = vize_l0::SourceRoot::new(source).ok()?.whole_block();
    let syntax = syntax(arena, block, lang)?;
    let mut producer = FileProducer::new(arena, source).ok()?;
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program()?, block, 17).ok()?,
            ProgramScope::Module,
        )
        .ok()?;
    producer.finish().ok()
}

pub fn whole(source: &str) -> Option<Span> {
    Some(Span::new(0, u32::try_from(source.len()).ok()?))
}
