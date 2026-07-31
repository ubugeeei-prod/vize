//! Folding regions for the bodies inside an authored `<script>` block.

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

/// One region per multi-line object literal and function body, in document
/// order. The AST traversal and line scan together take O(n + r).
pub(super) fn script_regions(
    block: &SfcScriptBlock<'_>,
    content_start_line: u32,
) -> Vec<FoldingRange> {
    collect_script_regions(block, content_start_line).0
}

fn collect_script_regions(
    block: &SfcScriptBlock<'_>,
    content_start_line: u32,
) -> (Vec<FoldingRange>, usize) {
    if block.src.is_some() || block.content.is_empty() {
        return (Vec::new(), 0);
    }

    let source = block.content.as_ref();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type(block.lang.as_deref())).parse();
    let mut collector = ScriptRegionCollector {
        source: source.as_bytes(),
        cursor: 0,
        line: content_start_line,
        ranges: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    let work = collector.cursor + collector.ranges.len();
    (collector.ranges.into_iter().flatten().collect(), work)
}

#[cfg(test)]
pub(super) fn script_regions_with_metrics(
    block: &SfcScriptBlock<'_>,
    content_start_line: u32,
) -> (Vec<FoldingRange>, usize) {
    collect_script_regions(block, content_start_line)
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
    cursor: usize,
    line: u32,
    ranges: Vec<Option<FoldingRange>>,
}

impl ScriptRegionCollector<'_> {
    fn begin_braced(&mut self, span: Span) -> Option<PendingRegion> {
        let start = span.start as usize;
        let close = (span.end as usize).checked_sub(1)?;
        if self.source.get(start) != Some(&b'{') || self.source.get(close) != Some(&b'}') {
            return None;
        }

        let open_line = self.advance_to(start);
        let index = self.ranges.len();
        self.ranges.push(None);
        Some(PendingRegion {
            index,
            open_line,
            close,
        })
    }

    fn end_braced(&mut self, pending: PendingRegion) {
        let close_line = self.advance_to(pending.close);
        self.ranges[pending.index] = region(pending.open_line, close_line, None, None);
    }

    /// AST enter/exit events follow authored source order, so this cursor only
    /// advances and scans all intervening bytes at most once.
    fn advance_to(&mut self, byte_offset: usize) -> u32 {
        debug_assert!(
            byte_offset >= self.cursor,
            "script AST traversal must follow authored source order"
        );
        if byte_offset < self.cursor {
            return self.line;
        }
        self.line += self.source[self.cursor..byte_offset]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count() as u32;
        self.cursor = byte_offset;
        self.line
    }
}

struct PendingRegion {
    index: usize,
    open_line: u32,
    close: usize,
}

impl<'a> Visit<'a> for ScriptRegionCollector<'_> {
    fn visit_object_expression(&mut self, expression: &ObjectExpression<'a>) {
        let pending = self.begin_braced(expression.span);
        walk_object_expression(self, expression);
        if let Some(pending) = pending {
            self.end_braced(pending);
        }
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        let pending = self.begin_braced(body.span);
        walk_function_body(self, body);
        if let Some(pending) = pending {
            self.end_braced(pending);
        }
    }
}
