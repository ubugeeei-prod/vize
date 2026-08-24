//! Open and close tags: the element half of the builder.

use vize_s0::{Vec, is_void_tag};

use super::{Builder, Frame};
use crate::event::{Event, EventKind};
use crate::surface::{Attribute, CloseTag, Element, ElementClose, OpenTag, SurfaceChild, Token};

impl<'a> Builder<'a, '_> {
    pub(super) fn element(&mut self, ev: Event) {
        let (name_s, name_e) = (ev.start as usize, ev.end as usize);
        self.i += 1;
        let lt = name_s.saturating_sub(1);
        self.flush_gap(lt);
        let lt_name = self.token_at(lt, name_e);
        let tag = &self.src[name_s..name_e];
        let mut attrs: Vec<'a, Attribute<'a>> = Vec::new_in(&self.allocator);
        let mut slash = None;
        let gt;
        loop {
            match self.events.get(self.i).map(|next| next.kind) {
                Some(EventKind::AttrName) => {
                    let attr = self.attribute();
                    attrs.push(attr);
                }
                Some(EventKind::OpenTagEnd) => {
                    let idx = self.events[self.i].start as usize;
                    self.i += 1;
                    gt = self.open_gt(idx);
                    break;
                }
                Some(EventKind::SelfClosingTag) => {
                    let idx = self.events[self.i].start as usize;
                    self.i += 1;
                    let (found_slash, found_gt) = self.self_closing(idx);
                    slash = found_slash;
                    gt = found_gt;
                    break;
                }
                // Stray value plumbing without a name: covered by leading.
                Some(EventKind::AttrNameEnd | EventKind::AttrData | EventKind::AttrEnd) => {
                    self.i += 1;
                }
                // The tokenizer always terminates a tag (EOF recovery
                // included); reaching here means it did not — close with
                // a zero-width hole and reprocess the event outside.
                _ => {
                    gt = self.missing_to(self.cursor);
                    break;
                }
            }
        }
        let self_closing = slash.is_some();
        let open = OpenTag {
            lt_name,
            attrs,
            slash,
            gt,
        };
        if self_closing || is_void_tag(tag) {
            let element = Element {
                open,
                children: Vec::new_in(&self.allocator),
                close: ElementClose::NotExpected,
            };
            self.attach(element);
        } else {
            self.stack.push(Frame {
                open,
                children: Vec::new_in(&self.allocator),
            });
        }
    }

    /// The open tag's `>`. A non-`>` index only ever comes from the
    /// tokenizer's EOF-in-tag recovery (the inferred index), so the hole
    /// absorbs the remaining tag bytes as leading.
    fn open_gt(&mut self, idx: usize) -> Token<'a> {
        let bytes = self.src.as_bytes();
        if idx >= self.cursor && bytes.get(idx) == Some(&b'>') {
            self.token_at(idx, idx + 1)
        } else {
            self.missing_to(self.src.len())
        }
    }

    /// `/` then `>`, whitespace between them kept as the `>`'s leading.
    fn self_closing(&mut self, idx: usize) -> (Option<Token<'a>>, Token<'a>) {
        let slash_pos = self.find_byte(b'/', self.cursor, idx);
        let slash = slash_pos.map(|p| self.token_at(p, p + 1));
        let gt = self.token_at(idx, idx + 1);
        (slash, gt)
    }

    pub(super) fn close_tag(&mut self, ev: Event) {
        let (s, e) = (ev.start as usize, ev.end as usize);
        self.i += 1;
        let lt_start = self.close_lt_start(s);
        let gt_pos = self.find_byte(b'>', e, self.src.len());
        let name = &self.src[s..e];
        self.flush_gap(lt_start);
        let matched = self
            .stack
            .iter()
            .rposition(|frame| frame.tag().eq_ignore_ascii_case(name));
        let Some(depth) = matched else {
            // Stray end tag: a typed `Unexpected` hole, whole extent.
            let end = gt_pos.map_or(self.src.len(), |g| g + 1);
            let token = self.token_at(lt_start, end);
            self.push_child(SurfaceChild::Unexpected(token));
            return;
        };
        // Elements left open above the match get node-level holes.
        while self.stack.len() > depth + 1 {
            let frame = self.stack.pop().expect("stack holds depth + 1 frames");
            let element = Element {
                open: frame.open,
                children: frame.children,
                close: ElementClose::Missing,
            };
            self.attach(element);
        }
        let lt_slash_name = self.token_at(lt_start, e);
        let gt = match gt_pos {
            // `AfterClosingTagName` only exits at `>`, so junk between
            // the name and the `>` rides in the `>`'s leading.
            Some(g) => self.token_at(g, g + 1),
            None => self.missing_to(self.src.len()),
        };
        let frame = self.stack.pop().expect("matched frame exists");
        let element = Element {
            open: frame.open,
            children: frame.children,
            close: ElementClose::Present(CloseTag { lt_slash_name, gt }),
        };
        self.attach(element);
    }

    /// Back-scan from an end-tag name to its `</`, tolerating the
    /// whitespace forms the tokenizer accepts (`</ div`, `</div `).
    fn close_lt_start(&self, s: usize) -> usize {
        let bytes = self.src.as_bytes();
        let mut p = s;
        while p > self.cursor && bytes[p - 1].is_ascii_whitespace() {
            p -= 1;
        }
        if p > self.cursor && bytes[p - 1] == b'/' {
            p -= 1;
        }
        if p > self.cursor && bytes[p - 1] == b'<' {
            p - 1
        } else {
            self.cursor
        }
    }
}
