//! ecosystem/vue-i18n-no-missing-key
//!
//! Check static vue-i18n keys against same-file SFC catalogs.
//!
//! This first slice intentionally validates only local `<i18n>` JSON blocks.
//! The extractor is small and allocation-conscious so it can later back a
//! project-level index for external locale files and Maestro editor features.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use memchr::memmem;
use serde_json::Value;
use vize_atelier_sfc::{parse_sfc, SfcCustomBlock, SfcParseOptions};
use vize_carton::{CompactString, FxHashSet, String};

static META: RuleMeta = RuleMeta {
    name: "ecosystem/vue-i18n-no-missing-key",
    description: "Report static vue-i18n keys that are absent from local SFC messages",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct VueI18nNoMissingKey;

impl Rule for VueI18nNoMissingKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if !should_scan(ctx.source.as_bytes()) {
            return;
        }

        let Ok(descriptor) = parse_sfc(ctx.source, sfc_parse_options(ctx.filename)) else {
            return;
        };

        let mut keys = FxHashSet::default();
        for block in &descriptor.custom_blocks {
            collect_i18n_block_keys(block, &mut keys);
        }
        if keys.is_empty() {
            return;
        }

        for literal in translation_literals(ctx.source) {
            if keys.contains(literal.key) {
                continue;
            }
            let diagnostic = LintDiagnostic::warn(
                META.name,
                "Missing vue-i18n message key",
                literal.start as u32,
                literal.end as u32,
            )
            .with_help(
                "Add this key to the local <i18n> block or move the catalog into the project-level i18n index when external resources are enabled.",
            );
            ctx.report(diagnostic);
        }
    }
}

#[derive(Clone, Copy)]
struct TranslationLiteral<'a> {
    key: &'a str,
    start: usize,
    end: usize,
}

fn should_scan(bytes: &[u8]) -> bool {
    memmem::find(bytes, b"<i18n").is_some()
        && (memmem::find(bytes, b"$t(").is_some()
            || memmem::find(bytes, b"$te(").is_some()
            || memmem::find(bytes, b"$tm(").is_some()
            || memmem::find(bytes, b"t(").is_some()
            || memmem::find(bytes, b"te(").is_some()
            || memmem::find(bytes, b"tm(").is_some())
}

fn collect_i18n_block_keys(block: &SfcCustomBlock<'_>, keys: &mut FxHashSet<CompactString>) {
    if block.block_type.as_ref() != "i18n" || !is_json_block(block) {
        return;
    }

    let Ok(value) = serde_json::from_str::<Value>(block.content.as_ref()) else {
        return;
    };

    flatten_message_root(&value, keys);
}

fn is_json_block(block: &SfcCustomBlock<'_>) -> bool {
    block
        .attrs
        .get("lang")
        .is_none_or(|lang| matches!(lang.as_ref(), "json" | ""))
}

fn flatten_message_root(value: &Value, keys: &mut FxHashSet<CompactString>) {
    let Some(object) = value.as_object() else {
        return;
    };

    let mut used_locale_root = false;
    for (name, value) in object {
        if looks_like_locale(name) && value.is_object() {
            flatten_message_value("", value, keys);
            used_locale_root = true;
        }
    }

    if !used_locale_root {
        flatten_message_value("", value, keys);
    }
}

fn flatten_message_value(prefix: &str, value: &Value, keys: &mut FxHashSet<CompactString>) {
    match value {
        Value::String(_) if !prefix.is_empty() => {
            keys.insert(CompactString::new(prefix));
        }
        Value::Object(object) => {
            for (segment, child) in object {
                let next = join_key(prefix, segment);
                flatten_message_value(next.as_str(), child, keys);
            }
        }
        _ => {}
    }
}

fn join_key(prefix: &str, segment: &str) -> String {
    if prefix.is_empty() {
        return String::from(segment);
    }

    let mut key = String::with_capacity(prefix.len() + segment.len() + 1);
    key.push_str(prefix);
    key.push('.');
    key.push_str(segment);
    key
}

