//! Emit retained native JSX nodes inside plain expression children.

use super::{JsxEmit, JsxExpr, push_verbatim, render_sink_call};
use crate::virtual_ts::VizeMapping;
use vize_carton::String as CompactString;

pub(super) struct RenderContext<'a> {
    pub source: &'a str,
    pub roots: &'a [(u32, u32, Vec<JsxEmit>)],
}

impl RenderContext<'_> {
    /// Replace only JSX ranges supplied by the native OXC lowering. Every
    /// surrounding byte keeps the original callback, call and lexical scope.
    pub(super) fn rewrite_expression(
        &self,
        out: &mut CompactString,
        mappings: &mut Vec<VizeMapping>,
        expression: &JsxExpr,
    ) -> bool {
        // Native roots are sorted once by their authored AST start. Ordinary
        // sibling expressions inspect only their own range, never all roots.
        let first = self
            .roots
            .partition_point(|(start, _, _)| *start < expression.start);
        let last = self
            .roots
            .partition_point(|(start, _, _)| *start < expression.end);
        let Some(candidates) = self.roots.get(first..last) else {
            return false;
        };
        if candidates.is_empty() {
            return false;
        }
        let range = expression.start as usize..expression.end as usize;
        let Some(authored) = self.source.get(range.clone()) else {
            return false;
        };
        // Generated expressions use the existing mapping path. A nested rewrite
        // requires byte-exact custody of the authored expression at its AST span.
        if authored.trim() != expression.content.as_str() {
            return false;
        }
        let start = range.start + authored.len() - authored.trim_start().len();
        let end = range.end - (authored.len() - authored.trim_end().len());
        let mut cursor = start;
        let mut replaced = false;
        for (root_start, root_end, emits) in candidates {
            let root_start = *root_start as usize;
            let root_end = *root_end as usize;
            if root_start < cursor || root_end > end || root_start >= root_end {
                continue;
            }
            if root_start < start {
                continue;
            }
            push_verbatim(out, mappings, self.source, cursor, root_start);
            render_sink_call(out, mappings, emits, self);
            cursor = root_end;
            replaced = true;
        }
        if replaced {
            push_verbatim(out, mappings, self.source, cursor, end);
        }
        replaced
    }
}
