//! The template of a compile that records a source map.
//!
//! Some generated text maps to template bytes its node carries no location
//! for: a `v-for` alias, and a component tag read as a setup binding. Both
//! are still slices of the template the parser cut, so their position is
//! where the slice starts. A compile that asked for a map hands the template
//! to [`with_mapped_template`] before it generates code; the generator
//! uses those positions only while recording. An unmapped compile does not
//! prepare this context.

use std::cell::Cell;

#[cfg(doc)]
use super::document::authored_offset;
use super::for_alias_spans::locate_for_aliases;
use crate::RootNode;

std::thread_local! {
    /// The address and length of the template being generated with a map,
    /// for the lifetime of its preparation guard.
    static TEMPLATE: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

/// Run `generate` with source-owned aliases and component tag positions when
/// `source_map` is enabled. The private restoration guard cannot escape or be
/// dropped out of order by a caller. Nested compiles and unwinding restore the
/// caller's template before returning.
#[inline]
pub fn with_mapped_template<'a, R>(
    root: &mut RootNode<'a>,
    source: &str,
    source_map: bool,
    generate: impl FnOnce(&RootNode<'a>) -> R,
) -> R {
    if source_map {
        mapped(root, source, generate)
    } else {
        generate(root)
    }
}

#[cold]
#[inline(never)]
fn mapped<'a, R>(
    root: &mut RootNode<'a>,
    source: &str,
    generate: impl FnOnce(&RootNode<'a>) -> R,
) -> R {
    struct Restore<'s> {
        previous: (usize, usize),
        _source: &'s str,
    }
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            TEMPLATE.set(self.previous);
        }
    }
    locate_for_aliases(root, source);
    let _restore = Restore {
        previous: TEMPLATE.replace((source.as_ptr() as usize, source.len())),
        _source: source,
    };
    generate(root)
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
    use super::{template_offset, with_mapped_template};
    use vize_l0::Allocator;

    #[test]
    fn nested_preparations_restore_the_callers_template() {
        let allocator = Allocator::new();
        let outer = "<Outer />";
        let inner = "<Inner />";
        let (mut outer_root, _) = crate::parser::parse(&allocator, outer);
        let (mut inner_root, _) = crate::parser::parse(&allocator, inner);
        assert_eq!(template_offset(&outer[1..6]), None);
        with_mapped_template(&mut outer_root, outer, true, |_| {
            assert_eq!(template_offset(&outer[1..6]), Some(1));
            with_mapped_template(&mut inner_root, inner, true, |_| {
                assert_eq!(template_offset(&inner[1..6]), Some(1));
                assert_eq!(template_offset(&outer[1..6]), None);
            });
            assert_eq!(template_offset(&outer[1..6]), Some(1));
            assert_eq!(template_offset(&inner[1..6]), None);
        });
        assert_eq!(template_offset(&outer[1..6]), None);
        let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_mapped_template(&mut outer_root, outer, true, |_| {
                assert_eq!(template_offset(&outer[1..6]), Some(1));
                panic!("mapped compile aborted");
            });
        }));
        assert_eq!(
            unwound
                .expect_err("the compile unwound")
                .downcast_ref::<&str>(),
            Some(&"mapped compile aborted")
        );
        assert_eq!(template_offset(&outer[1..6]), None);
    }

    #[test]
    fn disabled_or_panicking_nested_generation_keeps_the_outer_source() {
        let allocator = Allocator::new();
        let outer = "<Outer />";
        let inner = "<Inner />";
        let (mut outer_root, _) = crate::parser::parse(&allocator, outer);
        let (mut inner_root, _) = crate::parser::parse(&allocator, inner);
        let returned = with_mapped_template(&mut outer_root, outer, true, |_| {
            with_mapped_template(&mut inner_root, inner, false, |_| {
                assert_eq!(template_offset(&outer[1..6]), Some(1));
                assert_eq!(template_offset(&inner[1..6]), None);
            });
            let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_mapped_template(&mut inner_root, inner, true, |_| {
                    assert_eq!(template_offset(&inner[1..6]), Some(1));
                    panic!("nested compile aborted");
                });
            }));
            assert_eq!(
                unwound
                    .expect_err("nested compile unwound")
                    .downcast_ref::<&str>(),
                Some(&"nested compile aborted")
            );
            assert_eq!(template_offset(&outer[1..6]), Some(1));
            assert_eq!(template_offset(&inner[1..6]), None);
            "generated"
        });
        assert_eq!(returned, "generated");
        assert_eq!(template_offset(&outer[1..6]), None);
    }
}
