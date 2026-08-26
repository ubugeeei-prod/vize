//! Comment and CDATA processing.

use vize_relief::{CommentNode, ErrorCode, Namespace, TemplateChildNode};
use vize_s0::{Box, directive::parse_vize_directive};

use super::super::Parser;

impl<'a> Parser<'a> {
    /// Process comment
    pub(in crate::parser) fn on_comment_impl(&mut self, start: usize, end: usize) {
        let content = self.get_source_retained(start, end);
        let loc_start = start.saturating_sub(4);
        let loc_end = self.comment_loc_end(start, end);
        let loc = self.create_loc(loc_start, loc_end); // Include <!-- and --> when present.

        // Check for @vize: directive. Only `kind` survives below, so the
        // line/offset bookkeeping the parsed directive carries is discarded;
        // pass the constant line the retired parser tracking always reported
        // here. Consumers that need the real line (patina's visitor) derive it
        // from the offset at their edge.
        let directive = parse_vize_directive(content, 1, loc.span.start);

        // Always preserve directive comments (even when options.comments = false)
        // so they can be explicitly handled by codegen and linter
        if directive.is_none() && !self.options.comments {
            return;
        }

        let mut comment = CommentNode::new(content, loc);
        comment.directive = directive.map(|d| d.kind);
        let boxed = Box::new_in(comment, &self.allocator);
        self.add_child(TemplateChildNode::Comment(boxed));
    }

    /// Process experimental in-tag `//` comments.
    pub(in crate::parser) fn on_in_tag_comment_impl(&mut self, start: usize, end: usize) {
        let content_start = start.saturating_add(2).min(end);
        let comment = CommentNode::new_in_tag(
            self.get_source_retained(content_start, end),
            self.create_loc(start, end),
        );
        if let Some(root) = self.root.as_mut() {
            root.comments.push(comment);
        }
    }

    fn comment_loc_end(&self, start: usize, end: usize) -> usize {
        let end = self.clamp_to_char_boundary(end);
        let rest = &self.source[end..];
        if rest.starts_with("-->") {
            end + 3
        } else if start == end && rest.starts_with("->") {
            end + 2
        } else if start == end && rest.starts_with('>') {
            end + 1
        } else {
            end.saturating_add(3).min(self.source.len())
        }
    }

    /// Process CDATA
    pub(in crate::parser) fn on_cdata_impl(&mut self, start: usize, end: usize) {
        let is_html_ns = self
            .stack
            .last()
            .map(|e| e.element.ns)
            .unwrap_or(Namespace::Html)
            == Namespace::Html;
        if is_html_ns {
            self.on_error_impl(ErrorCode::CdataInHtmlContent, start.saturating_sub(9));
        } else {
            self.on_text_impl(start, end);
        }
    }
}
