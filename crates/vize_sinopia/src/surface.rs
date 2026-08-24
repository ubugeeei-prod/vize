//! The S1 surface-tree node types.
//!
//! Every node is a composition of [`Token`]s; a token is two `&'a str`
//! slices of the input (`leading` + `text`), so rendering the tree in
//! order reproduces the source bytes exactly. The typed holes — `Missing`
//! tokens, [`ElementClose::Missing`], [`SurfaceChild::Unexpected`] — are
//! the crate-level hole policy's clauses 1 and 2 (see the crate docs).

use vize_s0::{Box, Vec};

/// Whether a token's syntax is present in the source or a typed hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenStatus {
    /// The token's `text` is real source bytes.
    Present,
    /// Required-but-absent syntax (hole policy clause 1). `text` is a
    /// zero-width slice at the offset where the syntax belongs; `leading`
    /// may still carry real gap bytes.
    Missing,
}

/// One terminal of the surface tree: the verbatim gap since the previous
/// terminal (`leading`) plus the terminal's own text.
///
/// `leading` is pure inter-token whitespace in well-formed source; in
/// recovered source it may carry junk bytes under a reported diagnostic
/// (hole policy clause 3). Both slices point into the parsed source, so
/// `render == source` holds by construction and Drop-freedom is trivial.
#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    /// Verbatim source between the previous terminal and `text`.
    pub leading: &'a str,
    /// The terminal's own bytes; empty when [`TokenStatus::Missing`].
    pub text: &'a str,
    /// Present source syntax, or a typed `Missing` hole.
    pub status: TokenStatus,
}

impl<'a> Token<'a> {
    pub(crate) fn present(leading: &'a str, text: &'a str) -> Self {
        Self {
            leading,
            text,
            status: TokenStatus::Present,
        }
    }

    pub(crate) fn missing(leading: &'a str, at: &'a str) -> Self {
        debug_assert!(at.is_empty());
        Self {
            leading,
            text: at,
            status: TokenStatus::Missing,
        }
    }

    pub fn is_missing(&self) -> bool {
        self.status == TokenStatus::Missing
    }
}

/// A child at any children level (the root or an element's contents).
#[derive(Debug)]
pub enum SurfaceChild<'a> {
    Element(Box<'a, Element<'a>>),
    /// A raw text run — entities undecoded, whitespace uncondensed.
    Text(Token<'a>),
    /// `{{ expr }}` with its delimiters as tokens.
    Interpolation(Box<'a, Interpolation<'a>>),
    /// A whole comment, framing included: `<!-- … -->` (and the abrupt
    /// empty forms `<!-->` / `<!--->`). Declarations (`<!DOCTYPE …>`)
    /// are consumed silently by the tokenizer in template mode and
    /// surface as [`SurfaceChild::Unexpected`] instead.
    Comment(Token<'a>),
    /// A whole `<![CDATA[ … ]]>` section, framing included.
    Cdata(Token<'a>),
    /// A whole `<? … >` processing instruction, framing included.
    ProcessingInstruction(Token<'a>),
    /// Hole policy clause 2: verbatim bytes with no structural home
    /// (stray end tag, `</>`, trailing junk at EOF).
    Unexpected(Token<'a>),
}

/// `{{ expr }}`. The tokenizer only reports an interpolation once its
/// closing delimiter matched, so all three tokens are always present
/// (an unterminated `{{ …` at EOF is a [`SurfaceChild::Text`] run, the
/// tokenizer's own recovery).
#[derive(Debug)]
pub struct Interpolation<'a> {
    /// The opening delimiter (`{{` — v1 supports the default delimiters).
    pub open: Token<'a>,
    /// The expression bytes between the delimiters, verbatim.
    pub content: Token<'a>,
    /// The closing delimiter (`}}`).
    pub close: Token<'a>,
}

/// An element: open tag, contents, and how the element ended.
#[derive(Debug)]
pub struct Element<'a> {
    pub open: OpenTag<'a>,
    pub children: Vec<'a, SurfaceChild<'a>>,
    pub close: ElementClose<'a>,
}

impl<'a> Element<'a> {
    /// The tag name as authored (the `lt_name` text without its `<`).
    pub fn tag(&self) -> &'a str {
        let text = self.open.lt_name.text;
        text.get(1..).unwrap_or(text)
    }
}

/// `<name attrs* /? >`.
#[derive(Debug)]
pub struct OpenTag<'a> {
    /// `<` plus the tag name, one token (`"<div"`).
    pub lt_name: Token<'a>,
    pub attrs: Vec<'a, Attribute<'a>>,
    /// The self-closing `/`, when authored.
    pub slash: Option<Token<'a>>,
    /// The closing `>`; a `Missing` hole at EOF-in-tag.
    pub gt: Token<'a>,
}

