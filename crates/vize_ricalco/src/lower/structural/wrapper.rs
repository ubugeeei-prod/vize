//! The `<template v-if>` wrapper-key channel (P2-9 series 5): the
//! branch key the legacy transform lifts off the wrapper element
//! (`extract_key_prop` runs on the branch node itself, template
//! wrappers included). The wrapper unwraps at lowering, so its key has
//! no op to ride — it is captured here as a fact channel the `v-if`
//! pass folds into branch-key facts — and the remaining wrapper
//! attributes stay recorded drops.

use alloc::vec::Vec as StdVec;

use vize_s0::{Span, String, cstr};
use vize_s1::Element;

use super::super::cx::{Cx, attr_slice, attr_span};
use super::super::directive::{Arg, AttrForm, Head};
use super::super::element::{Analyzed, attr_value_text};

/// One captured `<template v-if>` wrapper key (P2-9 series 5): the
/// branch key the legacy transform lifts off the wrapper element
/// (`extract_key_prop` runs on the branch node itself, template wrappers
/// included). The wrapper unwraps at lowering, so its key has no op to
/// ride — it is captured here as a fact channel the `v-if` pass folds
/// into [`BranchKey`](crate::pass::vif::BranchKey) facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrapperKey {
    /// A static `key` attribute; `None` = bare (never collides).
    Static {
        /// The authored value.
        value: Option<String>,
        /// The attribute's range.
        span: Span,
    },
    /// A `:key` binding: the trimmed authored value text (the parser's
    /// same-name expansion applied — a valueless `:key` reads `key`).
    Dynamic {
        /// The trimmed value text.
        source: String,
        /// The attribute's range.
        span: Span,
    },
}

/// Per-`ui.if` captured wrapper keys, one slot per branch in authored
/// order; keyed by the `ui.if` op's page-order id.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WrapperKeys {
    /// `branches[i]` is branch `i`'s wrapper key, when its carrier was
    /// an unwrapped `<template>` with one.
    pub branches: StdVec<Option<WrapperKey>>,
    /// `from_template[i]` is true when branch `i`'s carrier was an
    /// unwrapped `<template>` (slot-bearing templates stay wrapped and
    /// stay false). P2-11 emit needs this: after hoist, a static child
    /// of `<template v-if>` is a fragment, while the same element as a
    /// `v-if` branch root is a block.
    pub from_template: StdVec<bool>,
}

/// A `<template v-for>` the lowering unwraps: presence of this fact on
/// a `ui.for` is the emit signal, and `key` is the wrapper `:key` /
/// `key` the legacy lane keeps on the template vnode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ForWrapper {
    /// The wrapper's captured key, when it authored one.
    pub key: Option<WrapperKey>,
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<WrapperKey>();
    assert_owned::<WrapperKeys>();
    assert_owned::<ForWrapper>();
};

/// 64-bit footprints, guarded like every fact-size assert.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<WrapperKey>() == 40);
    assert!(core::mem::size_of::<WrapperKeys>() == 48);
    assert!(core::mem::size_of::<ForWrapper>() == 40);
};

/// The first key-ish attribute of a `<template v-if>` wrapper: a static
/// `key` attribute or a static-argument `:key` binding, in authored
/// order — the legacy `extract_key_prop` predicate. A dynamic-argument
/// `:[key]` is **not** captured (the legacy lane's arg-content match
/// admits it — a recorded quirk the differential lane counts rather
/// than imitates).
pub(crate) fn capture_wrapper_key<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
) -> Option<(usize, WrapperKey)> {
    for (index, attr) in element.open.attrs.iter().enumerate() {
        if Some(index) == analyzed.branch.map(|(idx, _)| idx) || Some(index) == analyzed.vfor {
            continue;
        }
        match &analyzed.forms[index] {
            AttrForm::Static if attr.name.text == "key" => {
                return Some((
                    index,
                    WrapperKey::Static {
                        value: attr_value_text(element, index).map(String::from),
                        span: attr_span(cx, attr),
                    },
                ));
            }
            AttrForm::Directive(directive)
                if directive.head == Head::Bind && directive.arg == Some(Arg::Static("key")) =>
            {
                let source = match attr_value_text(element, index) {
                    Some(text) => String::from(text.trim()),
                    // The parser's same-name expansion: a valueless
                    // `:key` reads `key`.
                    None => String::from("key"),
                };
                return Some((
                    index,
                    WrapperKey::Dynamic {
                        source,
                        span: attr_span(cx, attr),
                    },
                ));
            }
            _ => {}
        }
    }
    None
}

/// A `<template>` wrapper the lowering unwraps has no op to carry its
/// remaining attributes; each is dropped under an `Info` diagnostic and
/// a record — dropping is never silent. `captured` names the wrapper-key
/// attribute the caller lifted into the wrapper-key channel (P2-9
/// series 5), which is therefore not a drop.
pub(crate) fn record_template_drops<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    captured: Option<usize>,
) {
    for (index, attr) in element.open.attrs.iter().enumerate() {
        if Some(index) == analyzed.branch.map(|(idx, _)| idx)
            || Some(index) == analyzed.vfor
            || Some(index) == captured
        {
            continue;
        }
        cx.info(
            attr_span(cx, attr),
            cstr!(
                "`{}` on an unwrapped `<template>` wrapper has no S2 home; it is dropped under provenance",
                attr.name.text
            ),
        );
        cx.record(
            "drop.template-attribute",
            None,
            attr_slice(cx, attr),
            String::default(),
            attr_span(cx, attr),
        );
    }
}
