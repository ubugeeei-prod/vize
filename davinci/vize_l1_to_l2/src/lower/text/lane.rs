//! The text lane as plain data over one L1 child list: the owner's
//! whitespace rule, the condense plan, and what each text-family run
//! lowers to. The lowering records and mints from these results;
//! surface consumers read the same results without lowering.

use alloc::vec::Vec as StdVec;

use vize_l0::{Allocator, SourceBlock, Span, String};
use vize_l1::markup::{Element, SurfaceChild, Token};

use super::condense::plan_whitespace;
use super::leading_newline::normalize_special_text;
use super::{TextAction, TextPart};

mod scan;
use crate::lower::leaf::keeps_directive_comment;

/// The whitespace rule an owner gives its direct text children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextWhitespace {
    Normal,
    Pre,
    Rcdata,
}

/// The text rules an owner gives one child list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextLane {
    pub(crate) preserve_comments: bool,
    pub(crate) whitespace: TextWhitespace,
    /// Authored start of the first visible text child of an HTML pre/textarea.
    pub(crate) ignore_newline_at: Option<u32>,
    /// Inside a `<pre>` subtree.
    pub(crate) in_pre: bool,
    /// Inside a `v-pre` subtree.
    pub(crate) frozen: bool,
}

/// The arena and source block the text of one document lowers into.
#[derive(Clone, Copy)]
pub struct TextSource<'a> {
    pub(crate) allocator: &'a Allocator,
    pub(crate) block: SourceBlock<'a>,
}

impl<'a> TextSource<'a> {
    pub fn new(allocator: &'a Allocator, block: SourceBlock<'a>) -> Self {
        Self { allocator, block }
    }
}

/// A lane over its document: what every text computation reads.
#[derive(Clone, Copy)]
pub(crate) struct Lane<'a> {
    pub(crate) src: TextSource<'a>,
    pub(crate) rules: TextLane,
}

/// One child a run consumed without lowering it, in scan order.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Consumed<'a> {
    pub(crate) kind: &'static str,
    pub(crate) text: &'a str,
    pub(crate) span: Span,
}

/// What one text-family run lowers to.
#[derive(Debug)]
pub enum RunShape<'a> {
    /// A whitespace-only text the plan removes.
    Dropped,
    /// One text child as a leaf, with its rendered (entity-encoded) content.
    Text { content: &'a str, span: Span },
    /// One interpolation child as a leaf.
    Interpolation { span: Span },
    /// A merged run of `members` children: its parts tile `span`.
    Merged {
        parts: StdVec<TextPart>,
        span: Span,
        members: usize,
    },
    /// A `v-pre` run as one text, with its rendered content.
    Frozen { content: String, span: Span },
}

/// One run, and the index just past everything it consumed.
#[derive(Debug)]
pub struct TextRun<'a> {
    pub end: usize,
    pub shape: RunShape<'a>,
    pub(crate) consumed: StdVec<Consumed<'a>>,
}

/// Byte offset of `slice` inside `block`'s root source.
pub(crate) fn block_offset(block: SourceBlock<'_>, slice: &str) -> u32 {
    match block.offset_of(slice) {
        Some(offset) => offset,
        None => {
            debug_assert!(
                false,
                "L1 handed the lowering a string that is not a block slice"
            );
            block.start()
        }
    }
}

/// The span of `slice` inside `block`'s root source.
pub(crate) fn block_span(block: SourceBlock<'_>, slice: &str) -> Span {
    block.span_of(slice).unwrap_or_else(|| {
        let start = block_offset(block, slice);
        Span::new(start, start.saturating_add(slice.len() as u32))
    })
}

impl TextLane {
    /// The lane of a block's top-level children.
    pub const fn root(preserve_comments: bool) -> Self {
        Self {
            preserve_comments,
            whitespace: TextWhitespace::Normal,
            ignore_newline_at: None,
            in_pre: false,
            frozen: false,
        }
    }

    /// Whether the list is inside a `<pre>` subtree.
    pub const fn in_pre(self) -> bool {
        self.in_pre
    }

