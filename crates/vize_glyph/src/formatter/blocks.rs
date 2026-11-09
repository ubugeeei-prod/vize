//! Top-level SFC blocks from the L1 container.
//!
//! L1 owns every opening tag, attribute, raw-block boundary and location. A
//! lone template takes L1's allocation-free template-only observation; every
//! other file takes the complete container walk. Glyph keeps only its block
//! policy: one template, one script and one script setup; top-level void tags
//! stay root content.

use std::borrow::Cow;
use vize_l0::{FxHashMap, String, is_void_tag};
use vize_l1::container::Vue;
use vize_l1::container::vue::TemplateOnlyDescriptor;
use vize_l1::container::vue::compat::{CompatBlock, split_blocks, tag_name_eq};

use crate::error::FormatError;

pub(super) type BlockAttrs<'a> = FxHashMap<Cow<'a, str>, Cow<'a, str>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockKind {
    Script,
    ScriptSetup,
    Template,
    Style,
    Custom,
}

/// Byte offsets of one block: `start..end` is its content and
/// `tag_start..tag_end` the whole block, opening `<` through closing `>`.
#[derive(Debug, Clone, Copy)]
pub(super) struct BlockLocation {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) tag_start: usize,
    pub(super) tag_end: usize,
}

pub(super) struct SfcBlock<'a> {
    pub(super) kind: BlockKind,
    pub(super) name: &'a str,
    pub(super) attrs: BlockAttrs<'a>,
    pub(super) content: Cow<'a, str>,
    pub(super) loc: BlockLocation,
}

impl SfcBlock<'_> {
    pub(super) fn lang(&self) -> Option<&str> {
        self.attrs.get("lang").map(Cow::as_ref)
    }

    pub(super) fn has_attr(&self, name: &str) -> bool {
        self.attrs.contains_key(name)
    }
}

/// Split `source` into its blocks in source order.
pub(super) fn split(source: &str) -> Result<Vec<SfcBlock<'_>>, FormatError> {
    if let Some(template) = Vue.observe_template_only(source) {
        return Ok(vec![template_only(&template)]);
    }
    let mut blocks = Vec::new();
    let (mut template, mut script, mut setup) = (false, false, false);
    let mut duplicate = None;
    split_blocks(source, |block: CompatBlock<'_>| {
        let CompatBlock {
            tag_name,
            attrs,
            content,
            location,
        } = block;
        let Ok(name) = core::str::from_utf8(tag_name) else {
            return true;
        };
        let kind = if tag_name_eq(tag_name, b"template") {
            BlockKind::Template
        } else if tag_name_eq(tag_name, b"script") {
            if attrs.contains_key("setup") {
                BlockKind::ScriptSetup
            } else {
                BlockKind::Script
            }
        } else if tag_name_eq(tag_name, b"style") {
            BlockKind::Style
        } else if is_void_tag(name) {
            return true;
        } else {
            BlockKind::Custom
        };
        duplicate = match kind {
            BlockKind::Template => core::mem::replace(&mut template, true)
                .then_some("SFC can only contain one <template> block"),
            BlockKind::Script => core::mem::replace(&mut script, true)
                .then_some("SFC can only contain one <script> block"),
            BlockKind::ScriptSetup => core::mem::replace(&mut setup, true)
                .then_some("SFC can only contain one <script setup> block"),
            BlockKind::Style | BlockKind::Custom => None,
        };
        if duplicate.is_some() {
            return false;
        }
        blocks.push(SfcBlock {
            kind,
            name,
            attrs,
            content,
            loc: BlockLocation {
                start: location.start,
                end: location.end,
                tag_start: location.tag_start,
                tag_end: location.tag_end,
            },
        });
        true
    })
    .map_err(|error| FormatError::ParseError(error.message))?;
    match duplicate {
        Some(message) => Err(FormatError::ParseError(String::from(message))),
        None => Ok(blocks),
    }
}

fn template_only<'a>(template: &TemplateOnlyDescriptor<'a>) -> SfcBlock<'a> {
    let content = template.block();
    let mut attrs = BlockAttrs::default();
    if let Some(lang) = template.lang() {
        attrs.insert(Cow::Borrowed("lang"), Cow::Borrowed(lang));
    }
    SfcBlock {
        kind: BlockKind::Template,
        name: "template",
        attrs,
        content: Cow::Borrowed(content.source()),
        loc: BlockLocation {
            start: content.start() as usize,
            end: content.end() as usize,
            tag_start: template.open_tag().start as usize,
            tag_end: template.close_tag().end as usize,
        },
    }
}
