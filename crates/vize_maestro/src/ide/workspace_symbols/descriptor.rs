//! Borrow SFC text while the synchronous workspace-symbol collector owns its input.

pub(super) use vize_incremental::parse_borrowed_descriptor as parse;

#[cfg(test)]
mod tests;
