//! Folding regions derived from authored script AST spans.

use oxc_allocator::Allocator;
use oxc_ast::ast::{FunctionBody, ObjectExpression};
use oxc_ast_visit::{
    Visit,
    walk::{walk_function_body, walk_object_expression},
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use tower_lsp::lsp_types::FoldingRange;
use vize_atelier_sfc::SfcScriptBlock;

use super::region;

pub(super) fn script_regions(block: &SfcScriptBlock<'_>) -> Vec<FoldingRange> {
    if block.src.is_some() || block.content.is_empty() {
        return Vec::new();
    }

    let source = block.content.as_ref();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type(block.lang.as_deref())).parse();
    let mut collector = ScriptRegionCollector {
        source: source.as_bytes(),
        spans: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    collector.spans.sort_by_key(|span| (span.start, span.end));
    collector.spans.dedup();

    let base_line = block.loc.start_line.saturating_sub(1) as u32;
    let newline_offsets = source
        .match_indices('\n')
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    collector
        .spans
        .into_iter()
        .filter_map(|span| {
            let start = span.start as usize;
            let close = span.end.saturating_sub(1) as usize;
            let open_line = base_line + line_offset(&newline_offsets, start);
            let close_line = base_line + line_offset(&newline_offsets, close);
            region(open_line, close_line, None, None)
        })
        .collect()
}

fn line_offset(newlines: &[usize], byte_offset: usize) -> u32 {
    newlines.partition_point(|newline| *newline < byte_offset) as u32
}

fn source_type(lang: Option<&str>) -> SourceType {
    match lang {
        Some("ts") => SourceType::ts().with_module(true),
        Some("tsx") => SourceType::tsx().with_module(true),
        Some("jsx") => SourceType::jsx().with_module(true),
        _ => SourceType::mjs(),
    }
}

struct ScriptRegionCollector<'s> {
    source: &'s [u8],
    spans: Vec<Span>,
}

impl ScriptRegionCollector<'_> {
    fn push_braced(&mut self, span: Span) {
        let start = span.start as usize;
        let Some(close) = (span.end as usize).checked_sub(1) else {
            return;
        };
        if self.source.get(start) == Some(&b'{') && self.source.get(close) == Some(&b'}') {
            self.spans.push(span);
        }
    }
}

impl<'a> Visit<'a> for ScriptRegionCollector<'_> {
    fn visit_object_expression(&mut self, expression: &ObjectExpression<'a>) {
        self.push_braced(expression.span);
        walk_object_expression(self, expression);
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.push_braced(body.span);
        walk_function_body(self, body);
    }
}
