//! Genuine once-parsed Module and completed File fixtures; no detached AST.

use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::syntax::{NativeSyntax, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l2::file::FileArtifact;
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

pub fn syntax<'a>(arena: &'a Allocator, source: &'a str, lang: Lang) -> Option<NativeSyntax<'a>> {
    let block = SourceRoot::new(source).ok()?.whole_block();
    Some(parse_program_once(
        arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(lang),
    ))
}

pub fn file<'a>(arena: &'a Allocator, source: &'a str, lang: Lang) -> Option<FileArtifact<'a>> {
    let block = SourceRoot::new(source).ok()?.whole_block();
    let syntax = syntax(arena, source, lang)?;
    let mut producer = FileProducer::new(arena, source).ok()?;
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program()?, block, 0).ok()?,
            ProgramScope::Module,
        )
        .ok()?;
    producer.finish().ok()
}
