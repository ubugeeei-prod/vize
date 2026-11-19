//! The authored structure of one style block, in every style language.
//!
//! Rules, at-rules, declarations and other statements form a tree of nodes in
//! document order, each with file-absolute spans; comments are kept beside
//! them. Two language-neutral readers build it: one for braced syntax (CSS,
//! SCSS, Less, braced Stylus) and one for indented syntax (Sass, Stylus). The
//! dialect modules in `lang/` supply the lexical rules through [`Lexis`].
//! Nothing here evaluates the source: interpolation, mixins and control flow
//! stay as written.

use alloc::vec::Vec;
use vize_l0::{Span, String};

mod braced;
mod classify;
mod indented;
mod lex;

/// One style block's nodes and comments over its authored source.
#[derive(Debug, Clone)]
pub struct StyleSheet<'a> {
    source: &'a str,
    base: u32,
    lexis: Lexis,
    nodes: Vec<StyleNode>,
    comments: Vec<StyleComment>,
    balanced: bool,
    terminated: bool,
}

/// A rule, at-rule, declaration or other statement. [`StyleSheet::kind`]
/// says which.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleNode {
    /// From the prelude to the end of the block or the terminator.
    span: Span,
    prelude_end: u32,
    /// The block ends with the node.
    block_start: Option<u32>,
    parent: Option<u32>,
    /// The first `:` outside strings, comments, interpolation and
    /// brackets, relative to the prelude.
    colon: Option<u32>,
    interpolated: bool,
}

/// One node per statement: keep the vector dense.
const _: () = assert!(core::mem::size_of::<StyleNode>() == 40);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleNodeKind {
    /// A selector and its block.
    Rule,
    /// `@name params`, with or without a block. `name` excludes the `@`; in
    /// Sass, `=` and `+` spell `@mixin` and `@include` and are the name.
    AtRule { name: Span },
    /// `name: value`, with or without a nested block (`font: { … }`). Less
    /// variables (`@gap: 4px`) are declarations; Stylus may omit the colon.
    Declaration { name: Span, value: Span },
    /// Anything else: mixin calls, `&:extend()`, Stylus control flow,
    /// assignments and function definitions.
    Other,
}

/// A comment: `/* … */`, or a line comment (`// …`) where the language has them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleComment {
    span: Span,
    line: bool,
}

impl<'a> StyleSheet<'a> {
    /// The block source the spans point into.
    #[must_use]
    pub const fn source(&self) -> &'a str {
        self.source
    }

    /// Nodes in document order; a parent precedes its children.
    #[must_use]
    pub fn nodes(&self) -> &[StyleNode] {
        &self.nodes
    }

    /// Comments in document order.
    #[must_use]
    pub fn comments(&self) -> &[StyleComment] {
        &self.comments
    }

    /// Every block that opened was closed and no stray close appeared.
    /// Open blocks still end at the end of the source.
    #[must_use]
    pub const fn is_balanced(&self) -> bool {
        self.balanced
    }

    /// Every string, comment and interpolation was closed.
    #[must_use]
    pub const fn is_terminated(&self) -> bool {
        self.terminated
    }

    /// What a node is, read from its prelude.
    #[must_use]
    pub fn kind(&self, node: &StyleNode) -> StyleNodeKind {
        let text = self.text(node.prelude());
        let colon = node.colon.map(|colon| colon as usize);
        let start = node.span.start;
        let span = |(from, to): (usize, usize)| Span::new(start + from as u32, start + to as u32);
        match classify::classify(text, colon, node.block_start.is_some(), &self.lexis) {
            classify::Shape::Rule => StyleNodeKind::Rule,
            classify::Shape::AtRule { name } => StyleNodeKind::AtRule { name: span(name) },
            classify::Shape::Declaration { name, value } => StyleNodeKind::Declaration {
                name: span(name),
                value: span(value),
            },
            classify::Shape::Other => StyleNodeKind::Other,
        }
    }

    /// The authored text of a span inside this block.
    #[must_use]
    pub fn text(&self, span: Span) -> &'a str {
        let start = span.start.saturating_sub(self.base) as usize;
        let end = span.end.saturating_sub(self.base) as usize;
        self.source.get(start..end).unwrap_or_default()
    }

    /// The text of a span with the comments inside it removed.
    #[must_use]
    pub fn code(&self, span: Span) -> StyleCode<'a> {
        let first = self
            .comments
            .partition_point(|comment| comment.span.end <= span.start);
        let mut inside = self
            .comments
            .get(first..)
            .unwrap_or_default()
            .iter()
            .take_while(|comment| comment.span.start < span.end)
            .peekable();
        if inside.peek().is_none() {
            return StyleCode::Borrowed(self.text(span));
        }
        let mut code = String::default();
        let mut cursor = span.start;
        for comment in inside {
            code.push_str(self.text(Span::new(cursor, comment.span.start.max(cursor))));
            cursor = comment.span.end.min(span.end);
        }
        code.push_str(self.text(Span::new(cursor, span.end)));
        StyleCode::Owned(code)
    }
}

/// Authored text with comments removed; borrowed when there were none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleCode<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl core::ops::Deref for StyleCode<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::Owned(text) => text,
        }
    }
}

impl StyleNode {
    /// From the prelude to the end of the block or the terminator.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// The selector, the at-rule, or the whole declaration or statement,
    /// without surrounding whitespace and comments.
    #[must_use]
    pub const fn prelude(&self) -> Span {
        Span::new(self.span.start, self.prelude_end)
    }

    /// The block: braces included for braced syntax, the indented lines for
    /// indented syntax.
    #[must_use]
    pub const fn block(&self) -> Option<Span> {
        match self.block_start {
            Some(start) => Some(Span::new(start, self.span.end)),
            None => None,
        }
    }

