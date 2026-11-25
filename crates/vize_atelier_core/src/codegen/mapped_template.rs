//! The template of a compile that records a source map.
//!
//! Some generated text maps to template bytes its node carries no location
//! for: a `v-for` alias, and a component tag read as a setup binding. Both
//! are still slices of the template the parser cut, so their position is
//! where the slice starts. A compile that asked for a map hands the template
//! to [`prepare_mapped_template`] before it generates code; the generator
//! then anchors those slices inside the recording checks it already makes,
//! and a compile without a map runs none of this.

use std::cell::Cell;

#[cfg(doc)]
use super::document::authored_offset;
use super::for_alias_spans::locate_for_aliases;
use crate::RootNode;

std::thread_local! {
    /// The address and length of the template being generated with a map,
    /// until its map is serialized.
    static TEMPLATE: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

/// Prepare `root`, transformed from `source`, for generating code with a
/// source map: locate its `v-for` aliases and keep `source` for the tags the
/// generator reads as setup bindings, until the map is serialized.
#[cold]
#[inline(never)]
pub fn prepare_mapped_template(root: &mut RootNode<'_>, source: &str) {
    locate_for_aliases(root, source);
    TEMPLATE.set((source.as_ptr() as usize, source.len()));
}

/// Where `text`, a slice of the prepared template, starts in it.
/// Only addresses are compared, as [`authored_offset`] does.
pub(super) fn template_offset(text: &str) -> Option<u32> {
    let (address, len) = TEMPLATE.get();
    let start = (text.as_ptr() as usize).checked_sub(address)?;
    let inside = address != 0 && !text.is_empty() && start + text.len() <= len;
    inside.then(|| u32::try_from(start).ok()).flatten()
}

/// Forget the prepared template once its map is serialized.
pub(super) fn forget_mapped_template() {
    TEMPLATE.set((0, 0));
}
