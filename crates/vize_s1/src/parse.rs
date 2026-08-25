//! The parse entry: armature's tokenizer → event stream → surface tree,
//! with the byte-fidelity verifier asserted on every construction.

use vize_armature::tokenizer::Tokenizer;
use vize_relief::ErrorCode;
use vize_s0::{Allocator, Vec};

use crate::build::build;
use crate::event::{Event, Recorder};
use crate::render::check_fidelity;
use crate::surface::SurfaceTree;

/// A recoverable tokenizer diagnostic, by code and byte offset. S1 keeps
/// armature's codes verbatim; rendering them through the P2-1
/// `Diagnostic` channel is the S1→S2 lowering's job (P2-8).
#[derive(Debug, Clone, Copy)]
pub struct SurfaceError {
    pub code: ErrorCode,
    pub offset: u32,
}

/// Parse a Vue template into its lossless S1 surface tree.
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
/// fidelity is unaffected and the semantic decision is S1→S2's).
///
/// [`Interpolation`]: crate::surface::Interpolation
pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    assert!(
        u32::try_from(source.len()).is_ok(),
        "S1 sources are u32-addressed"
    );
    let mut events: Vec<'a, Event> = Vec::new_in(&allocator);
    let mut errors: Vec<'a, SurfaceError> = Vec::new_in(&allocator);
    {
        let recorder = Recorder {
            events: &mut events,
            errors: &mut errors,
        };
        let mut tokenizer = Tokenizer::new(source, recorder);
        tokenizer.tokenize();
    }
    let tree = build(allocator, source, &events);
    debug_assert!(
        check_fidelity(&tree).is_ok(),
        "S1 fidelity: render(tree) != source"
    );
    (tree, errors)
}
