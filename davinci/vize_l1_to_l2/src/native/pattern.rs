//! A const native dialect pattern table consumed during owner construction.

use super::{Context, NativeHoleKind};
use vize_l0::Span;
use vize_l1::markup::directive::DirectiveName;
use vize_l2::artifact::ComponentFactory;

mod vue3;

#[derive(Clone, Copy)]
pub(super) struct Directive {
    pub ordinal: usize,
    pub head: DirectiveName,
    pub name_span: Span,
    pub span: Span,
    pub value: Option<Span>,
    pub missing: bool,
}

struct Pattern<'a, R> {
    accepts: fn(&str, DirectiveName) -> bool,
    lower: fn(&mut Context<'a>, &mut R, Directive),
}

impl<'a> Context<'a> {
    pub(super) fn directive<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
        directive: Directive,
    ) {
        for pattern in vue3::Patterns::<'a, R>::TABLE {
            if (pattern.accepts)(self.block.root_source(), directive.head) {
                (pattern.lower)(self, region, directive);
                return;
            }
        }
        self.hole(region, NativeHoleKind::Directive, directive.span);
    }
}

#[cfg(test)]
mod tests;
