//! One formatting result for an L1 Program and one borrowed option set.
//! Standalone script and SFC-body consumers read the same stabilized bytes.

use super::{block_identity::retained_empty_body, format_script_content_stable};
use crate::{FormatError, FormatOptions, ImportSortOptions};
use core::cell::OnceCell;
use vize_l0::{Allocator, String};
use vize_l1::embed::{Shape, syntax::EmbedSyntax};

/// A request-local script formatter. Construction borrows its inputs. The
/// first read runs Glyph with the source type and exact text of the L1 Program.
/// Subsequent reads share the complete bytes or the same diagnostic object.
pub struct ScriptFormatQuery<'q, 'a> {
    arena: &'q Allocator,
    program: &'q EmbedSyntax<'a>,
    options: &'q FormatOptions,
    sorting: Option<&'q ImportSortOptions>,
    output: OnceCell<Result<String, FormatError>>,
    block_replacement: OnceCell<Option<&'a str>>,
}

impl<'q, 'a> ScriptFormatQuery<'q, 'a> {
    /// Bind an observed whole Program and the settings for this request.
    /// Recovered Programs keep Glyph's own printing-profile admission.
    ///
    /// # Errors
    /// Wrapped expressions and other grammar shapes return their actual shape.
    #[inline]
    pub fn new(
        arena: &'q Allocator,
        program: &'q EmbedSyntax<'a>,
        options: &'q FormatOptions,
        sorting: Option<&'q ImportSortOptions>,
    ) -> Result<Self, Shape> {
        let shape = program.grammar().shape;
        if shape != Shape::Program {
            return Err(shape);
        }
        Ok(Self {
            arena,
            program,
            options,
            sorting,
            output: OnceCell::new(),
            block_replacement: OnceCell::new(),
        })
    }

    #[inline]
    fn format(&self) -> Result<String, FormatError> {
        format_script_content_stable(
            self.program.source().text(),
            self.options,
            self.arena,
            self.program.source_type(),
            self.sorting,
        )
    }

    /// Borrow standalone output. A refusal borrows the original formatter error.
    #[inline]
    pub fn standalone(&self) -> Result<&str, &FormatError> {
        self.output.get_or_init(|| self.format()).as_deref()
    }

    /// Borrow the SFC script body. A nonempty body that formatted to an empty
    /// program preserves its descriptor identity using Glyph's SFC rule.
    #[inline]
    pub fn sfc_body(&self) -> Result<&str, &FormatError> {
        let output = self.standalone()?;
        let replacement = self.block_replacement.get_or_init(|| {
            let source = self.program.source().text();
            if !output.trim().is_empty() || source.trim().is_empty() {
                None
            } else {
                Some(retained_empty_body(source, self.program.source_type()))
            }
        });
        Ok(replacement.unwrap_or(output))
    }

    /// Transfer the standalone result to a consumer that needs ownership.
    /// An unread query runs the same formatter directly.
    #[inline]
    pub fn into_standalone(mut self) -> Result<String, FormatError> {
        self.output.take().unwrap_or_else(|| self.format())
    }
}

#[cfg(test)]
mod tests;
