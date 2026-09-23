//! Resolve free names in CSS v-bind expressions in the SFC setup scope.

use oxc_semantic::Semantic;
use vize_s0::cstr;

use super::{
    Allocator, AstKind, FxHashSet, IdeContext, Occurrence, Parser, SemanticBuilder, SourceType,
    Span, edits,
};

#[derive(Default)]
pub(super) struct Analysis {
    pub occurrences: Vec<Occurrence>,
    pub free_names: FxHashSet<vize_s0::String>,
}

pub(super) fn analyze(ctx: &IdeContext<'_>, semantic: &Semantic<'_>) -> Analysis {
    let Some(descriptor) = ctx.descriptor() else {
        return Analysis::default();
    };
    let Some(scope) = semantic.nodes().iter().find_map(|node| match node.kind() {
        AstKind::Function(function)
            if function.id.as_ref().is_some_and(|id| id.name == "__setup") =>
        {
            function.scope_id.get()
        }
        _ => None,
    }) else {
        return Analysis::default();
    };
    let mut result = Analysis::default();
    for style in &descriptor.styles {
        for range in vize_croquis::sfc::__internal::v_bind_expression_ranges(&style.content) {
            let Some(expression) = style.content.get(range.clone()) else {
                continue;
            };
            let code = cstr!("({expression})");
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &code, SourceType::ts()).parse();
            if parsed.panicked || !parsed.diagnostics.is_empty() {
                continue;
            }
            let css = SemanticBuilder::new()
                .with_build_nodes(true)
                .build(&parsed.program)
                .semantic;
            for node in css.nodes().iter() {
                let AstKind::IdentifierReference(id) = node.kind() else {
                    continue;
                };
                let Some(reference) = id.reference_id.get() else {
                    continue;
                };
                if css.scoping().get_reference(reference).symbol_id().is_some() {
                    continue;
                }
                result.free_names.insert(id.name.as_str().into());
                let Some(symbol) = semantic.scoping().find_binding(scope, id.name) else {
                    continue;
                };
                let start = style.loc.start + range.start + id.span.start as usize - 1;
                let end = style.loc.start + range.start + id.span.end as usize - 1;
                result.occurrences.push(Occurrence {
                    edit: edits::kind(&css, node.id()),
                    symbol,
                    generated: Span::new(u32::MAX, u32::MAX),
                    authored: Some((start, end)),
                    declaration: false,
                });
            }
        }
    }
    result
}
