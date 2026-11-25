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
use std::marker::PhantomData;

#[cfg(doc)]
use super::document::authored_offset;
use super::for_alias_spans::locate_for_aliases;
use crate::RootNode;

std::thread_local! {
    /// The address and length of the template being generated with a map,
    /// for the lifetime of its preparation guard.
    static TEMPLATE: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

/// Prepare `root`, transformed from `source`, for generating code with a
/// source map: locate its `v-for` aliases and keep `source` for the tags the
/// generator reads as setup bindings, until the returned guard is dropped.
/// Nested compiles restore the previous template, including on early returns
/// and unwinding. The guard borrows the source whose address it records.
#[cold]
#[inline(never)]
pub fn prepare_mapped_template<'a>(root: &mut RootNode<'_>, source: &'a str) -> MappedTemplate<'a> {
    locate_for_aliases(root, source);
    MappedTemplate {
        previous: TEMPLATE.replace((source.as_ptr() as usize, source.len())),
        source: PhantomData,
    }
}

/// Restore the caller's mapped template when this compile ends.
#[must_use = "keep the guard until code generation and map serialization finish"]
pub struct MappedTemplate<'a> {
    previous: (usize, usize),
    // The source stays borrowed, and the guard must be dropped on the thread
    // whose template it changed. The marker makes it neither Send nor Sync.
    source: PhantomData<(&'a str, *mut ())>,
}

impl Drop for MappedTemplate<'_> {
    fn drop(&mut self) {
        TEMPLATE.set(self.previous);
    }
}

/// Where `text`, a slice of the prepared template, starts in it.
/// Only addresses are compared, as [`authored_offset`] does.
pub(super) fn template_offset(text: &str) -> Option<u32> {
    let (address, len) = TEMPLATE.get();
    let start = (text.as_ptr() as usize).checked_sub(address)?;
    let end = start.checked_add(text.len())?;
    let inside = address != 0 && !text.is_empty() && end <= len;
    inside.then(|| u32::try_from(start).ok()).flatten()
}

#[cfg(test)]
mod tests {
    use super::{prepare_mapped_template, template_offset};
    use vize_l0::Allocator;

    #[test]
    fn nested_preparations_restore_the_callers_template() {
        let allocator = Allocator::new();
        let outer = "<Outer />";
        let inner = "<Inner />";
        let (mut outer_root, _) = crate::parser::parse(&allocator, outer);
        let (mut inner_root, _) = crate::parser::parse(&allocator, inner);
        assert_eq!(template_offset(&outer[1..6]), None);
        {
            let _outer = prepare_mapped_template(&mut outer_root, outer);
            assert_eq!(template_offset(&outer[1..6]), Some(1));
            {
                let _inner = prepare_mapped_template(&mut inner_root, inner);
                assert_eq!(template_offset(&inner[1..6]), Some(1));
                assert_eq!(template_offset(&outer[1..6]), None);
            }
            assert_eq!(template_offset(&outer[1..6]), Some(1));
            assert_eq!(template_offset(&inner[1..6]), None);
        }
        assert_eq!(template_offset(&outer[1..6]), None);
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _outer = prepare_mapped_template(&mut outer_root, outer);
            assert_eq!(template_offset(&outer[1..6]), Some(1));
            panic!("mapped compile aborted");
        }));
        assert_eq!(
            unwound
                .expect_err("the compile unwound")
                .downcast_ref::<&str>(),
            Some(&"mapped compile aborted")
        );
        assert_eq!(template_offset(&outer[1..6]), None);
    }
}
