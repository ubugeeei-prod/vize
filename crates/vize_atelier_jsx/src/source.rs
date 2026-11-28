//! One generic parse/analysis pipeline, with a zero-sized ordinary root policy.

use crate::diagnostics::JsxDiagnostic;
use crate::syntax::RootProjection;
use crate::{JsxLang, Lowerer, LoweringOptions, SpanMapper, analyze, finder, parse};
use oxc_semantic::SemanticBuilder;
use vize_croquis::Croquis;
use vize_l0::Allocator;

pub(super) struct LowerParts<'a, P: RootProjection<'a>> {
    pub(super) roots: std::vec::Vec<P::Root>,
    pub(super) analysis: Croquis,
    pub(super) diagnostics: std::vec::Vec<JsxDiagnostic>,
    pub(super) custom_element_spans: std::vec::Vec<(u32, u32)>,
}

pub(super) fn lower_parts<'a, P: RootProjection<'a>>(
    bump: &'a Allocator,
    allocator: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
    options: LoweringOptions<'_>,
) -> LowerParts<'a, P> {
    let parse_source = parse::prepare_source_for_parse(source, lang);
    let parsed = parse::parse_module(allocator, parse_source.as_ref(), lang);
    let scoping = Some(
        SemanticBuilder::new()
            .build(&parsed.program)
            .semantic
            .into_scoping(),
    );
    let mapper = SpanMapper::new(source);
    let mut lowerer = Lowerer::with_compat(bump, &mapper, options.compat, options.babel, scoping)
        .with_slot_parameter_types(options.preserve_slot_parameter_types);
    lowerer.collect_boolean_bindings(&parsed.program);
    for diagnostic in parsed.diagnostics {
        lowerer.report(diagnostic);
    }
    let roots =
        finder::lower_program_roots::<P>(&parsed.program, &mut lowerer, options.default_mode);
    let analysis = analyze::analyze_program(&parsed.program, source);
    let (diagnostics, custom_element_spans) = lowerer.into_compat_parts();
    LowerParts {
        roots,
        analysis,
        diagnostics,
        custom_element_spans,
    }
}
