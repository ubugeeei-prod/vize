//! An opt-in, versioned style edit with checked native SCSS equivalence and
//! a formatter fixed point on the complete prospective document.
use super::{SnapshotRefusal, SourceSnapshot};
use crate::{document::DocumentStore, utils::offset_to_position};
use tower_lsp::lsp_types::{
    OneOf, OptionalVersionedTextDocumentIdentifier, Range, TextDocumentEdit, TextEdit,
};
use vize_atelier_sfc::scss::{ScssMismatch, ScssProducts, ScssRefusal};
use vize_glyph::{FormatError, GlyphFormatter};
use vize_l0::{Allocator, Span, String};
use vize_l1::{
    container::vue::{DescriptorObservation, StyleView, Vue},
    css::lang::OriginalStyleStructure,
};

#[derive(Debug)]
pub enum StyleFormatError {
    Host(SnapshotRefusal),
    ForeignSource,
    Descriptor,
    Style(ScssRefusal),
    Formatting(FormatError),
    Header,
    ExternalStyle,
    Semantics(ScssMismatch),
    Unstable,
    Position,
}

/// Carries the selected original extent and the checked replacement on a
/// particular editor snapshot. A fresh document guard is required to publish.
pub struct VerifiedStyleFormat<'s> {
    snapshot: &'s SourceSnapshot,
    span: Span,
    replacement: Option<String>,
}

/// The selected original style, its compiler products and the owning editor
/// revision. The descriptor borrow keeps the original style identity private.
pub struct PreparedScss<'s, 'o, 'a> {
    snapshot: &'s SourceSnapshot,
    descriptor: &'o DescriptorObservation<'a>,
    ordinal: usize,
    original: StyleView<'o, 'a>,
    products: ScssProducts<'a>,
}

impl SourceSnapshot {
    pub fn verified_style_format<'s>(
        &'s self,
        documents: &DocumentStore,
        descriptor: &DescriptorObservation<'_>,
        ordinal: usize,
        formatter: &GlyphFormatter<'_>,
    ) -> Result<VerifiedStyleFormat<'s>, StyleFormatError> {
        let (prepared, ()) = self.prepare_scss_with(documents, descriptor, ordinal, |_| ())?;
        prepared.format(formatter)
    }

    /// Select one genuine style on this revision. Compiler output, class
    /// queries, editor definitions and a requested rule pass share its tree.
    pub fn prepare_scss_with<'s, 'o, 'a, R>(
        &'s self,
        documents: &DocumentStore,
        descriptor: &'o DescriptorObservation<'a>,
        ordinal: usize,
        visit: impl FnOnce(OriginalStyleStructure<'_, 'a>) -> R,
    ) -> Result<(PreparedScss<'s, 'o, 'a>, R), StyleFormatError> {
        self.check_current(documents)
            .map_err(StyleFormatError::Host)?;
        if !core::ptr::eq(descriptor.source(), self.source()) {
            return Err(StyleFormatError::ForeignSource);
        }
        let original = style_at(descriptor, ordinal)?;
        if original.attrs().iter().any(|attr| attr.name == "src") {
            return Err(StyleFormatError::ExternalStyle);
        }
        let (products, result) =
            ScssProducts::from_style_with(original, visit).map_err(StyleFormatError::Style)?;
        Ok((
            PreparedScss {
                snapshot: self,
                descriptor,
                ordinal,
                original,
                products,
            },
            result,
        ))
    }
}

