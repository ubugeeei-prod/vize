//! Byte-level prefilters that decide whether a script block can possibly match
//! an enabled built-in rule before the block is parsed.
//!
//! Split out of the parent module so it stays focused on rule dispatch: the
//! per-rule substring gate ([`script_rule_may_match`]) and the ecosystem-rule
//! fast path both live here.

use memchr::memmem;
use vize_atelier_sfc::SfcDescriptor;

use super::registry::{
    RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS, RULE_NUXT_CONFIG_KEYS_ORDER,
    RULE_NUXT_NO_CONFIG_TEST_KEY, RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES,
    RULE_NUXT_PREFER_IMPORT_META, RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED,
    RULE_PREFER_IMPORT_FROM_VUE, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};
use super::{Linter, active_builtin_script_rule_entries};

pub(super) fn script_rule_may_match(rule_name: &str, source: &str) -> bool {
    let bytes = source.as_bytes();
    match rule_name {
        RULE_PINIA_PREFER_STORE_TO_REFS => memmem::find(bytes, b"Store").is_some(),
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH => {
            (memmem::find(bytes, b".push").is_some() || memmem::find(bytes, b".replace").is_some())
                && (memmem::find(bytes, b"'/").is_some() || memmem::find(bytes, b"\"/").is_some())
                && (memmem::find(bytes, b"router").is_some()
                    || memmem::find(bytes, b"Router").is_some())
        }
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT => {
            memmem::find(bytes, b"toMatchSnapshot").is_some()
                && memmem::find(bytes, b".html").is_some()
        }
        RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES => memmem::find(bytes, b"definePageMeta").is_some(),
        RULE_NUXT_NO_CONFIG_TEST_KEY | RULE_NUXT_CONFIG_KEYS_ORDER => {
            memmem::find(bytes, b"export").is_some()
        }
        RULE_PREFER_COMPUTED => memmem::find(bytes, b"watch").is_some(),
        RULE_PREFER_IMPORT_FROM_VUE => memmem::find(bytes, b"@vue/").is_some(),
        RULE_NUXT_PREFER_IMPORT_META => memmem::find(bytes, b"process").is_some(),
        RULE_NO_RESTRICTED_GLOBALS => {
            memmem::find(bytes, b"process").is_some()
                || memmem::find(bytes, b"localStorage").is_some()
                || memmem::find(bytes, b"sessionStorage").is_some()
        }
        RULE_NO_RESTRICTED_MEMBERS => false,
        _ => true,
    }
}

/// Whether a built-in script rule applies to the current file.
///
/// The Nuxt config ordering rule mirrors an upstream rule that is activated by
/// config overrides. Native presets do not have an override layer, so preserve
/// that boundary here instead of treating every default export as Nuxt config.
pub(super) fn script_rule_applies_to_filename(rule_name: &str, filename: &str) -> bool {
    rule_name != RULE_NUXT_CONFIG_KEYS_ORDER || is_nuxt_config_filename(filename)
}

fn is_nuxt_config_filename(filename: &str) -> bool {
    let mut components = filename
        .rsplit(['/', '\\'])
        .filter(|component| !component.is_empty());
    let Some(basename) = components.next() else {
        return false;
    };

    has_script_extension(basename, "nuxt.config")
        || (components.next() == Some(".config") && has_script_extension(basename, "nuxt"))
}

fn has_script_extension(filename: &str, stem: &str) -> bool {
    matches!(
        filename.strip_prefix(stem),
        Some(
            ".js"
                | ".jsx"
                | ".ts"
                | ".tsx"
                | ".cjs"
                | ".cjsx"
                | ".cts"
                | ".ctsx"
                | ".mjs"
                | ".mjsx"
                | ".mts"
                | ".mtsx"
        )
    )
}

pub(super) fn descriptor_scripts_may_match_ecosystem_rule(descriptor: &SfcDescriptor<'_>) -> bool {
    descriptor
        .script
        .as_ref()
        .is_some_and(|script| source_may_match_ecosystem_rule(script.content.as_ref()))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| source_may_match_ecosystem_rule(script.content.as_ref()))
}

fn is_ecosystem_script_rule(rule_name: &str) -> bool {
    matches!(
        rule_name,
        RULE_PINIA_PREFER_STORE_TO_REFS
            | RULE_VUE_ROUTER_PREFER_NAMED_PUSH
            | RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT
    )
}

pub(super) fn has_only_active_ecosystem_script_rules(linter: &Linter) -> bool {
    active_builtin_script_rule_entries(linter)
        .all(|entry| is_ecosystem_script_rule(entry.rule_name))
}

fn source_may_match_ecosystem_rule(source: &str) -> bool {
    [
        RULE_PINIA_PREFER_STORE_TO_REFS,
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
    ]
    .into_iter()
    .any(|rule_name| script_rule_may_match(rule_name, source))
}

#[cfg(test)]
mod tests {
    use super::is_nuxt_config_filename;

    #[test]
    fn recognizes_supported_nuxt_config_paths() {
        for extension in [
            "js", "jsx", "ts", "tsx", "cjs", "cjsx", "cts", "ctsx", "mjs", "mjsx", "mts", "mtsx",
        ] {
            let root = format!("apps/web/nuxt.config.{extension}");
            let hidden = format!("apps/web/.config/nuxt.{extension}");
            assert!(is_nuxt_config_filename(&root), "{root}");
            assert!(is_nuxt_config_filename(&hidden), "{hidden}");
        }

        assert!(is_nuxt_config_filename(r"apps\web\nuxt.config.js"));
        assert!(is_nuxt_config_filename(r"apps\web\.config\nuxt.mts"));
    }

    #[test]
    fn rejects_unrelated_or_lookalike_paths() {
        for filename in [
            "vitest.config.ts",
            "vize.config.ts",
            "components/DataTable.vue",
            "nuxt.config.d.ts",
            "nuxt.config.json",
            "config/nuxt.ts",
            ".config/not-nuxt.ts",
            "my-nuxt.config.ts",
        ] {
            assert!(!is_nuxt_config_filename(filename), "{filename}");
        }
    }
}
