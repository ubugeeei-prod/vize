//! Open and close tags: the element half of the builder.

use vize_l0::{Box, Vec, is_void_tag};

use super::{Builder, Frame};
use crate::event::{Event, EventKind};
use crate::markup::surface::{
    Attribute, CloseTag, Element, ElementClose, OpenTag, SurfaceChild, Token,
};

impl<'a, const RAW: bool, const LINT: bool> Builder<'a, '_, RAW, LINT> {
    pub(super) fn element(&mut self, ev: Event) {
        let (name_s, name_e) = (ev.start as usize, ev.end as usize);
        self.i += 1;
        let lt = name_s.saturating_sub(1);
        self.flush_gap(lt);
        let mut lt_name = self.token_at(lt, name_e);
        let tag = crate::slice::range(self.src, name_s, name_e);
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
                    let idx = self.events.get(self.i).map_or(self.cursor, |event| {
                        lt_name.mark_verbatim_opening(event.is_verbatim_opening());
                        if LINT {
                            lt_name.mark_lint_tag(
                                event.lint_tag(),
                                event.lint_header_is_literal(),
                                event.lint_in_table_context(),
                                event.lint_in_recovery_context(),
                            );
                        }
                        event.start as usize
                    });
                    self.i += 1;
                    gt = self.open_gt(idx);
                    break;
                }
                Some(EventKind::SelfClosingTag) => {
                    let idx = self.events.get(self.i).map_or(self.cursor, |event| {
                        lt_name.mark_verbatim_opening(event.is_verbatim_opening());
                        if LINT {
                            lt_name.mark_lint_tag(
                                event.lint_tag(),
                                event.lint_header_is_literal(),
                                event.lint_in_table_context(),
                                event.lint_in_recovery_context(),
                            );
                        }
                        event.start as usize
                    });
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
        let inherited_v_pre = self.stack.last().is_some_and(|frame| frame.in_v_pre);
        let in_v_pre = inherited_v_pre || attrs.iter().any(|attr| attr.name.text == "v-pre");
        for (index, attr) in attrs.iter().enumerate() {
            if (!is_directive(attr.name.text) || inherited_v_pre)
                && attrs
                    .get(..index)
                    .unwrap_or_default()
                    .iter()
                    .any(|existing| {
                        (!is_directive(existing.name.text) || inherited_v_pre)
                            && existing.name.text.eq_ignore_ascii_case(attr.name.text)
                    })
            {
                let start = self.source_offset(attr.name.text);
                self.errors
                    .push(crate::markup::SurfaceError::duplicate_attribute(
                        vize_l0::Span::new(
                            start,
                            start.saturating_add(attr.name.text.len() as u32),
                        ),
                    ));
            }
        }
        // The lint-header provider already records unsupported directive heads
        // in its typed `unsupported` channel. Keep that selected-owner contract
        // separate; the ordinary surface path owns parser/template diagnostics.
        if !LINT && !inherited_v_pre {
            for attr in &attrs {
                self.directive_errors(attr.name.text);
            }
        }
        let ignored_start = self.recovery.is_repeated_form(&self.stack, tag);
        let (ns, implicit_depth) = self.recovery.open(&self.stack, tag, self_closing);
        let table_tree_element = is_table_tree_element(tag, in_v_pre, &attrs);
        let open = OpenTag {
            lt_name,
            attrs,
            slash,
            gt,
        };
        if ignored_start {
            let span = self.open_span(&open);
            self.errors
                .push(crate::markup::SurfaceError::ignored_repeated_form(span));
        }
        if !self_closing
            && !is_void_tag(tag)
            && self
                .recovery
                .should_foster_start(&self.stack, tag, ns, table_tree_element)
        {
            let span = self.open_span(&open);
            self.errors
                .push(crate::markup::SurfaceError::foster_parenting(span));
        }
        if !ignored_start && !self_closing && !is_void_tag(tag) && self.stack.len() == 4096 {
            let span = self.open_span(&open);
            self.errors
                .push(crate::markup::SurfaceError::nesting_too_deep(span));
        }
        if let Some(depth) = implicit_depth {
            self.repaired = true;
            self.implicitly_close_stack_element_at(depth);
        }

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
                ns,
                ignored_start,
                in_v_pre,
                table_tree_element,
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
        let name = crate::slice::range(self.src, s, e);
        self.flush_gap(lt_start);
        let adoption_depth = self.recovery.adoption(&self.stack, name);
        let Some(depth) = self.recovery.close(&self.stack, name) else {
            if self.recovery.take_consumed_implicit_close() {
                let end = gt_pos.map_or(self.src.len(), |g| g + 1);
                let token = self.token_at(lt_start, end);
                self.push_child(SurfaceChild::Unexpected(token));
                return;
            }
            if self.recovery.consume_reopened_formatting(name) {
                let end = gt_pos.map_or(self.src.len(), |g| g + 1);
                let token = self.token_at(lt_start, end);
                self.push_child(SurfaceChild::Unexpected(token));
                return;
            }
            // Stray end tag: a typed `Unexpected` hole, whole extent.
            self.unexpected_close_tag(lt_start, gt_pos);
            return;
        };
        if adoption_depth.is_some() {
            let end = gt_pos.map_or(self.src.len(), |g| g + 1);
            self.errors
                .push(crate::markup::SurfaceError::adoption_agency(
                    vize_l0::Span::new(lt_start as u32, end as u32),
                ));
        }
        // Elements left open above the match get node-level holes.
        while self.stack.len() > depth + 1 {
            let Some(frame) = self.stack.pop() else {
                break;
            };
            if adoption_depth.is_none() {
                self.missing_end_tag(&frame);
            }
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
        // `matched` found the frame at `depth`, the top after the pops above.
        let Some(frame) = self.stack.pop() else {
            return;
        };
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
        let before = |p: usize| {
            (p > self.cursor)
                .then(|| bytes.get(p - 1).copied())
                .flatten()
        };
        let mut p = s;
        while before(p).is_some_and(|byte| byte.is_ascii_whitespace()) {
            p -= 1;
        }
        if before(p) == Some(b'/') {
            p -= 1;
        }
        if before(p) == Some(b'<') {
            p - 1
        } else {
            self.cursor
        }
    }

    fn unexpected_close_tag(&mut self, lt_start: usize, gt_pos: Option<usize>) {
        let end = gt_pos.map_or(self.src.len(), |g| g + 1);
        self.errors.push(crate::markup::SurfaceError::at_encounter(
            vize_l0::ErrorCode::InvalidEndTag,
            vize_l0::Span::new(lt_start as u32, end as u32),
            end as u32,
        ));
        let token = self.token_at(lt_start, end);
        self.push_child(SurfaceChild::Unexpected(token));
    }

    fn directive_errors(&mut self, name: &str) {
        let start = self.source_offset(name);
        if name
            .strip_prefix("v-")
            .is_some_and(|tail| tail.is_empty() || tail.starts_with(['.', ':']))
        {
            self.errors
                .push(crate::markup::SurfaceError::missing_directive_name(
                    vize_l0::Span::new(start, start.saturating_add(name.len() as u32)),
                ));
            return;
        }
        if !is_directive(name) {
            return;
        }

        let mut dynamic_depth = 0_u32;
        let mut modifier_start = None;
        for (index, byte) in name.bytes().enumerate() {
            match byte {
                b'[' => dynamic_depth = dynamic_depth.saturating_add(1),
                b']' => dynamic_depth = dynamic_depth.saturating_sub(1),
                b'.' if dynamic_depth == 0 && !(index == 0 && name.starts_with('.')) => {
                    if modifier_start == Some(index) {
                        let offset = start.saturating_add(index as u32);
                        self.errors.push(crate::markup::SurfaceError::at(
                            vize_l0::ErrorCode::MissingDirectiveModifier,
                            vize_l0::Span::new(offset, offset),
                        ));
                    }
                    modifier_start = Some(index + 1);
                }
                _ => {}
            }
        }
        if modifier_start == Some(name.len()) {
            let end = start.saturating_add(name.len() as u32);
            self.errors.push(crate::markup::SurfaceError::at(
                vize_l0::ErrorCode::MissingDirectiveModifier,
                vize_l0::Span::new(end, end),
            ));
        }
    }

    fn implicitly_close_stack_element_at(&mut self, depth: usize) {
        let mut child = None;
        while self.stack.len() > depth {
            let Some(frame) = self.stack.pop() else {
                break;
            };
            let mut children = frame.children;
            if let Some(element) = child.take() {
                children.push(SurfaceChild::Element(Box::new_in(element, &self.allocator)));
            }
            child = Some(Element {
                open: frame.open,
                children,
                close: ElementClose::Implicit,
            });
        }
        if let Some(element) = child {
            self.attach(element);
        }
    }
}

fn is_directive(name: &str) -> bool {
    matches!(name.as_bytes().first(), Some(b':' | b'@' | b'#' | b'.')) || name.starts_with("v-")
}

fn is_table_tree_element(tag: &str, in_v_pre: bool, attrs: &[Attribute<'_>]) -> bool {
    if in_v_pre {
        return true;
    }
    if tag == "slot"
        || tag == "component"
        || tag.chars().next().is_some_and(char::is_uppercase)
        || matches!(
            tag,
            "Teleport"
                | "Suspense"
                | "KeepAlive"
                | "BaseTransition"
                | "Transition"
                | "TransitionGroup"
        )
    {
        return false;
    }
    if tag != "template" {
        return true;
    }
    !attrs.iter().any(|attr| {
        matches!(
            attr.name.text,
            "v-if" | "v-else-if" | "v-else" | "v-for" | "v-slot"
        ) || attr.name.text.starts_with('#')
    })
}
