//! Script binding reads inside Musea `<art>` blocks.
//!
//! Musea compiles each `<variant>` body as a template in the
//! `<script setup>` scope, so a component, prop, or helper named there is a
//! use just like one named in `<template>`. The `<art>` and `<variant>` tags
//! carry static metadata only and read nothing.

use vize_atelier_sfc::SfcDescriptor;
use vize_l0::{Allocator, SourceRoot};
use vize_relief::{ElementNode, TemplateChildNode};

use super::template_use::{TemplateUse, walk_children};

const ART_BLOCK: &str = "art";
const VARIANT_TAG: &str = "variant";

pub(super) fn note_art_uses(content: &str, uses: &mut TemplateUse) {
    if content.trim().is_empty() {
        return;
    }
    let allocator = Allocator::default();
    let (root, _errors) = vize_armature::parse(&allocator, content);
    note_variants(&root.children, root.source, uses);
}

pub(super) fn is_art_block(block_type: &str) -> bool {
    block_type == ART_BLOCK
}

/// The linter also runs template rules on each variant body on its own. That
/// pass sees one variant, while the SFC pass already counts every variant,
/// so the script rules skip it.
pub(super) fn is_art_variant_source(descriptor: &SfcDescriptor<'_>, source: &str) -> bool {
    let Some(start) = SourceRoot::new(descriptor.source.as_ref())
        .ok()
        .and_then(|root| root.whole_block().offset_of(source))
    else {
        return false;
    };
    let start = start as usize;
    descriptor.custom_blocks.iter().any(|block| {
        is_art_block(&block.block_type) && block.loc.start <= start && start < block.loc.end
    })
}

/// A `<variant>` anywhere under `<art>` is a template body. Other elements
/// only group variants, so the walk looks through them.
fn note_variants(children: &[TemplateChildNode<'_>], source: &str, uses: &mut TemplateUse) {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        if element.tag == VARIANT_TAG {
            note_variant(element, source, uses);
        } else {
            note_variants(&element.children, source, uses);
        }
    }
}

fn note_variant(variant: &ElementNode<'_>, source: &str, uses: &mut TemplateUse) {
    let mut shadow = Vec::new();
    walk_children(&variant.children, source, &mut shadow, uses);
}
