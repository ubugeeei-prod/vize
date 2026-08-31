//! Event stream → surface tree, with the byte-coverage discipline that
//! makes `render == source` hold by construction.
//!
//! The builder walks the recorded events with a byte `cursor`. Every
//! token is built as `leading = [cursor, start)` + `text = [start, end)`
//! and advances the cursor, so the in-order token walk partitions the
//! source. The three ways stray bytes get a home are the crate-level
//! hole policy's clauses (see `lib.rs`): `Missing` tokens, `Unexpected`
//! children for uncovered gaps at children level, and `leading` for
//! uncovered gaps inside a tag.
//!
//! This root file holds the byte-coverage primitives and the
//! children-level nodes; tags live in [`tag`], attributes in [`attr`].

mod attr;
mod tag;

use vize_relief::Namespace;
use vize_s0::{Allocator, Box, Vec};

use crate::event::{Event, EventKind};
use crate::surface::{
    Element, ElementClose, Interpolation, OpenTag, SurfaceChild, SurfaceTree, Token,
};

/// v1 supports the tokenizer's default `{{` / `}}` delimiters; custom
/// delimiters (a `ParserOptions` concern) are recorded as deferred in the
/// P2-7 record.
const DELIM_OPEN_LEN: usize = 2;
const DELIM_CLOSE_LEN: usize = 2;

#[derive(Clone, Copy)]
enum RawKind {
    Comment,
    Cdata,
    Pi,
}

struct Frame<'a> {
    open: OpenTag<'a>,
    children: Vec<'a, SurfaceChild<'a>>,
    ns: Namespace,
}

impl<'a> Frame<'a> {
    fn tag(&self) -> &'a str {
        let text = self.open.lt_name.text;
        text.get(1..).unwrap_or(text)
    }
}

pub(crate) fn build<'a>(
    allocator: &'a Allocator,
    src: &'a str,
    events: &[Event],
) -> SurfaceTree<'a> {
    let mut b = Builder {
        src,
        allocator,
        events,
        i: 0,
        cursor: 0,
        root: Vec::new_in(&allocator),
        stack: Vec::new_in(&allocator),
        implicitly_closed_tags: Vec::new_in(&allocator),
    };
    b.run();
    // EOF: bytes the tokenizer consumed without reporting structure
    // (`</` at EOF, an unterminated bogus construct) become a typed
    // `Unexpected` hole at the innermost open level…
    b.flush_gap(src.len());
    // …then every still-open element gets its node-level `Missing` hole.
    while let Some(frame) = b.stack.pop() {
        let element = Element {
            open: frame.open,
            children: frame.children,
            close: ElementClose::Missing,
        };
        b.attach(element);
    }
    SurfaceTree {
        source: src,
        children: b.root,
    }
}

struct Builder<'a, 'e> {
    src: &'a str,
    allocator: &'a Allocator,
    events: &'e [Event],
    i: usize,
    cursor: usize,
    root: Vec<'a, SurfaceChild<'a>>,
    stack: Vec<'a, Frame<'a>>,
    implicitly_closed_tags: Vec<'a, &'a str>,
}

