use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions, ScriptView},
    },
    embed::{
        EmbedSource,
        syntax::{NativeSyntax, ProgramOptions, parse_program_once},
    },
};
use vize_l2::{
    file::FileArtifact,
    lang::js::{FileProducer, ProgramInput, ProgramScope, VueOrdinaryEmpty},
};
use vize_l4::{
    module::{ordinary::emit_ordinary_empty, setup::COMPONENT_BINDING},
    write::{NoLinks, Recorded},
};

mod module_ordinary_cases;

struct Observed<'a> {
    descriptor: DescriptorObservation<'a>,
    syntax: NativeSyntax<'a>,
}
impl<'a> Observed<'a> {
    fn new(arena: &'a Allocator, source: &'a str) -> Result<Self, &'static str> {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let script = descriptor
            .admitted()
            .map_err(|_| "original descriptor")?
            .ordinary()
            .ok_or("actual ordinary")?;
        let syntax = parse_program_once(
            arena,
            EmbedSource::authored(source, script.block().span()).map_err(|_| "actual source")?,
            ProgramOptions::module(script.lang()),
        );
        Ok(Self { descriptor, syntax })
    }
    fn script(&self) -> Result<ScriptView<'_, 'a>, &'static str> {
        self.descriptor
            .admitted()
            .map_err(|_| "original descriptor")?
            .ordinary()
            .ok_or("actual ordinary")
    }
    fn file(&self, arena: &'a Allocator) -> Result<FileArtifact<'a>, &'static str> {
        let script = self.script()?;
        let mut producer =
            FileProducer::new(arena, self.descriptor.source()).map_err(|_| "File")?;
        producer
            .program(
                ProgramInput::checked(
                    self.syntax.admitted_program().ok_or("original parser")?,
                    script.block(),
                    script.container_index(),
                )
                .map_err(|_| "actual input")?,
                ProgramScope::Module,
            )
            .map_err(|_| "sole walk")?;
        producer.finish().map_err(|_| "complete File")
    }
    fn checked<'f>(
        &self,
        file: &'f FileArtifact<'a>,
    ) -> Result<VueOrdinaryEmpty<'f, '_, '_, 'a>, &'static str> {
        VueOrdinaryEmpty::checked(
            file,
            self.script()?,
            self.syntax.admitted_program().ok_or("original parser")?,
        )
        .map_err(|_| "sealed ordinary")
    }
}
fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("original emission law")
    }
}
fn equal<T: PartialEq>(actual: T, expected: T) -> Result<(), &'static str> {
    check(actual == expected)
}
