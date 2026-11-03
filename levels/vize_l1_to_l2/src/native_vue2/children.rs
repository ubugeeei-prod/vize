//! Walk the actual dialect children, including suffixes after typed refusals.

use super::{Context, Vue2HoleKind};
use vize_l0::Span;
use vize_l1::dialect::vue2::surface::RetainedTextChildren;
use vize_l1::markup::{SurfaceChild, entity::EntityContext};
use vize_l2::{artifact::RegionBuilder, expr::ExprRef};

impl<'a> Context<'_, 'a> {
    pub(super) fn children(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        children: RetainedTextChildren<'_, 'a>,
    ) {
        vize_l0::ensure_sufficient_stack(|| {
            for child in children {
                match child.surface() {
                    SurfaceChild::Element(element) => {
                        self.element(region, element, child.children());
                    }
                    SurfaceChild::Text(token) => {
                        let span = self.span(token.text);
                        if token
                            .text
                            .chars()
                            .any(|ch| matches!(ch, '\r' | '\n' | '\t' | '\u{000c}'))
                            || token.text.trim().is_empty()
                        {
                            // Keep the original run; default Vue 2 whitespace
                            // collapse/removal needs its own source-owned policy.
                            self.hole(span, Vue2HoleKind::WhitespacePolicy);
                        }
                        let decoded = crate::native::text::decode_references(
                            self.allocator,
                            token.text,
                            EntityContext::Text,
                        );
                        self.built(region.text(decoded, span), span);
                    }
                    SurfaceChild::Interpolation(node) => {
                        let span = Span::new(
                            self.span(node.open.text).start,
                            self.span(node.close.text).end,
                        );
                        let view = match self.original.text_for(child.reborrow()) {
                            Ok(view) => view,
                            Err(error) => {
                                self.hole(span, Vue2HoleKind::OriginalText(error));
                                continue;
                            }
                        };
                        if !view.chain().filters().is_empty() {
                            self.hole(span, Vue2HoleKind::Filter);
                            continue;
                        }
                        let Some(retained) = view.chain().base().retained() else {
                            self.hole(span, Vue2HoleKind::OriginalSurface);
                            continue;
                        };
                        match crate::native::retain_expression_in(
                            self.allocator,
                            self.original.block().root_source(),
                            retained,
                        ) {
                            Ok(expression) => self
                                .built(region.interpolation(ExprRef::Js(expression), span), span),
                            Err(error) => self.hole(span, Vue2HoleKind::Expression(error)),
                        }
                    }
                    SurfaceChild::Comment(token)
                    | SurfaceChild::Cdata(token)
                    | SurfaceChild::ProcessingInstruction(token)
                    | SurfaceChild::Unexpected(token) => {
                        self.hole(self.span(token.text), Vue2HoleKind::UnsupportedMarkup);
                    }
                }
            }
        });
    }
}
