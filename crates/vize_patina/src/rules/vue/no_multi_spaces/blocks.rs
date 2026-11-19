//! Spacing inside the start and end tags of an SFC's top-level blocks.
//!
//! eslint-plugin-vue checks the tokens of every top-level element's start
//! and end tag, so `<script setup  lang="ts">` and `<style  scoped>` report
//! the gap between two tokens that start on the same line.

use super::META;
use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};

pub(super) fn check(ctx: &mut LintContext<'_>) {
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return;
    };
    let mut gaps = Vec::new();
    let locations = descriptor
        .template
        .iter()
        .map(|block| &block.loc)
        .chain(descriptor.script.iter().map(|block| &block.loc))
        .chain(descriptor.script_setup.iter().map(|block| &block.loc))
        .chain(descriptor.styles.iter().map(|block| &block.loc))
        .chain(descriptor.custom_blocks.iter().map(|block| &block.loc));
    for location in locations {
        tag_gaps(ctx.source, location.tag_start, location.start, &mut gaps);
        tag_gaps(ctx.source, location.end, location.tag_end, &mut gaps);
    }
    for (start, end) in gaps {
        let fix = Fix::new(
            "Replace multiple spaces with single space",
            TextEdit::replace(start, end, " "),
        );
        ctx.report_in_sfc(
            LintDiagnostic::warn(META.name, "Multiple consecutive spaces", start, end)
                .with_fix(fix),
        );
    }
}

/// Every whitespace run longer than one byte between two tokens of the tag
/// `source[start..end]` that start on the same line. Only a quoted value can
/// span lines, so a token starts at its opening quote or after whitespace.
fn tag_gaps(source: &str, start: usize, end: usize, gaps: &mut Vec<(u32, u32)>) {
    let Some(tag) = source.as_bytes().get(start..end) else {
        return;
    };
    let mut quote = None;
    let mut token_start = 0;
    let mut index = 0;
    while let Some(&byte) = tag.get(index) {
        match (quote, byte) {
            (Some(open), byte) if byte == open => quote = None,
            (Some(_), _) => {}
            (None, b'"' | b'\'') => {
                quote = Some(byte);
                token_start = index;
            }
            (None, byte) if byte.is_ascii_whitespace() => {
                let run = index;
                while tag.get(index).is_some_and(u8::is_ascii_whitespace) {
                    index += 1;
                }
                let one_line = tag
                    .get(token_start..index)
                    .is_some_and(|text| !text.iter().any(|byte| matches!(byte, b'\n' | b'\r')));
                if index - run > 1 && index < tag.len() && one_line {
                    gaps.push(((start + run) as u32, (start + index) as u32));
                }
                token_start = index;
                continue;
            }
            (None, _) => {}
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests;
