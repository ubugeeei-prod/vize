use vize_l0::{
    Allocator, SourceBlock, SourceRoot,
    config::{VueDialect, VueVersion},
};
pub mod body;
use vize_l1::SurfaceParseOptions;
use vize_l1::container::{
    Vue,
    vue::{DescriptorObservation, DescriptorOptions, ScriptRole, ScriptView},
};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape,
    syntax::{NativeSyntax, ProgramOptions, RetainedExpression, parse_once, parse_program_once},
};
use vize_l2::{
    artifact::{ComponentBody, ComponentFactory},
    expr::{JsExpr, js::JsCoordinates},
    file::{Declaration, FileArtifact, TemplatePolicy, TemplateScope, vue::VueExposure},
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};

#[derive(Clone, Copy)]
struct LexicalValues;
impl TemplatePolicy for LexicalValues {
    fn visible(self, _: &Declaration) -> bool {
        true
    }
}

pub struct Observed<'a> {
    descriptor: DescriptorObservation<'a>,
    syntax: NativeSyntax<'a>,
}

impl<'a> Observed<'a> {
    pub fn new(arena: &'a Allocator, source: &'a str) -> Option<Self> {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let admitted = descriptor.admitted().ok()?;
        let script = admitted.setup().or_else(|| admitted.ordinary())?;
        let syntax = parse_program_once(
            arena,
            EmbedSource::authored(source, script.block().span()).ok()?,
            ProgramOptions::module(script.lang()),
        );
        Some(Self { descriptor, syntax })
    }

    pub fn script(&self) -> Option<ScriptView<'_, 'a>> {
        let view = self.descriptor.admitted().ok()?;
        view.setup().or_else(|| view.ordinary())
    }

    fn producer(&self, arena: &'a Allocator) -> Option<FileProducer<'a>> {
        let script = self.script()?;
        let mut producer = FileProducer::new(arena, self.descriptor.source()).ok()?;
        producer
            .program(
                ProgramInput::checked(
                    self.syntax.admitted_program()?,
                    script.block(),
                    script.container_index(),
                )
                .ok()?,
                if script.role() == ScriptRole::Setup {
                    ProgramScope::Nested
                } else {
                    ProgramScope::Module
                },
            )
            .ok()?;
        Some(producer)
    }

    pub fn element<B: ComponentBody<'a>>(
        &self,
        arena: &'a Allocator,
        span: vize_l0::Span,
        body: B,
    ) -> Option<FileArtifact<'a>> {
        let mut producer = self.producer(arena)?;
        {
            let mut region = producer
                .template_region(TemplateScope::LastUnit, LexicalValues)
                .ok()?;
            region
                .element(
                    "p",
                    vize_l2::op::Namespace::Html,
                    vize_l0::Vec::new_in(&arena),
                    span,
                    body,
                )
                .ok()?;
        }
        producer.finish().ok()
    }

    pub fn file(
        &self,
        arena: &'a Allocator,
        expressions: &[&'a JsExpr<'a>],
    ) -> Option<FileArtifact<'a>> {
        let mut producer = self.producer(arena)?;
        {
            // This real diagnostic factory proves lexical rows, not native
            // template/head custody or Vue product construction.
            let mut region = producer
                .template_region(TemplateScope::LastUnit, LexicalValues)
                .ok()?;
            for expression in expressions {
                region.interpolation(expression, expression.span).ok()?;
            }
        }
        producer.finish().ok()
    }

    pub fn exposure<'f>(&self, file: &'f FileArtifact<'a>) -> Option<VueExposure<'f, '_, '_, 'a>> {
        VueExposure::checked(file, self.script()?, self.syntax.admitted_program()?).ok()
    }
}

pub fn expression<'a>(
    arena: &'a Allocator,
    source: &'a str,
    text: &str,
    lang: Lang,
) -> Option<RetainedExpression<'a>> {
    let start = source.rfind(text)?;
    let block: SourceBlock<'a> = SourceRoot::new(source)
        .ok()?
        .block(
            source.get(start..start.checked_add(text.len())?)?,
            u32::try_from(start).ok()?,
        )
        .ok()?;
    expression_at(arena, source, block.span(), lang)
}

pub fn expression_at<'a>(
    arena: &'a Allocator,
    source: &'a str,
    span: vize_l0::Span,
    lang: Lang,
) -> Option<RetainedExpression<'a>> {
    parse_once(
        arena,
        Embed {
            source: EmbedSource::authored(source, span).ok()?,
            grammar: Grammar {
                shape: Shape::Expr,
                lang,
            },
        },
    )
    .into_expression()
    .ok()
}

pub fn bridge<'a>(
    arena: &'a Allocator,
    file: &'a str,
    owner: &RetainedExpression<'a>,
) -> Option<&'a JsExpr<'a>> {
    let source = owner.source();
    let coordinates = JsCoordinates::checked(
        file,
        source.text(),
        source.span(),
        owner.parser_prefix(),
        &[],
    )
    .ok()?;
    JsExpr::from_retained_in(
        arena,
        owner.expression()?,
        source.text(),
        source.span(),
        coordinates,
    )
    .ok()
}
