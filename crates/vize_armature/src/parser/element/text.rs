//! Text, fostered text, and interpolation processing.

use vize_carton::Box;
use vize_relief::*;

use super::super::Parser;

impl<'a> Parser<'a> {
    /// Process text content
    pub(in crate::parser) fn on_text_impl(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let source = self.get_source(start, end).to_owned();
        self.append_or_merge_text(&source, start, end);
    }

    /// Process text entity content
    pub(in crate::parser) fn on_text_entity_impl(&mut self, ch: char, start: usize, end: usize) {
        let mut content = [0_u8; 4];
        self.append_or_merge_text(ch.encode_utf8(&mut content), start, end);
    }

    /// Append or merge text node
    fn append_or_merge_text(&mut self, content: &str, start: usize, end: usize) {
        if self.should_foster_text(content) {
            self.append_or_merge_fostered_text(content, start, end);
            return;
        }

        let merge_start_off = if let Some(entry) = self.stack.last() {
            match entry.element.children.last() {
                Some(TemplateChildNode::Text(t)) => Some(t.loc.start.offset as usize),
                _ => None,
            }
        } else {
            match self.root.as_ref().and_then(|root| root.children.last()) {
                Some(TemplateChildNode::Text(t)) => Some(t.loc.start.offset as usize),
                _ => None,
            }
        };

        if let Some(merge_start) = merge_start_off {
            let end_pos = self.get_pos(end);
            let source_span = self.get_source(merge_start, end).into();
            if let Some(entry) = self.stack.last_mut()
                && let Some(TemplateChildNode::Text(text_node)) = entry.element.children.last_mut()
            {
                text_node.content.push_str(content);
                text_node.loc.end = end_pos;
                text_node.loc.source = source_span;
            } else if let Some(root) = self.root.as_mut()
                && let Some(TemplateChildNode::Text(text_node)) = root.children.last_mut()
            {
                text_node.content.push_str(content);
                text_node.loc.end = end_pos;
                text_node.loc.source = source_span;
            }
        } else {
            let loc = self.create_loc(start, end);
            let text_node = TextNode::new(content, loc);
            let boxed = Box::new_in(text_node, self.allocator);
            self.add_child(TemplateChildNode::Text(boxed));
        }
    }

    fn append_or_merge_fostered_text(&mut self, content: &str, start: usize, end: usize) {
        let Some(table_index) = self.nearest_table_index() else {
            self.append_or_merge_text(content, start, end);
            return;
        };

        let merge_start_off = match self.stack[table_index].fostered_before.last() {
            Some(TemplateChildNode::Text(t)) => Some(t.loc.start.offset as usize),
            _ => None,
        };

        if let Some(merge_start) = merge_start_off {
            let end_pos = self.get_pos(end);
            let source_span = self.get_source(merge_start, end).into();
            if let Some(TemplateChildNode::Text(text_node)) =
                self.stack[table_index].fostered_before.last_mut()
            {
                text_node.content.push_str(content);
                text_node.loc.end = end_pos;
                text_node.loc.source = source_span;
            }
        } else {
            let loc = self.create_loc(start, end);
            let text_node = TextNode::new(content, loc);
            let boxed = Box::new_in(text_node, self.allocator);
            self.stack[table_index]
                .fostered_before
                .push(TemplateChildNode::Text(boxed));
        }
    }

    /// Process interpolation
    pub(in crate::parser) fn on_interpolation_impl(&mut self, start: usize, end: usize) {
        self.build_interpolation(start, end, false);
    }

    /// Process a Vue 1.x raw-HTML interpolation (`{{{ expr }}}`), the pre-Vue-2
    /// `v-html` equivalent. Only reached behind the `legacy` feature with a
    /// Vue 1.x dialect; the resulting node is flagged `raw` so codegen emits the
    /// expression unescaped instead of through `_toDisplayString`.
    ///
    /// `start`/`end` already span the trimmed-by-delimiter expression (the
    /// tokenizer strips the extra `{` / `}`), so the only difference from a plain
    /// interpolation is the triple-mustache delimiter width used for the node's
    /// outer source location.
    #[cfg(feature = "legacy")]
    pub(in crate::parser) fn on_raw_interpolation_impl(&mut self, start: usize, end: usize) {
        self.build_interpolation(start, end, true);
    }

    fn build_interpolation(&mut self, start: usize, end: usize, raw: bool) {
        let raw_content = self.get_source(start, end);
        let content = raw_content.trim();

        // Calculate trimmed positions for accurate source mapping
        let leading_ws = raw_content.len() - raw_content.trim_start().len();
        let trimmed_start = start + leading_ws;
        let trimmed_end = trimmed_start + content.len();

        // Raw `{{{ … }}}` interpolation uses three-byte delimiters; a plain
        // `{{ … }}` uses the configured (default two-byte) delimiters. `raw` is
        // only ever true behind the `legacy` feature.
        let (open_len, close_len) = if raw {
            (3, 3)
        } else {
            (
                self.options.delimiters.0.len(),
                self.options.delimiters.1.len(),
            )
        };
        let full_start = start - open_len;
        let full_end = end + close_len;
        let loc = self.create_loc(full_start, full_end);
        let inner_loc = self.create_loc(trimmed_start, trimmed_end);

        // Create expression node
        let expr = SimpleExpressionNode::new(content, false, inner_loc);
        let expr_boxed = Box::new_in(expr, self.allocator);

        let interp = InterpolationNode {
            content: ExpressionNode::Simple(expr_boxed),
            loc,
            // `vize_armature/legacy` forwards to `vize_relief/_legacy`, so the
            // `raw` field exists exactly when this feature is enabled.
            #[cfg(feature = "legacy")]
            raw,
        };
        let boxed = Box::new_in(interp, self.allocator);
        self.add_child(TemplateChildNode::Interpolation(boxed));
    }
}
