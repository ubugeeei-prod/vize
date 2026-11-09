use vize_l0::{Allocator, SourceBlock, SourceRoot};
use vize_l1::embed::syntax::{NativeSyntax, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l1_to_l2::native::{NativeComponent, NativeProduced};
use vize_l2::file::{Declaration, TemplatePolicy, TemplateScope};
use vize_l2::lang::js::FileProducer;

#[derive(Clone, Copy)]
struct LexicalValues;

impl TemplatePolicy for LexicalValues {
    fn visible(self, _declaration: &Declaration) -> bool {
        true
    }
}

pub fn block<'a>(source: &'a str, content: &str) -> Option<SourceBlock<'a>> {
    let start = source.find(content)?;
    let authored = source.get(start..start.checked_add(content.len())?)?;
    SourceRoot::new(source)
        .ok()?
        .block(authored, u32::try_from(start).ok()?)
        .ok()
}

pub fn script<'a>(
    arena: &'a Allocator,
    source: &'a str,
    content: &str,
) -> Option<(NativeSyntax<'a>, SourceBlock<'a>)> {
    let block = block(source, content)?;
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, block.span()).ok()?,
        ProgramOptions::module(Lang::Js),
    );
    Some((syntax, block))
}

pub fn construct<'a>(
    arena: &'a Allocator,
    source: &'a str,
    template: &str,
    producer: &mut FileProducer<'a>,
    scope: TemplateScope,
) -> Option<NativeProduced<'a>> {
    let component = NativeComponent::parse_in(arena, block(source, template)?).ok()?;
    // Diagnostic construction exercises real File rows without certifying
    // original Vue template custody or runtime access classes.
    let mut region = producer.template_region(scope, LexicalValues).ok()?;
    component.construct_in(&mut region, Lang::Js).ok()
}