    /// The lane an element op gives its children: `tag` as authored,
    /// `html` when the element is in the HTML namespace, `opens_v_pre`
    /// when the element opens a `v-pre` subtree.
    pub fn children_of<'a>(
        self,
        src: TextSource<'a>,
        element: &Element<'a>,
        tag: &str,
        html: bool,
        opens_v_pre: bool,
    ) -> Self {
        Lane { src, rules: self }.children_of(element, tag, html, opens_v_pre)
    }

    /// The condense plan of one child list, one action per child.
    pub fn plan<'a>(
        self,
        src: TextSource<'a>,
        children: &[SurfaceChild<'a>],
    ) -> StdVec<TextAction<'a>> {
        Lane { src, rules: self }.plan(children)
    }

    /// The run starting at `children[start]`, a text or interpolation child.
    pub fn run<'a>(
        self,
        src: TextSource<'a>,
        children: &[SurfaceChild<'a>],
        plan: &[TextAction<'a>],
        start: usize,
    ) -> TextRun<'a> {
        Lane { src, rules: self }.run(children, plan, start)
    }
}

impl<'a> Lane<'a> {
    pub(crate) fn offset(&self, slice: &str) -> u32 {
        block_offset(self.src.block, slice)
    }

    pub(crate) fn token_span(&self, token: &Token<'_>) -> Span {
        block_span(self.src.block, token.text)
    }

    pub(crate) fn allocator(&self) -> &'a Allocator {
        self.src.allocator
    }

    pub(crate) fn preserve_comments(&self) -> bool {
        self.rules.preserve_comments
    }

    pub(crate) fn ignore_newline_at(&self) -> Option<u32> {
        self.rules.ignore_newline_at
    }

    pub(crate) fn condense_suppressed(&self) -> bool {
        self.rules.whitespace != TextWhitespace::Normal
    }

    pub(crate) fn normalize_pre_newlines(&self) -> bool {
        self.rules.whitespace == TextWhitespace::Pre
    }

    pub(crate) fn children_of(
        self,
        element: &Element<'a>,
        tag: &str,
        html: bool,
        opens_v_pre: bool,
    ) -> TextLane {
        let in_pre = self.rules.in_pre || super::suppresses_condense(tag);
        let rcdata = html && tag == "textarea";
        let whitespace = if rcdata {
            TextWhitespace::Rcdata
        } else if in_pre {
            TextWhitespace::Pre
        } else {
            TextWhitespace::Normal
        };
        let ignore_newline_at = if rcdata || (html && tag == "pre") {
            element
                .children
                .iter()
                .find(|child| match child {
                    SurfaceChild::Comment(token) => {
                        self.preserve_comments() || keeps_directive_comment(token.text)
                    }
                    _ => true,
                })
                .and_then(|child| match child {
                    SurfaceChild::Text(token) => Some(self.offset(token.text)),
                    _ => None,
                })
        } else {
            None
        };
        TextLane {
            whitespace,
            ignore_newline_at,
            in_pre,
            frozen: self.rules.frozen || opens_v_pre,
            ..self.rules
        }
    }

    pub(crate) fn plan(&self, children: &[SurfaceChild<'a>]) -> StdVec<TextAction<'a>> {
        plan_whitespace(self, children)
    }

    pub(crate) fn run(
        &self,
        children: &[SurfaceChild<'a>],
        plan: &[TextAction<'a>],
        start: usize,
    ) -> TextRun<'a> {
        if self.rules.frozen {
            self.frozen_run(children, plan, start)
        } else {
            self.text_run(children, plan, start)
        }
    }

    /// A lone child's rendered content and span.
    fn leaf(&self, child: &SurfaceChild<'a>, action: TextAction<'a>) -> RunShape<'a> {
        match child {
            SurfaceChild::Text(token) => {
                let mut content = match action {
                    TextAction::Content(content) => content,
                    _ => token.text,
                };
                if let Some(normalized) =
                    normalize_special_text(self, content, self.offset(token.text))
                {
                    content = self.allocator().alloc_str(normalized.as_str());
                }
                RunShape::Text {
                    content,
                    span: self.token_span(token),
                }
            }
            SurfaceChild::Interpolation(node) => RunShape::Interpolation {
                span: Span::new(
                    self.offset(node.open.text),
                    self.token_span(&node.close).end,
                ),
            },
            _ => RunShape::Dropped,
        }
    }
}
