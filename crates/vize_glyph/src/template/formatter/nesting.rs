//! The element nesting the template formatter lays out.
//!
//! Every output line is indented once per open element, so the bytes written
//! grow with nesting depth times line count. Nesting past the limit is a
//! format error instead: the caller keeps the template as authored.

use crate::error::FormatError;

/// Deepest element nesting the formatter indents.
///
/// The same value as the template parser's limit, `MAX_ELEMENT_NESTING_DEPTH`
/// in `vize_armature::parser::element::nesting`. The formatter scans bytes
/// without that parser and cannot import its private constant, so the value
/// is repeated here: a template the compiler refuses is not laid out either.
pub(crate) const MAX_ELEMENT_NESTING_DEPTH: usize = 4096;

/// Reported for a template nested past [`MAX_ELEMENT_NESTING_DEPTH`].
pub(crate) const NESTING_DEPTH_ERROR: &str = "element nesting exceeds the supported depth of 4096";

/// Admit one more open element around the content that follows.
///
/// `open_elements` counts the elements already open at this point.
#[inline]
pub(super) fn open_element(open_elements: usize) -> Result<(), FormatError> {
    if open_elements >= MAX_ELEMENT_NESTING_DEPTH {
        return Err(nesting_depth_error());
    }
    Ok(())
}

#[cold]
#[inline(never)]
fn nesting_depth_error() -> FormatError {
    FormatError::TemplateFormatError(NESTING_DEPTH_ERROR.into())
}
