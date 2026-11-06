//! Text insertion modes for the explicit document envelope.

use vize_l0::Span;

use super::{Builder, Mode, Refusal};
use crate::markup::{Namespace, Profile};

impl<P: Profile> Builder<'_, '_, P> {
    /// Text belongs to body content and cells; table text may only be
    /// whitespace, which the browser keeps in place instead of fostering it.
    pub(super) fn text(&mut self, span: Span, space: bool) -> Result<(), Refusal> {
        // The shipped parser retains a literal NULL while Chromium replaces it
        // in special-text states. Do not manufacture agreement between them.
        if self.current_namespace() == Some(Namespace::Html)
            && matches!(
                self.current(),
                Some("title" | "style" | "script" | "textarea")
            )
            && self.source(span)?.contains('\0')
        {
            return Err(Refusal::NonHtmlToken);
        }
        // The retained lexer deliberately treats every `title` as RCDATA. In
        // SVG it is an HTML integration point instead, so markup hidden inside
        // one text callback cannot be certified as the browser's child tree.
        if self.current_namespace() == Some(Namespace::Svg)
            && self.current() == Some("title")
            && self.source(span)?.contains('<')
        {
            return Err(Refusal::NonHtmlToken);
        }
        match self.mode {
            Mode::InBody | Mode::InCell | Mode::InCaption => Ok(()),
            Mode::InHead if self.in_head_text() => Ok(()),
            Mode::InTable | Mode::InTableBody | Mode::InRow | Mode::InColumnGroup if !space => {
                Err(Refusal::TableText(span))
            }
            Mode::InTable | Mode::InTableBody | Mode::InRow | Mode::InColumnGroup => Ok(()),
            _ if !space => Err(Refusal::ExplicitEnvelope),
            Mode::HtmlEnd | Mode::End => {
                if !span.is_empty() {
                    self.body_tail_text.get_or_insert(span);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