impl<'a> Builder<'a, '_> {
    fn run(&mut self) {
        while let Some(ev) = self.events.get(self.i).copied() {
            match ev.kind {
                EventKind::Text => self.text(ev),
                EventKind::Interpolation => self.interpolation(ev),
                EventKind::OpenTagName => self.element(ev),
                EventKind::CloseTag => self.close_tag(ev),
                EventKind::Comment => self.raw_node(ev, RawKind::Comment),
                EventKind::Cdata => self.raw_node(ev, RawKind::Cdata),
                EventKind::ProcessingInstruction => self.raw_node(ev, RawKind::Pi),
                // Attribute plumbing outside an open tag is a recovery
                // artifact; its bytes are covered by the gap rules.
                EventKind::OpenTagEnd
                | EventKind::SelfClosingTag
                | EventKind::AttrName
                | EventKind::AttrNameEnd
                | EventKind::AttrData
                | EventKind::AttrEnd => self.i += 1,
            }
        }
    }

    // ---- byte-coverage primitives --------------------------------------

    /// Token whose `leading` is the verbatim gap `[cursor, start)`.
    fn token_at(&mut self, start: usize, end: usize) -> Token<'a> {
        debug_assert!(self.cursor <= start && start <= end && end <= self.src.len());
        let leading = &self.src[self.cursor.min(start)..start];
        let text = &self.src[start..end];
        self.cursor = end;
        Token::present(leading, text)
    }

    /// A `Missing` hole at `leading_end`, absorbing `[cursor, leading_end)`
    /// as its leading (empty in the common case).
    fn missing_to(&mut self, leading_end: usize) -> Token<'a> {
        debug_assert!(self.cursor <= leading_end && leading_end <= self.src.len());
        let leading = &self.src[self.cursor.min(leading_end)..leading_end];
        self.cursor = self.cursor.max(leading_end);
        Token::missing(leading, &self.src[self.cursor..self.cursor])
    }

    /// Children-level coverage: an uncovered gap before `target` becomes
    /// a typed `Unexpected` node (hole policy clause 2).
    fn flush_gap(&mut self, target: usize) {
        debug_assert!(self.cursor <= target);
        if self.cursor < target {
            let text = &self.src[self.cursor..target];
            self.cursor = target;
            self.push_child(SurfaceChild::Unexpected(Token::present("", text)));
        }
    }

    fn find_byte(&self, byte: u8, from: usize, to: usize) -> Option<usize> {
        let bytes = self.src.as_bytes();
        bytes
            .get(from..to.min(bytes.len()))?
            .iter()
            .position(|b| *b == byte)
            .map(|p| from + p)
    }

    fn push_child(&mut self, child: SurfaceChild<'a>) {
        match self.stack.last_mut() {
            Some(frame) => frame.children.push(child),
            None => self.root.push(child),
        }
    }

    fn attach(&mut self, element: Element<'a>) {
        let boxed = Box::new_in(element, &self.allocator);
        self.push_child(SurfaceChild::Element(boxed));
    }

    // ---- children-level nodes ------------------------------------------

    fn text(&mut self, ev: Event) {
        let start = ev.start as usize;
        let mut end = ev.end as usize;
        self.i += 1;
        // Merge contiguous runs (the tokenizer splits at entities).
        while let Some(next) = self.events.get(self.i) {
            if next.kind == EventKind::Text && next.start as usize == end {
                end = next.end as usize;
                self.i += 1;
            } else {
                break;
            }
        }
        self.flush_gap(start);
        let token = self.token_at(start, end);
        self.push_child(SurfaceChild::Text(token));
    }

    fn interpolation(&mut self, ev: Event) {
        let (s, e) = (ev.start as usize, ev.end as usize);
        self.i += 1;
        let open_start = s.saturating_sub(DELIM_OPEN_LEN);
        self.flush_gap(open_start);
        let open = self.token_at(open_start, s);
        let content = self.token_at(s, e);
        // The tokenizer only reports an interpolation once the closing
        // delimiter matched, so it is always present.
        let close = self.token_at(e, (e + DELIM_CLOSE_LEN).min(self.src.len()));
        let node = Interpolation {
            open,
            content,
            close,
        };
        self.push_child(SurfaceChild::Interpolation(Box::new_in(
            node,
            &self.allocator,
        )));
    }

    /// Comments, CDATA and processing instructions: the event carries the
    /// content range; the framing before it starts exactly at the cursor,
    /// and the closer (if present in the source) is probed after it. One
    /// verbatim token covers the whole construct.
    fn raw_node(&mut self, ev: Event, kind: RawKind) {
        let e = ev.end as usize;
        self.i += 1;
        let tail = &self.src[e.min(self.src.len())..];
        let closer = match kind {
            RawKind::Comment => {
                if tail.starts_with("-->") {
                    3
                } else if tail.starts_with("--!>") {
                    4
                } else if tail.starts_with('>') {
                    1
                } else {
                    0
                }
            }
            RawKind::Cdata => {
                if tail.starts_with("]]>") {
                    3
                } else {
                    0
                }
            }
            RawKind::Pi => {
                if tail.starts_with('>') {
                    1
                } else {
                    0
                }
            }
        };
        let token = self.token_at(self.cursor, e + closer);
        self.push_child(match kind {
            RawKind::Comment => SurfaceChild::Comment(token),
            RawKind::Cdata => SurfaceChild::Cdata(token),
            RawKind::Pi => SurfaceChild::ProcessingInstruction(token),
        });
    }
}
