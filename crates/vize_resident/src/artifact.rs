//! Owned stage artifacts of one SFC block, and the pure stage functions that
//! compute them.
//!
//! These functions are the **clean** path. The salsa tier ([`crate::db`])
//! calls exactly them from its queries, and TS-42 (P5-9) compares every
//! artifact the tier serves after an edit sequence with
//! [`compute_file_artifacts`] run from scratch. Every function here reads a
//! [`BlockSource`] — content, never position — plus the project's
//! [`StageConfig`]; a block's position lives in the S0 side table
//! (`Block::start` in the database), outside every artifact and key.

use std::borrow::Cow;

use vize_croquis::sfc::{SfcParseOptions, parse_sfc};
use vize_davinci::diagnostic::Diagnostic;
use vize_davinci::key::{ArtifactKey, source_block_key};
use vize_relief::ErrorCode;
use vize_s0::config::VueVersion;
use vize_s0::{Allocator, FxHashMap, String};
use vize_s1_to_s2::key::SurfacePage;
use vize_s1_to_s2::{LegacyCaps, lower_style_block, lower_with_caps};
use vize_s2::folio::S2Folio;

/// The kind of an SFC block, as its identity in the database sees it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlockKind {
    /// `<template>`.
    Template,
    /// `<script>` without `setup`.
    Script,
    /// `<script setup>`.
    ScriptSetup,
    /// `<style>`.
    Style,
    /// Any custom block, by its tag name (`<i18n>`, `<docs>`, ...).
    Custom(String),
}

impl BlockKind {
    /// The block's tag name — the S0 key's `kind` field.
    #[must_use]
    pub fn tag(&self) -> &str {
        match self {
            Self::Template => "template",
            Self::Script | Self::ScriptSetup => "script",
            Self::Style => "style",
            Self::Custom(tag) => tag.as_str(),
        }
    }
}

/// Everything a stage artifact of one block may read: its kind, header
/// attributes (sorted — a set) and content, plus the S0 key over them. No
/// position: equal content at any offset is an equal `BlockSource`.
///
/// Its equality is the firewall's: salsa compares a re-created block's
/// `source` with the stored one and keeps the stored value when they are
/// equal. The `seeded-stale-cache` feature (TS-42's seeded defect, never on
/// in a real build) weakens it to kind, header and content *length* — a key
/// covering less than its input — so TS-42 can prove it catches the
/// resulting stale artifacts.
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "seeded-stale-cache"), derive(PartialEq, Eq))]
pub struct BlockSource {
    /// The block kind.
    pub kind: BlockKind,
    /// Header attributes as `(name, value)`, sorted; bare attributes carry
    /// an empty value (the SFC descriptor's spelling).
    pub attrs: Vec<(String, String)>,
    /// The block content, exactly as authored.
    pub text: String,
    /// The P5-1a S0 source-block key over the three fields above.
    pub key: ArtifactKey,
}

#[cfg(feature = "seeded-stale-cache")]
impl PartialEq for BlockSource {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.attrs == other.attrs && self.text.len() == other.text.len()
    }
}

#[cfg(feature = "seeded-stale-cache")]
impl Eq for BlockSource {}

impl BlockSource {
    /// The value of header attribute `name`, if present.
    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(attr, _)| attr.as_str() == name)
            .map(|(_, value)| value.as_str())
    }
}

/// One block of a split file: identity (`kind`, `ordinal` among blocks of
/// the same kind), position, and content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSlot {
    /// The block kind.
    pub kind: BlockKind,
    /// Index among blocks of the same kind, in document order.
    pub ordinal: u32,
    /// File-absolute byte offset of the block content.
    pub start: u32,
    /// The block content.
    pub source: BlockSource,
}

/// Project configuration the stages read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct StageConfig {
    /// The configured Vue line; selects the lowering's template dialect.
    pub vue_version: VueVersion,
}

/// One tokenizer finding of an S1 parse, block-relative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceFinding {
    /// The tokenizer's error code.
    pub code: ErrorCode,
    /// Byte offset inside the block.
    pub offset: u32,
}

/// The S1 artifact of a template block: its P5-1a key and the tokenizer's
/// findings. (The tree itself is arena-bound and never leaves its parse.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceArtifact {
    /// The S1 surface page key.
    pub key: ArtifactKey,
    /// Tokenizer findings, in report order.
    pub findings: Vec<SurfaceFinding>,
}

/// The S2 artifact of a block: the owned page, its P5-1a key, and the
/// lowering's diagnostics. Spans are block-relative (the block is lowered as
/// its own root); add the block's `start` for file-absolute positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageArtifact {
    /// The S2 page key.
    pub key: ArtifactKey,
    /// The owned page.
    pub folio: S2Folio,
    /// Surface and lowering diagnostics, in decision order.
    pub diagnostics: Vec<Diagnostic>,
}

/// Every artifact of one block, as TS-42 compares them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockArtifacts {
    /// The block kind.
    pub kind: BlockKind,
    /// Index among blocks of the same kind.
    pub ordinal: u32,
    /// File-absolute start (the S0 side table).
    pub start: u32,
    /// The S0 source-block key.
    pub source_key: ArtifactKey,
    /// The S1 artifact (template blocks only).
    pub surface: Option<SurfaceArtifact>,
    /// The S2 artifact (template and style blocks).
    pub page: Option<PageArtifact>,
}

