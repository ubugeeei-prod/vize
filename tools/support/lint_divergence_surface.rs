//! Lint divergence surface rules: findings the baseline cannot see, and
//! baseline findings Vize intentionally does not report.

use super::LintRecord;

/// Whether a Patina finding lies outside eslint-plugin-vue's comparable surface.
pub(crate) fn outside_baseline_surface(finding: &LintRecord, source: &str) -> bool {
    pug_template_lines(source).is_some_and(|(start, end)| (start..=end).contains(&finding.line))
        || unused_script_setup_import(finding, source)
}

const CAMEL_ARIA_DATA_REASON: &str = "Vize keeps camelCase aria*/data* names on components: type checkers read hyphenated aria-*/data-* names as fallthrough attributes, so hyphenating them would drop the prop type check.";

/// Baseline findings Vize deliberately does not report, with the reason.
pub(crate) fn baseline_intentional_reason(finding: &LintRecord) -> Option<&'static str> {
    if finding.rule_id != "vue/attribute-hyphenation" {
        return None;
    }
    let name = quoted_name(&finding.message)?;
    let name = name
        .strip_prefix("v-bind:")
        .or_else(|| name.strip_prefix(':'))
        .unwrap_or(name);
    ["aria", "data"]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix)
                .and_then(|rest| rest.chars().next())
                .is_some_and(|next| next.is_ascii_uppercase())
        })
        .then_some(CAMEL_ARIA_DATA_REASON)
}

/// eslint-plugin-vue's `vue/no-unused-components` checks only the Options API
/// `components` option, so an unused `<script setup>` import is not its surface.
fn unused_script_setup_import(finding: &LintRecord, source: &str) -> bool {
    if finding.rule_id != "vue/no-unused-components" {
        return false;
    }
    let Some(name) = quoted_name(&finding.message) else {
        return false;
    };
    script_setup_blocks(source).any(|block| {
        block.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with("import ") && contains_identifier(line, name)
        })
    })
}

/// The first single-quoted name in a diagnostic message.
fn quoted_name(message: &str) -> Option<&str> {
    let start = message.find('\'')? + 1;
    let length = message.get(start..)?.find('\'')?;
    message.get(start..start + length)
}

fn script_setup_blocks(source: &str) -> impl Iterator<Item = &str> {
    source.split("<script").skip(1).filter_map(|rest| {
        let open_end = rest.find('>')?;
        let setup = rest
            .get(..open_end)?
            .split_whitespace()
            .any(|attribute| attribute == "setup" || attribute.starts_with("setup="));
        let body = rest.get(open_end + 1..)?;
        setup.then(|| body.get(..body.find("</script>")?)).flatten()
    })
}

fn contains_identifier(line: &str, name: &str) -> bool {
    let is_identifier = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '$';
    line.match_indices(name).any(|(start, _)| {
        let before = line.get(..start).and_then(|text| text.chars().next_back());
        let after = line
            .get(start + name.len()..)
            .and_then(|text| text.chars().next());
        !before.is_some_and(is_identifier) && !after.is_some_and(is_identifier)
    })
}

/// The 1-based line range of an inline `<template lang="pug">` body — the
/// first top-level template, no `src` — or `None`. A body that starts on
/// the tag's line starts there; otherwise on the next line.
fn pug_template_lines(source: &str) -> Option<(u64, u64)> {
    let tag_start = source.find("<template")?;
    let tag_end = tag_start + source[tag_start..].find('>')?;
    let tag = &source[tag_start..tag_end];
    let pug = ["lang=\"pug\"", "lang='pug'", "lang=pug"]
        .iter()
        .any(|spelling| tag.contains(spelling));
    if !pug || tag.contains("src=") {
        return None;
    }
    let body_start = tag_end + 1;
    let body_end = body_start + source[body_start..].find("</template>")?;
    let line_of = |offset: usize| source[..offset].matches('\n').count() as u64 + 1;
    let first = if source[body_start..].starts_with(['\n', '\r']) {
        line_of(body_start) + 1
    } else {
        line_of(body_start)
    };
    Some((first, line_of(body_end)))
}
