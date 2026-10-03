//! Bounded HTML element ancestry from the sole original Document event tape.
//! No Component surface, external event vectors, reparse or DOM payloads enter.

use vize_l0::{Span, Vec};

use super::{DocumentLexicalRefusal, DocumentTreePolicy, NativeDocument};

mod build;

#[derive(Debug)]
struct Element {
    name: &'static str,
    authored_name: Span,
    opening: Span,
    closing: Option<Span>,
    parent: Option<usize>,
    next_sibling: Option<usize>,
    first_child: Option<usize>,
    last_child: Option<usize>,
    ignored_slash: bool,
}

/// Why no bounded native element structure was produced for the original run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentHtmlRefusal {
    Lexical(DocumentLexicalRefusal),
    ExplicitEnvelope,
    UnsupportedElement(Span),
    UnsupportedPolicy(DocumentTreePolicy),
    VueSyntax,
    NonHtmlToken,
    InvalidFrame(Span),
    ImpliedEnd(Span),
    PendingStructure,
}

/// Actual element ancestry for a strict explicit no-quirks HTML envelope.
///
/// Supported envelope: exactly `<!DOCTYPE html>`, `html`, `head`, then `body`,
/// with explicit matching ends. Head children are `base`, `link` and `meta`;
/// body elements are `div`, `span`, `br`, `hr`, `img` and `input`. HTML names
/// match without ASCII case, and a slash never closes a nonvoid HTML element.
/// Static attributes retain original opening bytes but get no DOM meaning.
///
/// Implied ends, tables, foreign content, raw-text elements, formatting,
/// templates, Vue directive/interpolation syntax and implicit envelopes refuse
/// this provider. Only element ancestry is certified; text, comments, decoded
/// attributes, petite-vue semantics and all general profile policies remain
/// unfinished. This view borrows the actual original lexer owner and arena.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlStructure;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentHtmlStructure<'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlStructure;
/// fn forge<'o, 'a>(original: DocumentHtmlStructure<'o, 'a>) -> DocumentHtmlStructure<'o, 'a> {
///     DocumentHtmlStructure { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct DocumentHtmlStructure<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    elements: Vec<'a, Element>,
}

impl<'o, 'a> DocumentHtmlStructure<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.owner
    }

    /// Elements in original start-tag order; each has its actual HTML parent.
    #[must_use]
    pub fn elements(&self) -> impl ExactSizeIterator<Item = DocumentHtmlElement<'_, 'o, 'a>> {
        (0..self.elements.len()).map(|index| DocumentHtmlElement { tree: self, index })
    }
}

/// A readonly element borrowing its genuine source-owned structure.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlElement;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentHtmlElement<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlElement;
/// fn forge<'s, 'o, 'a>(original: DocumentHtmlElement<'s, 'o, 'a>) -> DocumentHtmlElement<'s, 'o, 'a> {
///     DocumentHtmlElement { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct DocumentHtmlElement<'s, 'o, 'a> {
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    index: usize,
}

impl<'s, 'o, 'a> DocumentHtmlElement<'s, 'o, 'a> {
    fn element(&self) -> &Element {
        &self.tree.elements[self.index]
    }

    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.tree.owner
    }

    /// The actual HTML local name, folded by native construction.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.element().name
    }

    #[must_use]
    pub fn authored_name(&self) -> &'a str {
        self.element().authored_name.slice(self.owner().source())
    }

    #[must_use]
    pub fn opening_span(&self) -> Span {
        self.element().opening
    }

    #[must_use]
    pub fn opening(&self) -> &'a str {
        self.opening_span().slice(self.owner().source())
    }

    /// Void elements have no closing tag; nonvoid admitted elements always do.
    #[must_use]
    pub fn closing_span(&self) -> Option<Span> {
        self.element().closing
    }

    #[must_use]
    pub fn ignored_self_closing_slash(&self) -> bool {
        self.element().ignored_slash
    }

    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.element().parent.map(|index| Self {
            tree: self.tree,
            index,
        })
    }

    /// Direct element children, following the producer's original adjacency.
    pub fn children(&self) -> impl Iterator<Item = Self> + 's {
        let tree = self.tree;
        let mut next = self.element().first_child;
        core::iter::from_fn(move || {
            let index = next?;
            next = tree.elements[index].next_sibling;
            Some(Self { tree, index })
        })
    }
}

pub(super) fn construct<'o, 'a>(
    owner: &'o NativeDocument<'a>,
) -> Result<DocumentHtmlStructure<'o, 'a>, DocumentHtmlRefusal> {
    build::construct(owner)
}

#[cfg(test)]
mod tests;
