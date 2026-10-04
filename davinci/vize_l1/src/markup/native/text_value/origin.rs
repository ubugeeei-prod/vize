use super::{NativeChild, NativeTemplateComponent, NativeTextValueError as Kind};
use crate::{Element, ElementClose, SurfaceChild, Token, markup::NativeTemplateGrammar};
use vize_l0::{SourceBlock, Span};

/// Only original immutable arena addresses survive wrapper movement.
pub(super) struct Origin<'a> {
    pub(super) block: SourceBlock<'a>,
    pub(super) span: Span,
    child: *const SurfaceChild<'a>,
    parent: Option<*const Element<'a>>,
    ordinal: usize,
    template_index: usize,
    grammar: NativeTemplateGrammar,
}

impl<'a> Origin<'a> {
    pub(super) fn original(
        selected: &NativeTemplateComponent<'a>,
        child: &NativeChild<'_, 'a>,
    ) -> Result<(Self, Token<'a>), Kind> {
        if !core::ptr::eq(child.component(), selected.component()) {
            return Err(Kind::ForeignComponent);
        }
        let SurfaceChild::Text(token) = child.surface() else {
            return Err(Kind::NotText);
        };
        let carrier = selected.component().carrier();
        if !carrier.errors.is_empty()
            || !carrier.unsupported.is_empty()
            || token.is_missing()
            || token.text.is_empty()
        {
            return Err(Kind::RecoveredComponent);
        }
        if let Some(parent) = child.parent_element() {
            if parent.open.lt_name.is_missing()
                || parent.open.gt.is_missing()
                || parent
                    .open
                    .slash
                    .as_ref()
                    .is_some_and(|slash| slash.is_missing())
                || match &parent.close {
                    ElementClose::Missing => true,
                    ElementClose::Present(close) => {
                        close.lt_slash_name.is_missing() || close.gt.is_missing()
                    }
                    ElementClose::Implicit | ElementClose::NotExpected => false,
                }
            {
                return Err(Kind::RecoveredComponent);
            }
            if parent.open.is_verbatim() {
                return Err(Kind::Verbatim);
            }
            // These roles use the lexer's actual case-insensitive special
            // parent family. No source search or ancestry traversal occurs.
            let tag = parent.tag();
            if tag.eq_ignore_ascii_case("script") || tag.eq_ignore_ascii_case("style") {
                return Err(Kind::RawTextParent);
            }
            if tag.eq_ignore_ascii_case("title") || tag.eq_ignore_ascii_case("textarea") {
                return Err(Kind::RcDataParent);
            }
        }
        let block = selected.component().block();
        let span = block
            .span_of(token.text)
            .ok_or(Kind::Source(crate::embed::SourceError::InvalidAuthoredSpan))?;
        Ok((
            Self {
                block,
                span,
                child: child.surface(),
                parent: child.parent_element().map(core::ptr::from_ref),
                ordinal: child.ordinal(),
                template_index: selected.template_index(),
                grammar: selected.grammar(),
            },
            *token,
        ))
    }

    pub(super) fn matches(
        &self,
        selected: &NativeTemplateComponent<'a>,
        child: &NativeChild<'_, 'a>,
        original: &Token<'a>,
    ) -> bool {
        let SurfaceChild::Text(token) = child.surface() else {
            return false;
        };
        let block = selected.component().block();
        core::ptr::eq(child.component(), selected.component())
            && core::ptr::eq(self.child, child.surface())
            && self.parent == child.parent_element().map(core::ptr::from_ref)
            && self.ordinal == child.ordinal()
            && self.template_index == selected.template_index()
            && self.grammar == selected.grammar()
            && self.block.start() == block.start()
            && core::ptr::eq(self.block.root_source(), block.root_source())
            && core::ptr::eq(self.block.source(), block.source())
            && core::ptr::eq(original.text, token.text)
            && core::ptr::eq(original.leading, token.leading)
            && original.status == token.status
            && block.span_of(token.text) == Some(self.span)
    }
}
