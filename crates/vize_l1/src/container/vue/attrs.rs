//! Where a block's open-tag attributes go.

use alloc::borrow::Cow;

use vize_l0::FxHashMap;

/// Receives a block's open-tag attributes in source order.
///
/// `span` covers the whole attribute (name through closing quote).
pub trait AttrSink<'a> {
    fn attr(&mut self, name: Cow<'a, str>, value: Cow<'a, str>, span: (usize, usize));
}

/// The legacy descriptor's map: a later duplicate replaces an earlier one.
impl<'a> AttrSink<'a> for FxHashMap<Cow<'a, str>, Cow<'a, str>> {
    fn attr(&mut self, name: Cow<'a, str>, value: Cow<'a, str>, _span: (usize, usize)) {
        self.insert(name, value);
    }
}
