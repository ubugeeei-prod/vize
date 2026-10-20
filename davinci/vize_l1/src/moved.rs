//! Root paths of the markup surface, kept for one minor release (#6835).

use vize_l0::{Allocator, Vec};

use crate::markup;

macro_rules! moved_types {
    ($($name:ident $(<$lt:lifetime>)?),* $(,)?) => {
        $(
            #[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup`")]
            pub type $name $(<$lt>)? = markup::$name $(<$lt>)?;
        )*
    };
}

moved_types! {
    AttrValue<'a>, Attribute<'a>, CloseTag<'a>, Element<'a>, ElementClose<'a>,
    Interpolation<'a>, OpenTag<'a>, SurfaceChild<'a>, SurfaceTree<'a>, Token<'a>,
    TokenStatus, SurfaceError, SurfaceParseOptions, HoleCounts,
}

#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::parse`")]
pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (markup::SurfaceTree<'a>, Vec<'a, markup::SurfaceError>) {
    markup::parse(allocator, source)
}

#[deprecated(
    since = "0.475.0",
    note = "moved to `vize_l1::markup::parse_with_options`"
)]
pub fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: markup::SurfaceParseOptions,
) -> (markup::SurfaceTree<'a>, Vec<'a, markup::SurfaceError>) {
    markup::parse_with_options(allocator, source, options)
}

#[deprecated(
    since = "0.475.0",
    note = "moved to `vize_l1::markup::parse_with_authored`"
)]
pub fn parse_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    markup::SurfaceTree<'a>,
    Option<markup::SurfaceTree<'a>>,
    Vec<'a, markup::SurfaceError>,
) {
    markup::parse_with_authored(allocator, source)
}

#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::render`")]
pub fn render<'a>(tree: &markup::SurfaceTree<'a>, sink: &mut dyn FnMut(&'a str)) {
    markup::render(tree, sink);
}

#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::check_fidelity`")]
pub fn check_fidelity(tree: &markup::SurfaceTree<'_>) -> Result<(), usize> {
    markup::check_fidelity(tree)
}

#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::hole_counts`")]
pub fn hole_counts(tree: &markup::SurfaceTree<'_>) -> markup::HoleCounts {
    markup::hole_counts(tree)
}

/// The surface node types.
#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::surface`")]
pub mod surface {
    #[expect(deprecated, reason = "re-exports the deprecated root items")]
    pub use super::{
        AttrValue, Attribute, CloseTag, Element, ElementClose, Interpolation, OpenTag,
        SurfaceChild, SurfaceTree, Token, TokenStatus,
    };
}

/// The surface parse entry.
#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::parse`")]
pub mod parse {
    #[expect(deprecated, reason = "re-exports the deprecated root items")]
    pub use super::{
        SurfaceError, SurfaceParseOptions, parse, parse_with_authored, parse_with_options,
    };
}

/// Rendering and the fidelity check.
#[deprecated(since = "0.475.0", note = "moved to `vize_l1::markup::render`")]
pub mod render {
    #[expect(deprecated, reason = "re-exports the deprecated root items")]
    pub use super::{HoleCounts, check_fidelity, hole_counts, render};
}
