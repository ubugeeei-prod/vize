use super::{
    FileArtifact, PositionQueryError, TemplateQueryError, TemplateSiteRef, TemplateSiteScope,
    TemplateSymbolRef,
};
use crate::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use vize_l0::Span;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::NativeTemplateComponent,
};

fn native<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    attempt(arena, source, true)
}

fn attempt<'a>(arena: &'a Allocator, source: &'a str, complete: bool) -> NativeTemplateFile<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor.admitted().unwrap();
    let selected = NativeTemplateComponent::parse_in(arena, admitted)
        .unwrap()
        .unwrap();
    let mut original =
        NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("native owner"));
    let syntax = admitted.setup().map(|setup| {
        parse_program_once(
            arena,
            EmbedSource::authored(source, setup.block().span()).unwrap(),
            ProgramOptions::module(setup.lang()),
        )
    });
    if let Some(syntax) = &syntax {
        original
            .setup_program(syntax.admitted_program().unwrap())
            .unwrap();
    }
    {
        let mut walk = original.begin().unwrap();
        for child in walk.selected().children() {
            if let Err(issue) = walk.child(child) {
                assert!(!complete, "{issue:?}");
                break;
            }
        }
        assert_eq!(walk.complete().is_ok(), complete);
    }
    core::hint::black_box(original.finish())
}

fn file<'f, 'a>(output: &'f NativeTemplateFile<'a>) -> &'f FileArtifact<'a> {
    output.view().unwrap().file().unwrap()
}

fn at(source: &str, needle: &str) -> u32 {
    source.find(needle).unwrap() as u32
}

fn site<'f, 'a>(file: &'f FileArtifact<'a>, source: &str, needle: &str) -> TemplateSiteRef<'f, 'a> {
    file.template_symbol_at_offset(at(source, needle))
        .unwrap()
        .unwrap()
}

fn uses(file: &FileArtifact<'_>, symbol: TemplateSymbolRef<'_, '_>) -> alloc::vec::Vec<Span> {
    let mut spans = alloc::vec::Vec::new();
    file.for_each_template_reference_to(symbol, |site| spans.push(site.span()))
        .unwrap();
    spans.sort_by_key(|span| (span.start, span.end));
    spans
}

mod for_heads;
mod handlers;
mod refusals;
