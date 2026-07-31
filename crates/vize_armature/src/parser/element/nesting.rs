//! The element-nesting depth guard and its end-tag recovery.
//!
//! Elements nested deeper than [`MAX_ELEMENT_NESTING_DEPTH`] are attached to the
//! tree as leaves instead of being pushed onto the open-element stack, so the
//! AST the recursive later passes (transform, codegen, semantic analysis) walk
//! stays within a predictable amount of stack space regardless of the input.
//!
//! That recovery used to leak into the diagnostics. Because the over-limit
//! elements never reached the stack, their end tags found nothing to close and
//! `on_close_tag` reported `InvalidEndTag` for each of them — a false positive
//! pointing at end tags that are correct in the source, growing as
//! `2 * (depth - MAX_ELEMENT_NESTING_DEPTH)`. The parser therefore keeps the tags
//! of the flattened elements here so their end tags can be consumed silently,
//! and reports the limit once per over-limit region instead of once per element.

use vize_relief::{
    ElementNode,
    errors::{CompilerError, ErrorCode},
};

use super::super::Parser;

/// Maximum element nesting depth retained by the parser.
///
/// The limit is far beyond any realistic template while still cheap to enforce.
pub(super) const MAX_ELEMENT_NESTING_DEPTH: usize = 256;

/// Message attached to the recoverable error raised when the nesting limit is hit.
const NESTING_TOO_DEEP_MESSAGE: &str = "Element nesting is too deep.";

impl<'a> Parser<'a> {
    /// Record an element the nesting limit refused to descend into.
    ///
    /// The diagnostic is raised once per contiguous over-limit region: entering
    /// the region is the fact worth reporting, and repeating it for every
    /// element below it only buries the rest of the diagnostics.
    pub(super) fn record_flattened_element(&mut self, element: &ElementNode<'a>) {
        if self.flattened_tags.is_empty() {
            self.errors.push(CompilerError::with_message(
                ErrorCode::ExtendPoint,
                NESTING_TOO_DEEP_MESSAGE,
                Some(element.loc.clone()),
            ));
        }
        self.flattened_tags.push(element.tag.clone());
    }

    /// Consume the end tag of a flattened element, if this tag closes one.
    ///
    /// Flattened elements are always inner to everything on the open-element
    /// stack, so they are matched first, and an intervening unmatched tag is
    /// dropped exactly as the real stack would drop it.
    pub(super) fn close_flattened_element(&mut self, tag: &str) -> bool {
        let Some(index) = (0..self.flattened_tags.len())
            .rev()
            .find(|&i| self.flattened_tags[i].eq_ignore_ascii_case(tag))
        else {
            return false;
        };
        self.flattened_tags.truncate(index);
        true
    }

    /// Forget every flattened element.
    ///
    /// Called whenever an entry is closed on the open-element stack: flattened
    /// elements only exist below a full stack, so closing any stack entry closes
    /// all of them too.
    pub(super) fn clear_flattened_elements(&mut self) {
        self.flattened_tags.clear();
    }
}