/// Split an SFC into its blocks, in document order. A file the SFC parser
/// rejects has no blocks.
#[must_use]
pub fn split_blocks(text: &str) -> Vec<BlockSlot> {
    let Ok(descriptor) = parse_sfc(text, SfcParseOptions::default()) else {
        return Vec::new();
    };
    let mut slots = Vec::new();
    let mut push = |kind: BlockKind, ordinal: usize, start: usize, end: usize, attrs| {
        slots.extend(slot(text, kind, ordinal, start, end, attrs));
    };
    if let Some(template) = &descriptor.template {
        push(
            BlockKind::Template,
            0,
            template.loc.start,
            template.loc.end,
            &template.attrs,
        );
    }
    for (kind, script) in [
        (BlockKind::Script, &descriptor.script),
        (BlockKind::ScriptSetup, &descriptor.script_setup),
    ] {
        if let Some(script) = script {
            push(kind, 0, script.loc.start, script.loc.end, &script.attrs);
        }
    }
    for (ordinal, style) in descriptor.styles.iter().enumerate() {
        push(
            BlockKind::Style,
            ordinal,
            style.loc.start,
            style.loc.end,
            &style.attrs,
        );
    }
    let mut custom_ordinals: Vec<(String, usize)> = Vec::new();
    for custom in &descriptor.custom_blocks {
        let tag = String::from(custom.block_type.as_ref());
        let ordinal = match custom_ordinals.iter_mut().find(|(seen, _)| *seen == tag) {
            Some((_, count)) => {
                *count += 1;
                *count - 1
            }
            None => {
                custom_ordinals.push((tag.clone(), 1));
                0
            }
        };
        let (start, end) = (custom.loc.start, custom.loc.end);
        push(BlockKind::Custom(tag), ordinal, start, end, &custom.attrs);
    }
    slots.sort_by_key(|slot| slot.start);
    slots
}

type HeaderAttrs<'a> = FxHashMap<Cow<'a, str>, Cow<'a, str>>;

fn slot(
    text: &str,
    kind: BlockKind,
    ordinal: usize,
    start: usize,
    end: usize,
    attrs: &HeaderAttrs<'_>,
) -> Option<BlockSlot> {
    let content = text.get(start..end)?;
    let mut pairs: Vec<(String, String)> = attrs
        .iter()
        .map(|(name, value)| (String::from(name.as_ref()), String::from(value.as_ref())))
        .collect();
    pairs.sort_unstable();
    let borrowed: Vec<(&str, Option<&str>)> = pairs
        .iter()
        .map(|(name, value)| (name.as_str(), Some(value.as_str())))
        .collect();
    let key = source_block_key(kind.tag(), &borrowed, content);
    Some(BlockSlot {
        start: u32::try_from(start).ok()?,
        ordinal: u32::try_from(ordinal).ok()?,
        source: BlockSource {
            kind: kind.clone(),
            attrs: pairs,
            text: String::from(content),
            key,
        },
        kind,
    })
}

/// Whether a template block is HTML (the only S1 input language today).
fn is_html_template(source: &BlockSource) -> bool {
    source.kind == BlockKind::Template
        && source.attr("src").is_none()
        && source.attr("lang").is_none_or(|lang| lang == "html")
}

/// The S1 artifact of a block: `Some` for an HTML template block.
#[must_use]
pub fn surface_artifact(source: &BlockSource) -> Option<SurfaceArtifact> {
    if !is_html_template(source) {
        return None;
    }
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, source.text.as_str());
    Some(SurfaceArtifact {
        key: ArtifactKey::of(&SurfacePage(&tree), 0),
        findings: errors
            .iter()
            .map(|error| SurfaceFinding {
                code: error.code,
                offset: error.offset,
            })
            .collect(),
    })
}

/// The S2 artifact of a block: `Some` for an HTML template block (the op
/// tree) and for a style block (its `vue.css-bind` page).
#[must_use]
pub fn page_artifact(source: &BlockSource, config: StageConfig) -> Option<PageArtifact> {
    let allocator = Allocator::default();
    if source.kind == BlockKind::Style {
        let op = lower_style_block(&allocator, source.text.as_str(), 0);
        let folio = S2Folio::of(core::slice::from_ref(&op));
        return Some(PageArtifact {
            key: ArtifactKey::of(&folio, 0),
            folio,
            diagnostics: Vec::new(),
        });
    }
    if !is_html_template(source) {
        return None;
    }
    let caps = LegacyCaps::for_version(config.vue_version);
    let (tree, errors) = vize_s1::parse(&allocator, source.text.as_str());
    let lowered = lower_with_caps(&allocator, &tree, &errors, caps);
    let folio = S2Folio::of(&lowered.root.ops);
    Some(PageArtifact {
        key: ArtifactKey::of(&folio, 0),
        folio,
        diagnostics: lowered.diagnostics,
    })
}

/// Every artifact of every block of `text`, computed from scratch — the
/// clean side of TS-42.
#[must_use]
pub fn compute_file_artifacts(text: &str, config: StageConfig) -> Vec<BlockArtifacts> {
    split_blocks(text)
        .into_iter()
        .map(|slot| BlockArtifacts {
            surface: surface_artifact(&slot.source),
            page: page_artifact(&slot.source, config),
            source_key: slot.source.key,
            kind: slot.kind,
            ordinal: slot.ordinal,
            start: slot.start,
        })
        .collect()
}
