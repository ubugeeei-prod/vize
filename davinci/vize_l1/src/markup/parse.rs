//! Native Vue component surface construction from the generic lexer.

use vize_l0::{Allocator, Vec};

use super::{Component, LexOptions, Lexer};
use crate::event::Recorder;
use crate::parse::{SurfaceError, SurfaceParseOptions, construct};
use crate::surface::SurfaceTree;

/// Construct a lossless Vue component surface directly from [`Lexer`].
///
/// This entry point uses the existing surface recovery rules, `{{`/`}}`
/// delimiters and no raw-HTML interpolation. It preserves authored entities
/// without decoding the tree's source slices. Malformed input retains typed
/// holes and recoverable diagnostics, with byte-exact rendering.
///
/// `v-pre` suppression is unfinished: mustaches inside it remain interpolation
/// nodes, as in the root parse APIs. Document-profile tree rules, custom
/// delimiters, typed embeds and dialect-specific switches are also unfinished.
/// Compiler products retain their current parse routes until their gates pass.
pub fn parse_component<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    parse_component_with_options(allocator, source, SurfaceParseOptions::default())
}

/// [`parse_component`] with the existing in-tag-comment surface switch.
/// The narrow options cannot enable unsupported delimiter or profile modes.
pub fn parse_component_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    let (tree, _, errors) = projection(allocator, source, options, false);
    (tree, errors)
}

/// Parse once and retain an authored projection when interactive-tag recovery
/// changes the tree. Both projections use the same native event stream.
pub fn parse_component_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    projection(allocator, source, SurfaceParseOptions::default(), true)
}

fn projection<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
    authored: bool,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    construct(allocator, source, authored, |events, errors| {
        let recorder = Recorder { events, errors };
        Lexer::<Component, _>::new(
            source,
            recorder,
            LexOptions {
                in_tag_comments: options.experimental_in_tag_comments,
                ..LexOptions::default()
            },
        )
        .run();
    })
}
