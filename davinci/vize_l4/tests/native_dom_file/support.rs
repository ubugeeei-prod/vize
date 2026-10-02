//! Real checked Program and Vue factories; no fixture AST reconstruction.

use vize_l0::{Allocator, SourceBlock, SourceRoot};
use vize_l1::embed::syntax::{NativeSyntax, ProgramOptions, parse_program_once};
use vize_l1::embed::{EmbedSource, Lang};
use vize_l1_to_l2::native::{NativeComponent, NativeProduced};
use vize_l1_to_l2::vue_file::VueFileProducer;

pub fn block<'a>(source: &'a str, content: &str) -> SourceBlock<'a> {
    // Selection belongs only to the authored dev fixture. SourceRoot still
    // authenticates the original source/window used by the real factories.
    let start = source.find(content).unwrap();
    let end = start.checked_add(content.len()).unwrap();
    SourceRoot::new(source)
        .unwrap()
        .block(
            source.get(start..end).unwrap(),
            u32::try_from(start).unwrap(),
        )
        .unwrap()
}

pub fn script<'a>(
    arena: &'a Allocator,
    source: &'a str,
    content: &str,
) -> (NativeSyntax<'a>, SourceBlock<'a>) {
    let block = block(source, content);
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, block.span()).unwrap(),
        ProgramOptions::module(Lang::Js),
    );
    assert_eq!(syntax.diagnostics().count(), 0);
    (syntax, block)
}

pub fn construct<'a>(
    arena: &'a Allocator,
    source: &'a str,
    template: &str,
    producer: &mut VueFileProducer<'a>,
) -> NativeProduced<'a> {
    let component = NativeComponent::parse_in(arena, block(source, template)).unwrap();
    let mut region = producer.template_region().unwrap();
    let native = component.construct_in(&mut region, Lang::Js).unwrap();
    assert!(native.is_supported(), "{source}");
    native
}
