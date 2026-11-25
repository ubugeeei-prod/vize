//! Ordered, expanded CSS declarations for an output target or an analysis pass.
use alloc::vec::Vec;
use vize_l0::{Span, String};

#[derive(Debug)]
pub struct RuleRun<'a> {
    pub selector: String,
    pub selector_span: Span,
    pub group: usize,
    pub declarations: Vec<Property<'a>>,
}

#[derive(Debug)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: String,
    pub name_span: Span,
    pub value_span: Span,
}

/// A lowering owns selector expansion, declaration order and source provenance.
/// Group boundaries separate original rule groups in expanded CSS. All spans
/// are file-absolute. Values may originate in an earlier declaration.
pub trait ExpandedRules<'a> {
    fn runs(&self) -> &[RuleRun<'a>];

    /// Create a cursor for one walk in emission order. The lowering chooses
    /// which values carry a link and where each one originated. Unmapped
    /// output never constructs this cursor.
    fn value_links(&self) -> impl FnMut(&Property<'a>) -> Option<Span>;
}

pub(crate) mod sealed {
    pub trait ScopeAtSelectorEnd {}
}

/// The final compound of every selector accepts a scope attribute at its end.
/// Only lowerings that establish this during selector admission implement it.
pub trait ScopeAtSelectorEnd<'a>: ExpandedRules<'a> + sealed::ScopeAtSelectorEnd {}
