use vize_l0::{Allocator, SourceBlock, SourceRoot, Span};
use vize_l1::embed::syntax::{NativeSyntax, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l1_to_l2::native::{NativeComponent, NativeProduced};
use vize_l1_to_l2::vue_file::VueFileProducer;

pub fn block<'a>(source: &'a str, content: &str) -> Option<SourceBlock<'a>> {
    let start = source.find(content)?;
    let authored = source.get(start..start.checked_add(content.len())?)?;
    SourceRoot::new(source)
        .ok()?
        .block(authored, start as u32)
        .ok()
}

pub fn script<'a>(
    arena: &'a Allocator,
    source: &'a str,
    content: &str,
    lang: Lang,
) -> Option<(NativeSyntax<'a>, SourceBlock<'a>)> {
    let block = block(source, content)?;
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(lang),
    );
    Some((syntax, block))
}

pub fn construct<'a>(
    arena: &'a Allocator,
    source: &'a str,
    template: &str,
    producer: &mut VueFileProducer<'a>,
) -> Option<NativeProduced<'a>> {
    let component = NativeComponent::parse_in(arena, block(source, template)?).ok()?;
    let mut region = producer.template_region().ok()?;
    component.construct_in(&mut region, Lang::Js).ok()
}

pub fn whole_span(source: &str) -> Span {
    Span::new(0, source.len() as u32)
}
