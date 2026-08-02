//! Source heuristics that decide whether ecosystem-specific template rules
//! can be skipped for a given source.

use super::tag_scan::{find_tag_end, tag_name_at};

pub(super) fn source_may_contain_ecosystem_template_rule(
    template_source: &str,
    sfc_source: Option<&str>,
) -> bool {
    let template_bytes = template_source.as_bytes();
    if template_may_contain_ecosystem_element(template_bytes, sfc_source) {
        return true;
    }

    sfc_source.is_some_and(|source| source.contains("<i18n"))
        && template_may_call_i18n(template_bytes)
}

fn template_may_contain_ecosystem_element(bytes: &[u8], sfc_source: Option<&str>) -> bool {
    let imports_void_vue = sfc_source.is_some_and(|source| source.contains("@void/vue"));
    let mut cursor = 0;
    while let Some(relative) = memchr::memchr(b'<', &bytes[cursor..]) {
        let tag_start = cursor + relative;
        let Some((tag_name, name_end)) = tag_name_at(bytes, tag_start) else {
            cursor = tag_start + 1;
            continue;
        };

        if matches!(
            tag_name,
            b"RouterLink" | b"router-link" | b"NuxtLink" | b"nuxt-link"
        ) {
            return true;
        }
        if tag_name == b"Link" && imports_void_vue {
            return true;
        }

        let Some(tag_end) = find_tag_end(bytes, name_end) else {
            return false;
        };
        cursor = tag_end + 1;
    }
    false
}

fn template_may_call_i18n(bytes: &[u8]) -> bool {
    memchr::memmem::find(bytes, b"$t(").is_some()
        || memchr::memmem::find(bytes, b"$te(").is_some()
        || memchr::memmem::find(bytes, b"$tm(").is_some()
        || memchr::memmem::find(bytes, b"t(").is_some()
        || memchr::memmem::find(bytes, b"te(").is_some()
        || memchr::memmem::find(bytes, b"tm(").is_some()
}

#[cfg(test)]
mod tests {
    use super::source_may_contain_ecosystem_template_rule;

    #[test]
    fn ecosystem_template_hint_ignores_plain_internal_anchor() {
        assert!(!source_may_contain_ecosystem_template_rule(
            r#"<a href = "/docs">Docs</a>"#,
            None
        ));
    }

    #[test]
    fn ecosystem_template_hint_ignores_bound_href_with_script_path_strings() {
        let template = r#"<a :href="link.url">Docs</a>"#;
        let sfc = r#"<template><a :href="link.url">Docs</a></template>
<script setup>
const links = [{ url: '/docs' }]
</script>"#;
        assert!(!source_may_contain_ecosystem_template_rule(
            template,
            Some(sfc)
        ));
    }
}
