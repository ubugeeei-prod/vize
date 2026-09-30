//! Authored bytes and decoded expression coordinates for future L1 embeds.
//!
//! Entity decoding belongs to L1, before the language parser. Consumers must
//! not decode HTML again or reparse an embed in L2/L4. See #6836's design at
//! <https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929>.

use vize_l0::{Allocator, Span};

use super::{Embed, Grammar};

/// One correspondence between decoded text and the authoritative authored file.
///
/// `decoded` is relative to the embed's decoded text; `authored` uses absolute
/// UTF-8 byte offsets in the file. Entity expansion may change their lengths.
/// Validated construction and span translation are TODO (#6836).
#[derive(Debug, Clone, Copy)]
pub struct DecodeSegment {
    pub decoded: Span,
    pub authored: Span,
}

/// Borrowed decode correspondence, retained with the embedded syntax artifact.
///
/// TODO (#6836): define complete coverage, entity-boundary translation and
/// wrapper-span correction laws before a language provider consumes this map.
#[derive(Debug, Clone, Copy)]
pub struct DecodeMap<'a> {
    pub segments: &'a [DecodeSegment],
}

/// The authored file range and text passed to a future language provider.
///
/// With no decoding, `text` must be the exact authored slice and `decoded` is
/// `None`. Decoded text and map storage will borrow the shared arena; authored
/// bytes stay authoritative. This record alone does not validate these laws.
#[derive(Debug, Clone, Copy)]
pub struct EmbedSource<'a> {
    pub span: Span,
    pub text: &'a str,
    pub decoded: Option<DecodeMap<'a>>,
}

/// Prepare an attribute value's source without parsing its language syntax.
///
/// TODO (#6836): validate the authored range, decode entities into the shared
/// arena only when necessary, and retain the exact authored-to-decoded map.
/// The caller supplies the already-resolved language and dialect-owned shape.
/// No current production parser calls this deliberately unfinished entry point.
///
/// # Panics
///
/// Always panics until the native source preparation is implemented.
#[expect(
    clippy::todo,
    reason = "Explicit unfinished #6836 skeleton; no production callers"
)]
pub fn prepare_attribute_value<'a>(
    _allocator: &'a Allocator,
    _authored_source: &'a str,
    _span: Span,
    _grammar: Grammar,
) -> Embed<'a> {
    todo!("#6836: prepare L1 attribute source and its entity-decode map")
}
