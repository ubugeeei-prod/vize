//! The pug S1 surface-tree node types.
//!
//! The shapes follow `pug-parser`'s AST (a tag owns its attribute parts,
//! its same-line content and its indented block) because the lowering
//! replays pug's code generator over them; every terminal is a
//! [`Token`] slice of the authored source, so the in-order render is the
//! source bytes (see [`super::render`]).

use vize_s0::{Box, Vec};

use crate::surface::Token;

/// pug constructs the Vue lowering refuses: they execute JavaScript at
/// build time, link other files, or define/expand mixins — none of which
/// Vue's documented pug support (a static preprocessor pass) covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PugRefusal {
    /// `doctype …`
    Doctype,
    /// `yield`
    Yield,
    /// `#{expr}` used as a tag name.
    Interpolation,
    /// `case …`
    Case,
    /// `when …`
    When,
    /// `default`
    Default,
    /// `extends …`
    Extends,
    /// `block NAME`, `append NAME`, `prepend NAME`
    Block,
    /// A bare `block` (a mixin's block slot).
    MixinBlock,
    /// `include …`
    Include,
    /// `mixin NAME(…)`
    Mixin,
    /// `+NAME(…)` — a mixin call.
    Call,
    /// `if` / `unless` / `else if` / `else`
    Conditional,
    /// `each` / `for`
    Each,
    /// `while …`
    While,
    /// `:filter`
    Filter,
    /// `-` block code.
    BlockCode,
}

impl PugRefusal {
    /// The construct as pug spells it, for diagnostics.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Doctype => "doctype",
            Self::Yield => "yield",
            Self::Interpolation => "#{…} tag interpolation",
            Self::Case => "case",
            Self::When => "when",
            Self::Default => "default",
            Self::Extends => "extends",
            Self::Block => "block",
            Self::MixinBlock => "block (mixin block)",
            Self::Include => "include",
            Self::Mixin => "mixin",
            Self::Call => "+mixin call",
            Self::Conditional => "if/unless/else",
            Self::Each => "each/for",
            Self::While => "while",
            Self::Filter => ":filter",
            Self::BlockCode => "- block code",
        }
    }
}

/// A parsed pug template: the top-level nodes plus a zero-width end token
/// whose `leading` holds the trailing bytes.
#[derive(Debug)]
pub struct PugTree<'a> {
    pub source: &'a str,
    pub nodes: Vec<'a, PugNode<'a>>,
    pub eof: Token<'a>,
}

#[derive(Debug)]
pub enum PugNode<'a> {
    Tag(Box<'a, PugTag<'a>>),
    /// A text run: piped lines (`| …`), joined with pug's `\n` rule.
    Text(Box<'a, PugText<'a>>),
    /// Raw HTML lines (`<…`) with their nested blocks, joined with `\n`.
    Html(Box<'a, PugHtml<'a>>),
    /// A `.` at line start followed by a text block.
    TextBlock(Box<'a, PugDotBlock<'a>>),
    Code(Box<'a, PugCode<'a>>),
    Comment(Box<'a, PugComment<'a>>),
    Refused(Box<'a, PugRefused<'a>>),
    /// Hole clause 2: bytes (or an orphan indented block) with no
    /// structural home.
    Unexpected(Box<'a, PugUnexpected<'a>>),
}

/// `tag(.class | #id | (attrs) | &attributes(…))* .? (text | = code | : expr | /)? block`
#[derive(Debug)]
pub struct PugTag<'a> {
    /// The tag name; `None` for the implicit `div` of `.a` / `#b`.
    pub name: Option<Token<'a>>,
    pub parts: Vec<'a, PugTagPart<'a>>,
    /// The text-only `.` marker.
    pub dot: Option<Token<'a>>,
    pub inline: PugInline<'a>,
    pub block: PugBlock<'a>,
}

impl<'a> PugTag<'a> {
    /// The rendered tag name (`div` when implicit).
    pub fn tag_name(&self) -> &'a str {
        self.name.map_or("div", |token| token.text)
    }
}

#[derive(Debug)]
pub enum PugTagPart<'a> {
    /// `.name` — the token includes the dot.
    Class(Token<'a>),
    /// `#name` — the token includes the hash.
    Id(Token<'a>),
    Attrs(Box<'a, PugAttrGroup<'a>>),
    /// `&attributes(…)`, refused by the lowering.
    AndAttributes(Token<'a>),
}

/// `( attr (,? attr)* )`.
#[derive(Debug)]
pub struct PugAttrGroup<'a> {
    pub open: Token<'a>,
    pub items: Vec<'a, PugAttrItem<'a>>,
    pub close: Token<'a>,
}

#[derive(Debug)]
pub enum PugAttrItem<'a> {
    Attr(PugAttr<'a>),
    /// Bytes after an attribute error, up to the `)`.
    Unexpected(Token<'a>),
}

/// `name`, `name=value`, `name!=value`, each with an optional `,`.
#[derive(Debug)]
pub struct PugAttr<'a> {
    /// The key as authored, quotes included when quoted.
    pub name: Token<'a>,
    /// `=` or `!=`.
    pub op: Option<Token<'a>>,
    /// The JavaScript expression source, verbatim.
    pub value: Option<Token<'a>>,
    pub comma: Option<Token<'a>>,
}

impl PugAttr<'_> {
    /// The key with its quotes removed.
    pub fn key(&self) -> &str {
        let text = self.name.text;
        match text.chars().next() {
            Some(quote @ ('"' | '\'')) => {
                let inner = &text[1..];
                inner.strip_suffix(quote).unwrap_or(inner)
            }
            _ => text,
        }
    }

    /// `!=` — the value is not HTML-escaped.
    pub fn unescaped(&self) -> bool {
        self.op.is_some_and(|op| op.text == "!=")
    }
}

