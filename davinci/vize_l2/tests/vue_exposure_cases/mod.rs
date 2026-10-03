use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::SurfaceParseOptions;
use vize_l1::container::{
    Vue,
    vue::{DescriptorObservation, DescriptorOptions, ScriptRole, ScriptView},
};
use vize_l1::embed::{
    EmbedSource,
    syntax::{NativeSyntax, ProgramOptions, parse_program_once},
};
use vize_l2::file::{
    FileArtifact,
    vue::{ExposureIssue, ExposureIssueKind, VueExposure},
};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

mod admission;
mod bindings;
mod invocations;

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
        let view = descriptor.admitted().map_err(|_| "actual descriptor")?;
        let script = view
            .setup()
            .or_else(|| view.ordinary())
            .ok_or("actual script")?;
        let syntax = parse_program_once(
            arena,
            EmbedSource::authored(source, script.block().span()).map_err(|_| "actual block")?,
            ProgramOptions::module(script.lang()),
        );
        Ok(Self { descriptor, syntax })
    }

    fn script(&self) -> Result<ScriptView<'_, 'a>, &'static str> {
        let view = self
            .descriptor
            .admitted()
            .map_err(|_| "actual descriptor")?;
        view.setup()
            .or_else(|| view.ordinary())
            .ok_or("actual script")
    }

    fn file(&self, arena: &'a Allocator) -> Result<FileArtifact<'a>, &'static str> {
        let script = self.script()?;
        let mut producer =
            FileProducer::new(arena, self.descriptor.source()).map_err(|_| "actual file")?;
        producer
            .program(
                ProgramInput::checked(
                    self.syntax.admitted_program().ok_or("actual Program")?,
                    script.block(),
                    script.container_index(),
                )
                .map_err(|_| "actual input")?,
                if script.role() == ScriptRole::Setup {
                    ProgramScope::Nested
                } else {
                    ProgramScope::Module
                },
            )
            .map_err(|_| "actual unit")?;
        producer.finish().map_err(|_| "actual artifact")
    }

    fn view<'f>(
        &self,
        file: &'f FileArtifact<'a>,
    ) -> Result<Result<VueExposure<'f, '_, '_, 'a>, ExposureIssue>, &'static str> {
        Ok(VueExposure::checked(
            file,
            self.script()?,
            self.syntax.admitted_program().ok_or("actual Program")?,
        ))
    }
}

fn kind(
    result: Result<VueExposure<'_, '_, '_, '_>, ExposureIssue>,
) -> Result<ExposureIssueKind, &'static str> {
    match result {
        Err(issue) => Ok(issue.kind),
        Ok(_) => Err("expected genuine admission refusal"),
    }
}