/// How an element's extent ended (the node-level half of hole clause 1).
#[derive(Debug)]
pub enum ElementClose<'a> {
    /// A real end tag.
    Present(CloseTag<'a>),
    /// Required but absent: the element was still open when its parent
    /// closed or the input ended. A typed hole, zero-width.
    Missing,
    /// Never required: the open tag was self-closing, or the tag is an
    /// HTML void element.
    NotExpected,
}

/// `</name >`.
#[derive(Debug)]
pub struct CloseTag<'a> {
    /// `</` plus the name, one token (`"</div"`, whitespace kept
    /// verbatim when authored as `</ div`).
    pub lt_slash_name: Token<'a>,
    /// The closing `>`; a `Missing` hole at EOF.
    pub gt: Token<'a>,
}

/// An attribute or directive. Directives stay *raw* at S1: `name` spans
/// the authored spelling (`v-on:click.stop`, `:[key]`, `#slot`) and the
/// semantic split into head/argument/modifiers is S1→S2's job (P2-8).
#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: Token<'a>,
    /// The `=`, when authored (absent in the recovered `a"v"` form too).
    pub eq: Option<Token<'a>>,
    pub value: Option<AttrValue<'a>>,
}

/// An attribute value with its quotes.
#[derive(Debug)]
pub struct AttrValue<'a> {
    /// `None` for unquoted values.
    pub open_quote: Option<Token<'a>>,
    /// The value bytes, verbatim (entities undecoded). A `Missing` hole
    /// when a value was announced but absent (`a= >`).
    pub content: Token<'a>,
    /// `None` for unquoted values; a `Missing` hole when the quote is
    /// unterminated at EOF.
    pub close_quote: Option<Token<'a>>,
}

impl<'a> AttrValue<'a> {
    pub(crate) fn unquoted(content: Token<'a>) -> Self {
        Self {
            open_quote: None,
            content,
            close_quote: None,
        }
    }
}

/// The parse result: the source and the root children level.
#[derive(Debug)]
pub struct SurfaceTree<'a> {
    pub source: &'a str,
    pub children: Vec<'a, SurfaceChild<'a>>,
}

/// Node footprints are pinned (P1-10 discipline: slices, not owned
/// strings; oxc containers). All of these hold pointers, so the figures
/// are 64-bit footprints and the asserts are guarded the way
/// `vize_relief`'s are ("the wasm32 build is 32-bit",
/// `crates/vize_relief/src/relief/elements.rs:31-36`).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<Token<'_>>() == 40);
    assert!(size_of::<Option<Token<'_>>>() == 40);
    assert!(size_of::<Interpolation<'_>>() == 120);
    assert!(size_of::<Attribute<'_>>() == 200);
    assert!(size_of::<AttrValue<'_>>() == 120);
    assert!(size_of::<OpenTag<'_>>() == 144);
    assert!(size_of::<CloseTag<'_>>() == 80);
    assert!(size_of::<ElementClose<'_>>() == 80);
    assert!(size_of::<Element<'_>>() == 248);
    assert!(size_of::<SurfaceChild<'_>>() == 48);
    assert!(size_of::<SurfaceTree<'_>>() == 40);
};
