//! Where a block's open-tag attributes go.

use alloc::borrow::Cow;

/// Receives a block's open-tag attributes in source order.
///
/// `span` covers the whole attribute (name through closing quote).
pub trait AttrSink<'a> {
    fn attr(&mut self, name: Cow<'a, str>, value: Cow<'a, str>, span: (usize, usize));
}