impl<'s, 'a> PreparedScss<'s, '_, 'a> {
    pub fn products(&self) -> &ScssProducts<'a> {
        &self.products
    }

    /// Verify a format request against the retained compiler inputs. Reject a
    /// superseded editor revision before building the prospective document.
    pub fn verified_format(
        &self,
        documents: &DocumentStore,
        formatter: &GlyphFormatter<'_>,
    ) -> Result<VerifiedStyleFormat<'s>, StyleFormatError> {
        self.snapshot
            .check_current(documents)
            .map_err(StyleFormatError::Host)?;
        self.format(formatter)
    }

    fn format(
        &self,
        formatter: &GlyphFormatter<'_>,
    ) -> Result<VerifiedStyleFormat<'s>, StyleFormatError> {
        let original = self.original;
        let span = original.original_block().authored_span();
        let before = &self.products;
        before.program().map_err(StyleFormatError::Style)?;
        let formatted = formatter
            .format_observed_style(original)
            .map_err(StyleFormatError::Formatting)?;
        let source = self.snapshot.source();
        let keep = source
            .get(span.start as usize..span.end as usize)
            .ok_or(StyleFormatError::Position)?;
        let replacement = if keep == formatted.code.as_str() {
            before
                .check_equivalent(before)
                .map_err(StyleFormatError::Semantics)?;
            None
        } else {
            let prefix = source
                .get(..span.start as usize)
                .ok_or(StyleFormatError::Position)?;
            let suffix = source
                .get(span.end as usize..)
                .ok_or(StyleFormatError::Position)?;
            let mut candidate =
                String::with_capacity(prefix.len() + formatted.code.len() + suffix.len());
            candidate.push_str(prefix);
            candidate.push_str(formatted.code.as_str());
            let end = candidate.len();
            candidate.push_str(suffix);
            let arena = Allocator::default();
            let observed = Vue.observe_descriptor(&arena, &candidate, self.descriptor.options());
            let after = style_at(&observed, self.ordinal)?;
            let after_span = after.original_block().authored_span();
            if span.start != after_span.start
                || after_span.end as usize != end
                || !same_header(original, after)
            {
                return Err(StyleFormatError::Header);
            }
            let after_products =
                ScssProducts::from_style(after).map_err(StyleFormatError::Style)?;
            before
                .check_equivalent(&after_products)
                .map_err(StyleFormatError::Semantics)?;
            let again = formatter
                .format_observed_style(after)
                .map_err(StyleFormatError::Formatting)?;
            if again.code != formatted.code {
                return Err(StyleFormatError::Unstable);
            }
            Some(formatted.code)
        };
        Ok(VerifiedStyleFormat {
            snapshot: self.snapshot,
            span,
            replacement,
        })
    }
}

fn same_header(before: StyleView<'_, '_>, after: StyleView<'_, '_>) -> bool {
    let a = before.attrs();
    let b = after.attrs();
    if a.len() != b.len() {
        return false;
    }
    if a.iter()
        .map(|v| (v.name, v.value))
        .eq(b.iter().map(|v| (v.name, v.value)))
    {
        return true;
    }
    // The stock formatter can order plain SFC header flags. Duplicates and
    // dynamic or unfamiliar attributes retain their original ordering contract.
    a.iter().enumerate().all(|(i, attr)| {
        matches!(attr.name, "lang" | "scoped" | "module")
            && !a.iter().take(i).any(|prior| prior.name == attr.name)
            && b.iter()
                .any(|other| (other.name, other.value) == (attr.name, attr.value))
    })
}

fn style_at<'o, 'a>(
    descriptor: &'o DescriptorObservation<'a>,
    ordinal: usize,
) -> Result<StyleView<'o, 'a>, StyleFormatError> {
    descriptor
        .admitted()
        .map_err(|_| StyleFormatError::Descriptor)?
        .styles()
        .nth(ordinal)
        .ok_or(StyleFormatError::Descriptor)
}

impl VerifiedStyleFormat<'_> {
    /// Return the actual wire edit only while this snapshot still owns the
    /// open document. A byte-identical block produces no edit.
    pub fn publish(
        &self,
        documents: &DocumentStore,
    ) -> Result<Option<TextDocumentEdit>, StyleFormatError> {
        self.snapshot
            .with_current(documents, |document| {
                let Some(replacement) = &self.replacement else {
                    return Ok(None);
                };
                let text = document.position_text();
                let start = offset_to_position(text, self.span.start as usize)
                    .ok_or(StyleFormatError::Position)?;
                let end = offset_to_position(text, self.span.end as usize)
                    .ok_or(StyleFormatError::Position)?;
                Ok(Some(TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: self.snapshot.uri().clone(),
                        version: Some(self.snapshot.version()),
                    },
                    edits: vec![OneOf::Left(TextEdit {
                        range: Range::new(start, end),
                        new_text: replacement.as_str().into(),
                    })],
                }))
            })
            .map_err(StyleFormatError::Host)?
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod corpus;

#[cfg(all(test, feature = "experimental-style-lint-products"))]
mod composition;
