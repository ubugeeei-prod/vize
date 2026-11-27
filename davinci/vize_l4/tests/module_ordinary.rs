use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{
        Vue,
        vue::{DescriptorOptions, L1Ir, ScriptView},
    },
    embed::{
        EmbedSource,
        syntax::{EmbedSyntax, ProgramOptions, parse_program_once},
    },
    markup::SurfaceParseOptions,
};
use vize_l2::{
    ir::L2Ir,
    lang::js::{L2IrProducer, ProgramInput, ProgramScope, VueOrdinaryEmpty},
};
use vize_l4::{
    module::{ordinary::emit_ordinary_empty, setup::COMPONENT_BINDING},
    write::{NoLinks, Recorded},
};

mod module_ordinary_cases;

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
        Ok(Self {
            l1ir: descriptor,
            syntax,
        })
    }
    fn script(&self) -> Result<ScriptView<'_, 'a>, &'static str> {
        self.l1ir
            .admitted()
            .map_err(|_| "original descriptor")?
            .ordinary()
            .ok_or("actual ordinary")
    }
    fn file(&self, arena: &'a Allocator) -> Result<L2Ir<'a>, &'static str> {
        let script = self.script()?;
        let mut producer = L2IrProducer::new(arena, self.l1ir.source()).map_err(|_| "File")?;
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
        l2ir: &'f L2Ir<'a>,
    ) -> Result<VueOrdinaryEmpty<'f, '_, '_, 'a>, &'static str> {
        VueOrdinaryEmpty::checked(
            l2ir,
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
