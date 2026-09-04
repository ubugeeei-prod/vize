//! vue/warn-custom-block
//!
//! Warn about custom blocks in SFC files.
//!
//! Custom blocks (blocks other than `<script>`, `<template>`, `<style>`)
//! require additional configuration and tooling support. This rule warns
//! about their usage to ensure they are intentional.
//!
//! ## Common Custom Blocks
//!
//! - `<i18n>` - Vue I18n translations
//! - `<docs>` - Component documentation
//! - `<story>` - Storybook stories
//! - `<test>` - Component tests
//!
//! ## Examples
//!
//! ### Triggers Warning
//! ```vue
//! <i18n>
//! { "en": { "hello": "Hello" } }
//! </i18n>
//!
//! <docs>
//! # MyComponent
//! This is a custom component.
//! </docs>
//! ```
//!
//! ## Detection
//!
//! Custom blocks are *top-level* SFC blocks (siblings of `<template>`,
//! `<script>`, and `<style>`), so detection relies on the SFC parser's block
//! boundaries rather than any raw-text or indentation heuristic. Elements
//! inside the `<template>` block — including unindented multi-root fragment
//! children starting at column 0 — are template content, never custom blocks
//! (issue #3210).
//!
//! ## Note
//!
//! This rule is informational. Custom blocks are valid and useful when
//! properly configured with the appropriate Vite/Webpack plugins.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/warn-custom-block",
    description: "Warn about custom blocks in SFC files",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Warn about custom blocks
#[derive(Default)]
pub struct WarnCustomBlock;

impl Rule for WarnCustomBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // Custom blocks are an SFC concept (`<i18n>`, `<docs>`, etc.).
        // Standalone HTML files (e.g. `index.html`, `.storybook/preview-head.html`)
        // are not Vue SFCs, so every top-level non-`script`/`template`/`style`
        // tag (`<link>`, `<meta>`, `<html>`, ...) would be flagged as a custom
        // block. Skip the rule on non-SFC files. See issue #2245.
        if !is_sfc_filename(ctx.filename) {
            return;
        }

        // The engine shares one parsed descriptor across SFC-level rules; parse
        // lazily only when a host drives this rule without preparing one. A
        // source that fails to parse as an SFC has no trustworthy block
        // boundaries — parse errors are reported elsewhere, so stay silent
        // instead of guessing.
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.warn_custom_block.parse_sfc",
                parse_sfc(
                    ctx.source,
                    SfcParseOptions {
                        filename: ctx.filename.into(),
                        ..Default::default()
                    },
                )
            ) {
                Ok(descriptor) => descriptor,
                Err(_) => return,
            };
            &owned_descriptor
        };

        if descriptor.custom_blocks.is_empty() {
            return;
        }

        // `loc.tag_start` is the `<` of the opening tag and `loc.start` is the
        // first content byte right past its `>` (past `/>` for self-closing
        // blocks), so this span covers exactly the opening tag. Collected
        // before reporting because the descriptor borrow must end before
        // `ctx.report` takes `ctx` mutably.
        let spans: Vec<(u32, u32)> = descriptor
            .custom_blocks
            .iter()
            .filter(|block| !is_known_musea_art_block(ctx.filename, block.block_type.as_ref()))
            .map(|block| (block.loc.tag_start as u32, block.loc.start as u32))
            .collect();

        for (start, end) in spans {
            ctx.report(
                LintDiagnostic::warn(
                    META.name,
                    "Custom block detected. Ensure proper plugin configuration.",
                    start,
                    end,
                )
                .with_help(
                    "Custom blocks require corresponding Vite/Webpack plugins to be processed",
                ),
            );
        }
    }
}

/// Returns `true` when the file should be treated as a Vue SFC for the purposes
/// of custom-block detection (i.e. its extension is `.vue`).
fn is_sfc_filename(filename: &str) -> bool {
    filename.rsplit('.').next() == Some("vue")
}

fn is_known_musea_art_block(filename: &str, block_type: &str) -> bool {
    block_type == "art" && filename.ends_with(".art.vue")
}

#[cfg(test)]
mod tests;
