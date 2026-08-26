//! Text, fostered text, and interpolation processing.

use vize_relief::{
    ExpressionNode, InterpolationNode, SimpleExpressionNode, TemplateChildNode, TextNode,
};
use vize_s0::{Box, String};

use super::super::{Parser, PendingText, TextSlot};

impl<'a> Parser<'a> {
    /// Process text content
    pub(in crate::parser) fn on_text_impl(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }

        let content = self.get_source_retained(start, end);
        self.append_or_merge_text(content, start, end);
    }

    /// Process text entity content
    pub(in crate::parser) fn on_text_entity_impl(&mut self, ch: char, start: usize, end: usize) {
        let mut buf = [0_u8; 4];
        let decoded: &'a str = self.allocator.alloc_str(ch.encode_utf8(&mut buf));
        self.append_or_merge_text(decoded, start, end);
    }

    /// Append or merge text node
    fn append_or_merge_text(&mut self, content: &'a str, start: usize, end: usize) {
        if self.should_foster_text(content) {
            self.append_or_merge_fostered_text(content, start, end);
            return;
        }

        let slot = self.current_text_slot();
        let can_merge = if let Some(entry) = self.stack.last() {
            matches!(
                entry.element.children.last(),
                Some(TemplateChildNode::Text(_))
            )
        } else {
            matches!(
                self.root.as_ref().and_then(|root| root.children.last()),
                Some(TemplateChildNode::Text(_))
            )
        };

        if !can_merge {
            let loc = self.create_loc(start, end);
            let text_node = TextNode::new(content, loc);
            let boxed = Box::new_in(text_node, &self.allocator);
            self.add_child(TemplateChildNode::Text(boxed));
            return;
        }

        self.merge_into_pending(slot, content);
        if let Some(entry) = self.stack.last_mut()
            && let Some(TemplateChildNode::Text(text_node)) = entry.element.children.last_mut()
        {
            text_node.loc.set_end(end as u32);
        } else if let Some(root) = self.root.as_mut()
            && let Some(TemplateChildNode::Text(text_node)) = root.children.last_mut()
        {
            text_node.loc.set_end(end as u32);
        }
    }

    fn append_or_merge_fostered_text(&mut self, content: &'a str, start: usize, end: usize) {
        let Some(table_index) = self.nearest_table_index() else {
            self.append_or_merge_text(content, start, end);
            return;
        };

        let can_merge = matches!(
            self.stack[table_index].fostered_before.last(),
            Some(TemplateChildNode::Text(_))
        );

        if !can_merge {
            let loc = self.create_loc(start, end);
            let text_node = TextNode::new(content, loc);
            let boxed = Box::new_in(text_node, &self.allocator);
            self.flush_pending_text();
            self.stack[table_index]
                .fostered_before
                .push(TemplateChildNode::Text(boxed));
            return;
        }

        self.merge_into_pending(TextSlot::Fostered(table_index), content);
        if let Some(TemplateChildNode::Text(text_node)) =
            self.stack[table_index].fostered_before.last_mut()
        {
            text_node.loc.set_end(end as u32);
        }
    }

    /// Accumulate `chunk` into the buffered run for `slot`, seeding the buffer
    /// from the node's current content the first time a run needs one.
    ///
    /// The node's `content` is deliberately left stale until
    /// [`Parser::flush_pending_text`] runs: that keeps a run of N entities
    /// linear instead of recopying the whole run once per entity, and the
    /// flush at every non-text callback boundary means nothing can read it in
    /// between.
    fn merge_into_pending(&mut self, slot: TextSlot, chunk: &str) {
        if let Some(pending) = self.pending_text.as_mut()
            && pending.slot == slot
        {
            pending.buf.push_str(chunk);
            return;
        }

        self.flush_pending_text();

        let existing = match slot {
            TextSlot::Stack(index) => self
                .stack
                .get(index)
                .and_then(|entry| entry.element.children.last()),
            TextSlot::Fostered(index) => self
                .stack
                .get(index)
                .and_then(|entry| entry.fostered_before.last()),
            TextSlot::Root => self.root.as_ref().and_then(|root| root.children.last()),
        };
        let Some(TemplateChildNode::Text(node)) = existing else {
            return;
        };
        let mut buf = String::with_capacity(node.content.len() + chunk.len());
        buf.push_str(node.content);
        buf.push_str(chunk);
        self.pending_text = Some(PendingText {
            buf,
            slot,
            start: node.loc.span.start as usize,
        });
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
        let raw_content = self.get_source_retained(start, end);
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
        let mut expr = SimpleExpressionNode::new(content, false, inner_loc);
        self.retain_expression_ast(&mut expr, trimmed_start, trimmed_end);
        let expr_boxed = Box::new_in(expr, &self.allocator);

        let interp = InterpolationNode {
            content: ExpressionNode::Simple(expr_boxed),
            loc,
            // `vize_armature/legacy` forwards to `vize_relief/_legacy`, so the
            // `raw` field exists exactly when this feature is enabled.
            #[cfg(feature = "legacy")]
            raw,
        };
        let boxed = Box::new_in(interp, &self.allocator);
        self.add_child(TemplateChildNode::Interpolation(boxed));
    }
}
