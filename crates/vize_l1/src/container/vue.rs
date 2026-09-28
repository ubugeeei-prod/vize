//! The Vue SFC container.
//!
//! Block boundaries are found in one byte pass: script blocks skip strings,
//! comments and regex literals; the root `<template>` tracks nesting while
//! skipping comments, raw-text elements and interpolations. The legacy SFC
//! descriptor (`vize_croquis`) retains its own scanner until #6880 closes.

mod attrs;
mod block;
mod template_boundary;

use alloc::borrow::Cow;

use memchr::memmem::Finder;
use vize_l0::{Allocator, Span, Vec};

pub use block::{
    AttrSink, BlockParseError, BlockParseOutput, BlockParseResult, parse_block_fast, tag_name_eq,
};

use super::{Block, BlockAttr, Container, ContainerError, ContainerErrorCode, ContainerFormat};

/// Vue single-file components: `<template>`, `<script>`, `<script setup>`,
/// `<style>` and custom blocks. Only the root `<template>` nests same-named
/// tags; every other block ends at the first matching close tag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vue;

impl ContainerFormat for Vue {
    fn split<'a>(&self, allocator: &'a Allocator, source: &'a str) -> Container<'a> {
        let mut container = Container {
            source,
            blocks: Vec::new_in(&allocator),
            errors: Vec::new_in(&allocator),
        };
        let bytes = source.as_bytes();
        let comment_end = Finder::new(b"-->");
        let mut seen = Seen::default();
        let mut pos = 0;
        while let Some(offset) = bytes.get(pos..).and_then(|rest| memchr::memchr(b'<', rest)) {
            pos += offset;
            if bytes
                .get(pos..)
                .is_some_and(|rest| rest.starts_with(b"<!--"))
            {
                let body = bytes.get(pos + 4..).unwrap_or_default();
                pos = comment_end
                    .find(body)
                    .map_or(bytes.len(), |end| pos + 4 + end + 3);
                continue;
            }
            let mut attrs = Attrs {
                list: Vec::new_in(&allocator),
            };
            match parse_block_fast(bytes, source, pos, 1, 1, &mut attrs) {
                Ok(Some(raw)) => {
                    let (name, _, content_start, content_end, end, _, _) = raw;
                    let name = source
                        .get(pos + 1..pos + 1 + name.len())
                        .unwrap_or_default();
                    if seen.duplicate(name, &attrs.list) {
                        container.errors.push(ContainerError {
                            code: ContainerErrorCode::DuplicateBlock,
                            offset: offset_u32(pos),
                        });
                    }
                    let is_void_custom = !is_known(name) && vize_l0::is_void_tag(name);
                    if !is_void_custom {
                        container.blocks.push(block(
                            name,
                            attrs.list,
                            [pos, content_start, content_end, end],
                        ));
                    }
                    pos = end;
                }
                Ok(None) => pos += 1,
                Err((code, _)) => {
                    container.errors.push(ContainerError {
                        code: if code == "UNTERMINATED_OPEN_TAG" {
                            ContainerErrorCode::UnterminatedOpenTag
                        } else {
                            ContainerErrorCode::MissingCloseTag
                        },
                        offset: offset_u32(pos),
                    });
                    break;
                }
            }
        }
        container
    }
}

fn is_known(name: &str) -> bool {
    ["template", "script", "style"]
        .iter()
        .any(|known| name.eq_ignore_ascii_case(known))
}

#[derive(Default)]
struct Seen {
    template: bool,
    script: bool,
    script_setup: bool,
}

impl Seen {
    /// Record a block; `true` when the format allows only one of it.
    fn duplicate(&mut self, name: &str, attrs: &[BlockAttr<'_>]) -> bool {
        let slot = if name.eq_ignore_ascii_case("template") {
            &mut self.template
        } else if name.eq_ignore_ascii_case("script") {
            if attrs.iter().any(|attr| attr.name == "setup") {
                &mut self.script_setup
            } else {
                &mut self.script
            }
        } else {
            return false;
        };
        core::mem::replace(slot, true)
    }
}

fn offset_u32(offset: usize) -> u32 {
    u32::try_from(offset).unwrap_or(u32::MAX)
}

fn span(start: usize, end: usize) -> Span {
    Span::new(offset_u32(start), offset_u32(end))
}

/// `[tag_start, content_start, content_end, tag_end]`.
fn block<'a>(
    name: &'a str,
    attrs: Vec<'a, BlockAttr<'a>>,
    [tag_start, content_start, content_end, tag_end]: [usize; 4],
) -> Block<'a> {
    let self_closing = content_start == tag_end;
    Block {
        name,
        open_tag: span(tag_start, content_start),
        attrs,
        content: span(content_start, content_end),
        close_tag: (!self_closing).then(|| span(content_end, tag_end)),
    }
}

/// Collects attributes as source slices, in order, duplicates kept.
struct Attrs<'a> {
    list: Vec<'a, BlockAttr<'a>>,
}

impl<'a> AttrSink<'a> for Attrs<'a> {
    fn attr(
        &mut self,
        name: Cow<'a, str>,
        value: Option<Cow<'a, str>>,
        (start, end): (usize, usize),
    ) {
        // The block scanner only ever borrows from the source.
        let Cow::Borrowed(name) = name else {
            return;
        };
        let value = match value {
            Some(Cow::Borrowed(value)) => Some(value),
            Some(Cow::Owned(_)) => return,
            None => None,
        };
        self.list.push(BlockAttr {
            name,
            value,
            span: span(start, end),
        });
    }
}

#[cfg(test)]
mod tests;
