use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::embed::{
    EmbedSource,
    syntax::{EmbedSyntax, ProgramOptions, parse_program_once},
};
use vize_l1::{
    container::{
        Vue,
        vue::{DescriptorOptions, L1Ir, ScriptRole, ScriptView},
    },
    markup::SurfaceParseOptions,
};
use vize_l2::file::{
    L2Ir,
    vue::{ExposureIssue, VueExposure},
};
use vize_l2::lang::js::{L2IrProducer, ProgramInput, ProgramScope};

mod vue_setup_cases;

struct Observed<'a> {
    l1ir: L1Ir<'a>,
    syntax: EmbedSyntax<'a>,
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
        Ok(Self {
            l1ir: descriptor,
            syntax,
        })
    }

    fn script(&self) -> Result<ScriptView<'_, 'a>, &'static str> {
        let view = self.l1ir.admitted().map_err(|_| "actual descriptor")?;
        view.setup()
            .or_else(|| view.ordinary())
            .ok_or("actual script")
    }

    fn file(&self, arena: &'a Allocator) -> Result<L2Ir<'a>, &'static str> {
        let script = self.script()?;
        let mut producer =
            L2IrProducer::new(arena, self.l1ir.source()).map_err(|_| "actual file")?;
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
        l2ir: &'f L2Ir<'a>,
    ) -> Result<Result<VueExposure<'f, '_, '_, 'a>, ExposureIssue>, &'static str> {
        Ok(VueExposure::checked(
            l2ir,
            self.script()?,
            self.syntax.admitted_program().ok_or("actual Program")?,
        ))
    }
}
