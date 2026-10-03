use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions, ScriptRole, ScriptView},
    },
    embed::{
        EmbedSource,
        syntax::{NativeSyntax, ProgramOptions, parse_program_once},
    },
};
use vize_l2::{
    file::FileArtifact,
    lang::js::{
        FileProducer, OrdinaryIssue, OrdinaryIssueKind, ProgramInput, ProgramScope,
        VueOrdinaryEmpty,
    },
};

mod vue_ordinary_cases;

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
        let admitted = descriptor.admitted().map_err(|_| "descriptor")?;
        let script = admitted
            .ordinary()
            .or_else(|| admitted.setup())
            .ok_or("script")?;
        let syntax = parse_program_once(
            arena,
            EmbedSource::authored(source, script.block().span()).map_err(|_| "source")?,
            ProgramOptions::module(script.lang()),
        );
        Ok(Self { descriptor, syntax })
    }
    fn script(&self) -> Result<ScriptView<'_, 'a>, &'static str> {
        let admitted = self.descriptor.admitted().map_err(|_| "descriptor")?;
        admitted
            .ordinary()
            .or_else(|| admitted.setup())
            .ok_or("script")
    }
    fn producer(&self, arena: &'a Allocator) -> Result<FileProducer<'a>, &'static str> {
        let script = self.script()?;
        let mut producer =
            FileProducer::new(arena, self.descriptor.source()).map_err(|_| "file")?;
        producer
            .program(
                ProgramInput::checked(
                    self.syntax.admitted_program().ok_or("Program")?,
                    script.block(),
                    script.container_index(),
                )
                .map_err(|_| "input")?,
                if script.role() == ScriptRole::Ordinary {
                    ProgramScope::Module
                } else {
                    ProgramScope::Nested
                },
            )
            .map_err(|_| "walk")?;
        Ok(producer)
    }
    fn file(&self, arena: &'a Allocator) -> Result<FileArtifact<'a>, &'static str> {
        self.producer(arena)?.finish().map_err(|_| "File")
    }
    fn checked<'f>(
        &self,
        file: &'f FileArtifact<'a>,
    ) -> Result<Result<VueOrdinaryEmpty<'f, '_, '_, 'a>, OrdinaryIssue>, &'static str> {
        Ok(VueOrdinaryEmpty::checked(
            file,
            self.script()?,
            self.syntax.admitted_program().ok_or("Program")?,
        ))
    }
}
fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required original law")
    }
}
fn equal<T: PartialEq>(actual: T, expected: T) -> Result<(), &'static str> {
    check(actual == expected)
}
fn kind<T>(result: Result<T, OrdinaryIssue>) -> Result<OrdinaryIssueKind, &'static str> {
    result
        .err()
        .map(|issue| issue.kind)
        .ok_or("expected refusal")
}
