//! The S1 page (`s1-page@1`): the lossless surface tree as a folio page.
//!
//! Every S1 string is a slice of the block source, and in canonical render
//! order the slices tile the source exactly (the S1 fidelity law). The page
//! therefore records **structure and offsets only**: each token is three
//! block-relative offsets `start:text:end`, where `start..text` is the
//! token's verbatim `leading` gap and `text..end` its own bytes. A reader
//! holding the block source rebuilds the tree ([`SurfacePage::materialize`])
//! after checking that the tokens tile it ([`SurfacePage::check_tiles`]), so
//! a guest can neither drop nor invent a byte.
//!
//! The grammar is documented in `docs/davinci/plan/folio-format.md` ("S1
//! page"). `Display` prints the same text as `Full`: nothing is elidable
//! from a page that is only offsets.

use vize_davinci::folio::{Folio, FolioError, FolioMode};

mod mirror;
mod parse;
mod print;
mod tile;

pub use tile::TileError;

/// One token: `start..text` is the `leading` gap, `text..end` the token's
/// own bytes; a `missing` token is a typed hole with `text == end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageToken {
    pub start: u32,
    pub text: u32,
    pub end: u32,
    pub missing: bool,
}

/// A child at any children level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNode {
    Element(PageElement),
    Text(PageToken),
    Interpolation(PageInterpolation),
    Comment(PageToken),
    Cdata(PageToken),
    ProcessingInstruction(PageToken),
    Unexpected(PageToken),
}

/// `{{ expr }}` as three tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageInterpolation {
    pub open: PageToken,
    pub content: PageToken,
    pub close: PageToken,
}

/// An element: open tag, children, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageElement {
    pub lt_name: PageToken,
    pub attrs: Vec<PageAttribute>,
    pub slash: Option<PageToken>,
    pub gt: PageToken,
    pub children: Vec<PageNode>,
    pub close: PageClose,
}

/// An attribute or raw directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageAttribute {
    pub name: PageToken,
    pub eq: Option<PageToken>,
    pub value: Option<PageAttrValue>,
}

/// An attribute value with its quotes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageAttrValue {
    pub open_quote: Option<PageToken>,
    pub content: PageToken,
    pub close_quote: Option<PageToken>,
}

/// How an element's extent ended (`vize_s1::ElementClose`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageClose {
    Tag {
        lt_slash_name: PageToken,
        gt: PageToken,
    },
    Implicit,
    Missing,
    NotExpected,
}

/// Document model of one S1 surface tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurfacePage {
    /// The root children level.
    pub children: Vec<PageNode>,
}

impl SurfacePage {
    /// The rendered byte length the page claims: the last token's end.
    #[must_use]
    pub fn bytes(&self) -> u32 {
        let mut end = 0;
        self.for_each_token(&mut |_, token| end = token.end);
        end
    }

    /// Visit every token in canonical render order with its role name.
    pub fn for_each_token(&self, visit: &mut dyn FnMut(&'static str, &PageToken)) {
        tile::walk_children(&self.children, visit);
    }
}

impl Folio for SurfacePage {
    fn print<W: core::fmt::Write>(&self, w: &mut W, _mode: FolioMode) -> core::fmt::Result {
        print::print(self, w)
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        parse::parse(input)
    }
}
