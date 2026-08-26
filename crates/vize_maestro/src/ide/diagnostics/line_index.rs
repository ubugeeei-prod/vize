//! Byte-offset to (line, UTF-16 column) mapping for LSP coordinates.
//!
//! Re-exports the shared, UTF-16-correct utility from `vize_s0`. The column
//! math (UTF-16 code units, so astral characters count as two) lives in exactly
//! one place; see `vize_s0::line_index` and issue #1389.

pub(in crate::ide) use vize_s0::line_index::LineIndex;
#[cfg(test)]
pub(in crate::ide) use vize_s0::line_index::offset_to_line_col;
