//! The parse entry: armature's tokenizer → event stream → surface tree,
//! with the byte-fidelity verifier asserted on every construction.

use vize_armature::tokenizer::Tokenizer;
use vize_l0::{Allocator, Vec};
use vize_relief::ErrorCode;

use crate::build::build;
use crate::event::{Event, Recorder};
use crate::render::check_fidelity;
use crate::surface::{SurfaceChild, SurfaceTree, Token};

/// Parse-time switches L1 owns for the L2 consumers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SurfaceParseOptions {
    /// Accept Vue's experimental `//` comments inside opening tag attrs.
    pub experimental_in_tag_comments: bool,
}

/// A recoverable tokenizer diagnostic, by code and byte offset. L1 keeps
/// armature's codes verbatim; rendering them through the P2-1
/// `Diagnostic` channel is the L1→L2 lowering's job (P2-8).
#[derive(Debug, Clone, Copy)]
pub struct SurfaceError {
    pub code: ErrorCode,
    pub offset: u32,
}

/// Parse a Vue template into its lossless L1 surface tree.
///
/// Total over arbitrary input: malformed source becomes typed holes (see
/// the crate-level hole policy), never a panic and never dropped bytes.
/// The debug verifier asserts `render(tree) == source` bytes on every
/// construction — the cheapest high-yield verifier in the prior-art
/// survey (SwiftSyntax import).
///
/// v1 scope, recorded in the P2-7 record: the tokenizer's default
/// `{{`/`}}` delimiters, and no `v-pre` interpolation suppression (a
/// `v-pre` subtree's `{{ x }}` is an [`Interpolation`] node here; byte
/// fidelity is unaffected and the semantic decision is L1→L2's).
///
/// [`Interpolation`]: crate::surface::Interpolation
pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    parse_with_options(allocator, source, SurfaceParseOptions::default())
}

/// [`parse`] with explicit L1 parse switches.
pub fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    let (tree, _, errors) = parse_projection(allocator, source, options, false);
    (tree, errors)
}

/// Parse once, retaining a non-repairing authored projection when the normal
/// surface construction implicitly closes a nested interactive element.
/// Both projections share the tokenizer's event stream and source slices.
pub fn parse_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    parse_projection(allocator, source, SurfaceParseOptions::default(), true)
}

fn parse_projection<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
    authored: bool,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    let mut events: Vec<'a, Event> = Vec::new_in(&allocator);
    let mut errors: Vec<'a, SurfaceError> = Vec::new_in(&allocator);
    // L1 addresses sources with `u32` offsets. A larger source keeps byte
    // fidelity as one typed `Unexpected` hole instead of mis-measured nodes.
    if u32::try_from(source.len()).is_err() {
        let mut children = Vec::new_in(&allocator);
        let hole = Token::present(crate::slice::range(source, 0, 0), source);
        children.push(SurfaceChild::Unexpected(hole));
        return (SurfaceTree { source, children }, None, errors);
    }
    {
        let recorder = Recorder {
            events: &mut events,
            errors: &mut errors,
        };
        let mut tokenizer = Tokenizer::new(source, recorder);
        tokenizer.set_in_tag_comments(options.experimental_in_tag_comments);
        tokenizer.tokenize();
    }
    let (tree, repaired) = build(allocator, source, &events, true);
    let authored = (authored && repaired).then(|| build(allocator, source, &events, false).0);
    debug_assert!(
        check_fidelity(&tree).is_ok(),
        "L1 fidelity: render(tree) != source"
    );
    if let Some(authored) = &authored {
        debug_assert!(check_fidelity(authored).is_ok(), "authored L1 fidelity");
    }
    (tree, authored, errors)
}
