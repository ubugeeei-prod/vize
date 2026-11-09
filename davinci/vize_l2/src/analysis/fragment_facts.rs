//! Owned fragment-carrier facts: a control-flow body realized through an
//! unwrapped carrier instead of a single root.
//!
//! An input lowering decides which authored surface unwraps into such a
//! carrier (a Vue `<template v-if>` / `<template v-for>`) and records what the
//! carrier itself authored: its key, its attributes and class, a chain-wide
//! `once`, or a runtime directive no L2 op can hold. Capture policy and its
//! provenance stay with the lowering; backends read only these facts.

use alloc::vec::Vec as StdVec;
use vize_l0::{Span, String};

/// One carrier's authored key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentKey {
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

/// Per-`ui.if` carrier facts, one slot per branch in authored order; keyed
/// by the `ui.if` op's page-order id.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BranchFragments {
    /// `branches[i]` is branch `i`'s carrier key, when its carrier authored
    /// one.
    pub branches: StdVec<Option<FragmentKey>>,
    /// `fragment[i]` is true when branch `i` renders through an unwrapped
    /// carrier (slot-bearing carriers stay wrapped and stay false). After
    /// hoisting, a static child of such a branch is a fragment, while the
    /// same element as a branch root is a block.
    pub fragment: StdVec<bool>,
    /// The chain's first carrier was rendered once: the whole chain is
    /// cached, exactly as a once-rendered conditional root is.
    pub once: bool,
}

/// An unwrapped `ui.for` body carrier: presence on a `ui.for` is the
/// signal, and `key` is the carrier key kept on the fragment vnode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoopFragment {
    /// The carrier's authored key, when it has one.
    pub key: Option<FragmentKey>,
    /// Static non-`class` attributes authored on the carrier. The shipped
    /// lane carries them on the generated fragment vnode.
    pub attributes: StdVec<FragmentAttr>,
    /// Static and dynamic `class` authored on the carrier.
    pub class: Option<FragmentClass>,
    /// A runtime directive (`v-show`, a custom directive) authored on the
    /// carrier. The shipped lane applies it to the fragment vnode; L2 has no
    /// home for it, so emission refuses the loop.
    pub runtime_directive: Option<Span>,
}

/// One static attribute authored on an unwrapped loop carrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentAttr {
    /// Attribute name, as authored.
    pub name: String,
    /// Attribute value; `None` represents a bare static attribute.
    pub value: Option<String>,
    /// The attribute's range.
    pub span: Span,
}

/// `class` authored on an unwrapped loop carrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentClass {
    /// Static class text, when authored.
    pub static_value: Option<String>,
    /// Dynamic class expression source, when authored.
    pub dynamic_source: Option<String>,
    /// Representative class attribute range.
    pub span: Span,
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<FragmentKey>();
    assert_owned::<BranchFragments>();
    assert_owned::<LoopFragment>();
    assert_owned::<FragmentAttr>();
    assert_owned::<FragmentClass>();
};

/// 64-bit footprints, guarded like every fact-size assert.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<FragmentKey>() == 40);
    assert!(core::mem::size_of::<BranchFragments>() == 56);
    assert!(core::mem::size_of::<LoopFragment>() == 136);
};