    /// The index of the enclosing node in [`StyleSheet::nodes`].
    #[must_use]
    pub fn parent(&self) -> Option<usize> {
        self.parent.map(|parent| parent as usize)
    }

    /// The prelude interpolates outside strings (`#{…}`, `@{…}`, Stylus
    /// `{…}`), so its selector or name is not what the preprocessor emits.
    #[must_use]
    pub const fn is_interpolated(&self) -> bool {
        self.interpolated
    }
}

impl StyleComment {
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// A `//` comment rather than `/* … */`.
    #[must_use]
    pub const fn is_line(&self) -> bool {
        self.line
    }
}

/// The lexical rules one style language supplies to the readers.
#[derive(Debug, Clone, Copy)]
pub(super) struct Lexis {
    /// `//` starts a comment that runs to the end of the line.
    pub line_comments: bool,
    pub interpolation: Interpolation,
    /// Blocks open by indentation rather than braces.
    pub indented: bool,
    /// A comment that starts a line also covers the deeper-indented lines
    /// after it, and `/*` needs no `*/` (Sass).
    pub indented_comments: bool,
    /// Declarations may omit the colon (Stylus `color red`).
    pub colonless_declarations: bool,
    /// Dialect statements the shared grammar does not know.
    pub construct: fn(&str) -> Construct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Interpolation {
    None,
    /// `#{…}` (Sass, SCSS); strings interpolate too.
    Hash,
    /// `@{…}` (Less); strings interpolate too.
    At,
    /// `{…}` that cannot open a block: after a name in braced syntax,
    /// anywhere in indented syntax (Stylus).
    Brace,
}

/// How a dialect reads a statement before the shared grammar does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Construct {
    /// Shared grammar: at-rule, declaration, rule or other.
    Shared,
    /// An at-rule spelled with a one-byte sigil (Sass `=` and `+`).
    Sigil,
    /// A dialect statement that is not CSS (Stylus control flow, calls).
    Other,
}

pub(super) const fn shared(_: &str) -> Construct {
    Construct::Shared
}

impl<'a> StyleSheet<'a> {
    /// Read `source`, which starts at file offset `base`, with one language's
    /// lexical rules.
    pub(super) fn read(source: &'a str, base: u32, lexis: &Lexis) -> Self {
        let mut builder = Builder {
            sheet: StyleSheet {
                source,
                base,
                lexis: *lexis,
                nodes: Vec::new(),
                comments: Vec::new(),
                balanced: true,
                terminated: true,
            },
            open: None,
        };
        if lexis.indented {
            indented::read(&mut builder);
        } else {
            braced::read(&mut builder);
        }
        builder.finish()
    }
}

/// A prelude as a reader found it: code bytes `start..end`, the first `:`
/// outside strings, comments, interpolation and brackets, and whether it
/// interpolates. Offsets are block-relative.
#[derive(Clone, Copy)]
struct Prelude {
    start: usize,
    end: usize,
    colon: Option<usize>,
    interpolated: bool,
}

impl Prelude {
    const fn new(start: usize) -> Self {
        Self {
            start,
            end: start,
            colon: None,
            interpolated: false,
        }
    }
}

/// Collects nodes as a reader finds them. Offsets are block-relative.
struct Builder<'a> {
    sheet: StyleSheet<'a>,
    /// The innermost open block; closing it reopens its parent.
    open: Option<u32>,
}

impl<'a> Builder<'a> {
    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.sheet.base + start as u32, self.sheet.base + end as u32)
    }

    fn bytes(&self) -> &'a [u8] {
        self.sheet.source.as_bytes()
    }

    fn comment(&mut self, start: usize, end: usize, line: bool) {
        let span = self.span(start, end);
        self.sheet.comments.push(StyleComment { span, line });
    }

    fn unterminated(&mut self) {
        self.sheet.terminated = false;
    }

    fn unbalanced(&mut self) {
        self.sheet.balanced = false;
    }

    /// A prelude that opens a block at `block_start`.
    fn open(&mut self, prelude: Prelude, block_start: usize) {
        let index = self.sheet.nodes.len() as u32;
        self.push(prelude, Some(block_start));
        self.open = Some(index);
    }

    /// Close the innermost open block at `end`; a close without one is stray.
    fn close(&mut self, end: usize) {
        let span_end = self.sheet.base + end as u32;
        match self
            .open
            .and_then(|index| self.sheet.nodes.get_mut(index as usize))
        {
            Some(node) => {
                node.span.end = span_end;
                self.open = node.parent;
            }
            None => self.unbalanced(),
        }
    }

    /// A statement whose terminator ends at `terminated`.
    fn statement(&mut self, prelude: Prelude, terminated: usize) {
        self.push(prelude, None);
        if let Some(node) = self.sheet.nodes.last_mut() {
            node.span.end = self.sheet.base + terminated as u32;
        }
    }

    fn push(&mut self, prelude: Prelude, block: Option<usize>) {
        let span = self.span(prelude.start, prelude.end);
        let node = StyleNode {
            span,
            prelude_end: span.end,
            block_start: block.map(|block| self.sheet.base + block as u32),
            parent: self.open,
            colon: prelude.colon.map(|colon| (colon - prelude.start) as u32),
            interpolated: prelude.interpolated,
        };
        self.sheet.nodes.push(node);
    }

    fn finish(mut self) -> StyleSheet<'a> {
        if self.open.is_some() {
            let end = self.sheet.source.len();
            while self.open.is_some() {
                self.close(end);
            }
            self.unbalanced();
        }
        self.sheet
    }
}

#[cfg(test)]
mod tests;