/// What follows a tag's head on the same line.
#[derive(Debug)]
pub enum PugInline<'a> {
    None,
    Text(PugText<'a>),
    Code(Box<'a, PugCode<'a>>),
    /// `: expr` block expansion.
    Expansion {
        colon: Token<'a>,
        node: Box<'a, PugNode<'a>>,
    },
    /// `/` self-closing marker.
    SelfClosing(Token<'a>),
}

/// A tag's indented block: child nodes, or (after `.`) a text block.
#[derive(Debug)]
pub enum PugBlock<'a> {
    Nodes(Vec<'a, PugNode<'a>>),
    Text(PugText<'a>),
}

/// Text pieces, concatenated by the lowering.
#[derive(Debug)]
pub struct PugText<'a> {
    pub pieces: Vec<'a, PugTextPiece<'a>>,
}

#[derive(Debug)]
pub enum PugTextPiece<'a> {
    /// The `|` of a piped line.
    Pipe(Token<'a>),
    /// Verbatim text bytes (no HTML escaping — pug text is raw HTML).
    Raw(Token<'a>),
    /// The `\` of `\#[` / `\#{` / `\!{`, dropped by the lowering.
    Escape(Token<'a>),
    /// A `\n` pug inserts between lines; zero-width, at this offset.
    Newline(u32),
    /// `#[ … ]` tag interpolation.
    Interpolation(Box<'a, PugInterpolation<'a>>),
    /// `#{…}` / `!{…}` code interpolation, refused by the lowering.
    Code(Token<'a>),
    Unexpected(Token<'a>),
}

/// `#[ node ]`; `close` is a `Missing` hole when unterminated. pug parses
/// exactly one expression here; anything more is kept (and diagnosed).
#[derive(Debug)]
pub struct PugInterpolation<'a> {
    pub open: Token<'a>,
    pub nodes: Vec<'a, PugNode<'a>>,
    pub close: Token<'a>,
}

/// Consecutive raw-HTML lines (`pug-parser`'s `parseTextHtml`).
#[derive(Debug)]
pub struct PugHtml<'a> {
    pub items: Vec<'a, PugHtmlItem<'a>>,
}

#[derive(Debug)]
pub enum PugHtmlItem<'a> {
    /// One `<…` line.
    Line(PugText<'a>),
    /// An indented block under an HTML line; its HTML nodes join the run.
    Block(Vec<'a, PugNode<'a>>),
    /// `= code` between HTML lines (it breaks the run).
    Code(Box<'a, PugCode<'a>>),
}

/// `.` then a text block, at expression level.
#[derive(Debug)]
pub struct PugDotBlock<'a> {
    pub dot: Token<'a>,
    pub text: PugText<'a>,
}

/// `= expr`, `!= expr` or `- code`, with an indented block for `-`.
#[derive(Debug)]
pub struct PugCode<'a> {
    pub flag: Token<'a>,
    pub code: Token<'a>,
    pub block: Vec<'a, PugNode<'a>>,
}

impl PugCode<'_> {
    /// `=` / `!=` output the value; `-` runs statements.
    pub fn buffered(&self) -> bool {
        self.flag.text != "-"
    }

    /// `=` escapes the value; `!=` does not.
    pub fn escaped(&self) -> bool {
        self.flag.text == "="
    }
}

/// `// text` (rendered) or `//- text` (dropped), with an optional body.
#[derive(Debug)]
pub struct PugComment<'a> {
    pub marker: Token<'a>,
    pub text: Token<'a>,
    pub body: Option<PugText<'a>>,
}

impl PugComment<'_> {
    pub fn buffered(&self) -> bool {
        self.marker.text == "//"
    }
}

/// A refused construct: its head line, any pipeless body, its block.
#[derive(Debug)]
pub struct PugRefused<'a> {
    pub refusal: PugRefusal,
    pub head: Token<'a>,
    pub body: Option<PugText<'a>>,
    pub block: Vec<'a, PugNode<'a>>,
}

/// Hole clause 2 at node level.
#[derive(Debug)]
pub struct PugUnexpected<'a> {
    pub token: Token<'a>,
    pub block: Vec<'a, PugNode<'a>>,
}
