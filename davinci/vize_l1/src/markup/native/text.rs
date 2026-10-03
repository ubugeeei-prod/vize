//! Default Vue 3 whitespace at a genuinely selected original root text event.

use super::{NativeChild, NativeTemplateComponent, NativeTemplateGrammar};
use crate::SurfaceChild;
use vize_l0::{SourceBlock, Span, StringBuilder};

/// The sole policy this bounded source provider can establish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRootTextProfile {
    Vue3Condense,
}

/// Refusal leaves the original selected parser and every child unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRootTextError {
    ForeignComponent,
    NestedChild,
    NotText,
    RecoveredComponent,
    Entity,
    InvalidSourceFrame,
}

/// Movable original root provenance and its derived whitespace value.
/// No bare Component, caller source/span, raw child or policy can mint this.
/// Omitted source events remain present here and in the original tree.
///
/// ```compile_fail
/// use vize_l1::markup::NativeRootText;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeRootText<'static>>();
/// ```
/// ```compile_fail
/// use vize_l1::markup::NativeRootText;
/// fn forge() { let _ = NativeRootText { content: Some("invented") }; }
/// ```
#[derive(Debug)]
pub struct NativeRootText<'a> {
    origin: Origin<'a>,
    content: Option<&'a str>,
}

#[derive(Debug)]
struct Origin<'a> {
    block: SourceBlock<'a>,
    template_index: usize,
    grammar: NativeTemplateGrammar,
    child: *const SurfaceChild<'a>,
    ordinal: usize,
    raw_text: &'a str,
    span: Span,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Derive default Vue 3 condense from this same original root visit.
    /// Nested pre/RCDATA ancestry and text-entity decoding remain unavailable.
    pub fn prepare_condensed_root_text(
        &self,
        child: NativeChild<'_, 'a>,
    ) -> Result<NativeRootText<'a>, NativeRootTextError> {
        if !core::ptr::eq(child.component(), self.component()) {
            return Err(NativeRootTextError::ForeignComponent);
        }
        if child.parent_element().is_some() {
            return Err(NativeRootTextError::NestedChild);
        }
        let SurfaceChild::Text(token) = child.surface() else {
            return Err(NativeRootTextError::NotText);
        };
        let carrier = self.component().carrier();
        if !carrier.errors.is_empty()
            || !carrier.unsupported.is_empty()
            || token.is_missing()
            || token.text.is_empty()
        {
            return Err(NativeRootTextError::RecoveredComponent);
        }
        if token.text.contains('&') {
            return Err(NativeRootTextError::Entity);
        }
        let block = self.component().block();
        let span = block
            .span_of(token.text)
            .ok_or(NativeRootTextError::InvalidSourceFrame)?;
        let ordinal = child.ordinal();
        let previous = ordinal
            .checked_sub(1)
            .and_then(|index| self.component().root_child_at(index));
        let next = ordinal
            .checked_add(1)
            .and_then(|index| self.component().root_child_at(index));
        let content = if token.text.bytes().all(whitespace) {
            if omit_blank(token.text, previous, next) {
                None
            } else if token.text == " " {
                Some(token.text)
            } else {
                Some(" ")
            }
        } else {
            Some(condense(self, token.text))
        };
        Ok(NativeRootText {
            origin: Origin {
                block,
                template_index: self.template_index(),
                grammar: self.grammar(),
                child: child.surface(),
                ordinal,
                raw_text: token.text,
                span,
            },
            content,
        })
    }
}

fn whitespace(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}

fn omit_blank(
    raw: &str,
    previous: Option<&SurfaceChild<'_>>,
    next: Option<&SurfaceChild<'_>>,
) -> bool {
    let (Some(previous), Some(next)) = (previous, next) else {
        return true;
    };
    matches!(
        (previous, next),
        (
            SurfaceChild::Comment(_),
            SurfaceChild::Comment(_) | SurfaceChild::Element(_)
        ) | (SurfaceChild::Element(_), SurfaceChild::Comment(_))
    ) || (matches!(previous, SurfaceChild::Element(_))
        && matches!(next, SurfaceChild::Element(_))
        && raw.bytes().any(|byte| matches!(byte, b'\n' | b'\r')))
}

fn condense<'a>(selected: &NativeTemplateComponent<'a>, raw: &'a str) -> &'a str {
    let mut previous_whitespace = false;
    let changed = raw.bytes().any(|byte| {
        let current = whitespace(byte);
        let changed = current && (byte != b' ' || previous_whitespace);
        previous_whitespace = current;
        changed
    });
    if !changed {
        return raw;
    }
    let mut output = StringBuilder::with_capacity_in(raw.len(), selected.component().allocator());
    previous_whitespace = false;
    for character in raw.chars() {
        if matches!(character, '\t' | '\n' | '\x0c' | '\r' | ' ') {
            if !previous_whitespace {
                output.push(' ');
            }
            previous_whitespace = true;
        } else {
            output.push(character);
            previous_whitespace = false;
        }
    }
    output.into_str()
}

impl<'a> NativeRootText<'a> {
    #[must_use]
    pub const fn profile(&self) -> NativeRootTextProfile {
        NativeRootTextProfile::Vue3Condense
    }
    #[must_use]
    pub const fn grammar(&self) -> NativeTemplateGrammar {
        self.origin.grammar
    }
    #[must_use]
    pub const fn block(&self) -> SourceBlock<'a> {
        self.origin.block
    }
    #[must_use]
    pub const fn raw_text(&self) -> &'a str {
        self.origin.raw_text
    }
    #[must_use]
    pub const fn span(&self) -> Span {
        self.origin.span
    }
    #[must_use]
    pub const fn content(&self) -> Option<&'a str> {
        self.content
    }
    /// Constant-time short join at the receiver's actual original root cursor.
    /// Same-byte reparses and different original slots cannot join.
    pub fn admitted_for_root_at<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        ordinal: usize,
    ) -> Option<NativeRootTextView<'s, 'a>> {
        let child = selected.component().root_child_at(ordinal)?;
        let block = selected.component().block();
        if ordinal != self.origin.ordinal
            || !core::ptr::eq(self.origin.child, child)
            || selected.template_index() != self.origin.template_index
            || selected.grammar() != self.origin.grammar
            || block.start() != self.origin.block.start()
            || !core::ptr::eq(block.root_source(), self.origin.block.root_source())
            || !core::ptr::eq(block.source(), self.origin.block.source())
        {
            return None;
        }
        Some(NativeRootTextView {
            owner: self,
            selected,
        })
    }
}

/// Readonly same-original-root join; this is not whole-body/File completion.
pub struct NativeRootTextView<'s, 'a> {
    owner: &'s NativeRootText<'a>,
    selected: &'s NativeTemplateComponent<'a>,
}
impl<'s, 'a> NativeRootTextView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn receipt(&self) -> &'s NativeRootText<'a> {
        self.owner
    }
}

#[cfg(test)]
mod tests;