fn looks_like_locale(name: &str) -> bool {
    if name.len() == 2 && name.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return true;
    }

    name.contains('-')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn translation_literals(source: &str) -> Vec<TranslationLiteral<'_>> {
    let bytes = source.as_bytes();
    let mut literals = Vec::new();
    let mut search_start = 0usize;

    while search_start < bytes.len() {
        let Some(relative) = memchr::memchr2(b'\'', b'"', &bytes[search_start..]) else {
            break;
        };
        let quote_start = search_start + relative;
        let quote = bytes[quote_start];
        let Some(paren) = preceding_open_paren(bytes, quote_start) else {
            search_start = quote_start + 1;
            continue;
        };
        let Some(callee) = callee_before_open_paren(source, bytes, paren) else {
            search_start = quote_start + 1;
            continue;
        };
        if !is_i18n_callee(callee) {
            search_start = quote_start + 1;
            continue;
        }

        let content_start = quote_start + 1;
        let Some(content_end) = find_string_end(bytes, quote, content_start) else {
            break;
        };
        let key = &source[content_start..content_end];
        if !key.is_empty() && !key.as_bytes().contains(&b'\\') {
            literals.push(TranslationLiteral {
                key,
                start: content_start,
                end: content_end,
            });
        }
        search_start = content_end + 1;
    }

    literals
}

fn preceding_open_paren(bytes: &[u8], quote_start: usize) -> Option<usize> {
    let mut idx = quote_start;
    while idx > 0 && bytes[idx - 1].is_ascii_whitespace() {
        idx -= 1;
    }
    if idx == 0 || bytes[idx - 1] != b'(' {
        return None;
    }
    Some(idx - 1)
}

fn callee_before_open_paren<'a>(
    source: &'a str,
    bytes: &[u8],
    open_paren: usize,
) -> Option<&'a str> {
    let mut end = open_paren;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    let mut start = end;
    while start > 0 && is_callee_byte(bytes[start - 1]) {
        start -= 1;
    }

    if start == end {
        return None;
    }
    source.get(start..end)
}

fn is_callee_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.')
}

fn is_i18n_callee(callee: &str) -> bool {
    if matches!(callee, "$t" | "$te" | "$tm" | "t" | "te" | "tm") {
        return true;
    }

    if !callee.contains("i18n") {
        return false;
    }

    let leaf = callee.rsplit('.').next().unwrap_or(callee);
    matches!(leaf, "$t" | "$te" | "$tm" | "t" | "te" | "tm")
}

fn find_string_end(bytes: &[u8], quote: u8, mut idx: usize) -> Option<usize> {
    while idx < bytes.len() {
        match bytes[idx] {
            b'\\' => idx += 2,
            byte if byte == quote => return Some(idx),
            _ => idx += 1,
        }
    }
    None
}

#[inline]
fn sfc_parse_options(filename: &str) -> SfcParseOptions {
    SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::{translation_literals, VueI18nNoMissingKey};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VueI18nNoMissingKey));
        Linter::with_registry(registry)
    }

    #[test]
    fn extracts_static_translation_literals() {
        let source = r#"{{ $t("home.title") }} {{ i18n.global.t('home.body') }}"#;
        let keys: Vec<_> = translation_literals(source)
            .into_iter()
            .map(|literal| literal.key)
            .collect();
        assert_eq!(keys, vec!["home.title", "home.body"]);
    }

    #[test]
    fn accepts_key_from_locale_root() {
        let source = r#"<template>{{ $t("home.title") }}</template>
<i18n lang="json">
{ "en": { "home": { "title": "Home" } } }
</i18n>"#;
        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn accepts_key_from_direct_root() {
        let source = r#"<template>{{ $t("home.title") }}</template>
<i18n lang="json">
{ "home": { "title": "Home" } }
</i18n>"#;
        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_missing_key() {
        let source = r#"<template>{{ $t("home.missing") }}</template>
<i18n lang="json">
{ "en": { "home": { "title": "Home" } } }
</i18n>"#;
        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}
