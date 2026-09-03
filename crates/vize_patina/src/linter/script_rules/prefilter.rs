//! Byte-level prefilters that decide whether a script block can possibly match
//! an enabled built-in rule before the block is parsed.
//!
//! Split out of the parent module so it stays focused on rule dispatch: the
//! per-rule substring gate ([`script_rule_may_match`]) and the ecosystem-rule
//! fast path both live here.

use memchr::memmem;
use vize_atelier_sfc::SfcDescriptor;

mod js_source;
mod ref_operand;

use super::registry::{
    RULE_NO_ARROW_FUNCTIONS_IN_WATCH, RULE_NO_ASYNC_IN_COMPUTED, RULE_NO_DUPE_KEYS,
    RULE_NO_DUPLICATE_ATTR_INHERITANCE, RULE_NO_IMPORT_COMPILER_MACROS, RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_MULTIPLE_SLOT_ARGS, RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO, RULE_NO_REF_AS_OPERAND,
    RULE_NO_REQUIRED_PROP_WITH_DEFAULT, RULE_NO_RESERVED_IDENTIFIERS, RULE_NO_RESERVED_KEYS,
    RULE_NO_RESERVED_PROPS, RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS,
    RULE_NO_SIDE_EFFECTS_IN_COMPUTED, RULE_NO_UNSTABLE_NESTED_COMPONENTS,
    RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD, RULE_NUXT_CONFIG_KEYS_ORDER,
    RULE_NUXT_NO_CONFIG_TEST_KEY, RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES,
    RULE_NUXT_PREFER_IMPORT_META, RULE_PINIA_PREFER_STORE_TO_REFS, RULE_PREFER_COMPUTED,
    RULE_PREFER_IMPORT_FROM_VUE, RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR,
    RULE_REQUIRE_VALID_DEFAULT_PROP, RULE_RETURN_IN_COMPUTED_PROPERTY,
    RULE_RETURN_IN_EMITS_VALIDATOR, RULE_VALID_DEFINE_EMITS, RULE_VALID_DEFINE_OPTIONS,
    RULE_VALID_DEFINE_PROPS, RULE_VALID_NEXT_TICK, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};
use super::{Linter, active_builtin_script_rule_entries};
use js_source::{imports_from_vue, source_may_define_component_options};
use ref_operand::source_may_contain_ref_operand;

pub(super) fn script_rule_may_match(rule_name: &str, source: &str) -> bool {
    let bytes = source.as_bytes();
    match rule_name {
        RULE_NO_ASYNC_IN_COMPUTED => contains(bytes, b"computed") && contains(bytes, b"async"),
        RULE_NO_IMPORT_COMPILER_MACROS => {
            imports_from_vue(bytes) && contains_any(bytes, COMPILER_MACRO_NEEDLES)
        }
        RULE_NO_INTERNAL_IMPORTS => {
            contains(bytes, b"vue") && contains_any(bytes, INTERNAL_IMPORT_NEEDLES)
        }
        RULE_NO_RESERVED_IDENTIFIERS => contains_any(bytes, RESERVED_IDENTIFIER_NEEDLES),
        RULE_NO_RESERVED_KEYS | RULE_NO_DUPE_KEYS | RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO => {
            source_may_define_component_options(bytes)
        }
        RULE_NO_SIDE_EFFECTS_IN_COMPUTED => {
            source_may_define_component_options(bytes)
                && contains(bytes, b"computed")
                && contains(bytes, b"this.")
        }
        RULE_NO_ARROW_FUNCTIONS_IN_WATCH => {
            source_may_define_component_options(bytes)
                && contains(bytes, b"watch")
                && contains(bytes, b"=>")
        }
        RULE_NO_REQUIRED_PROP_WITH_DEFAULT => {
            source_may_declare_props(bytes)
                && contains(bytes, b"required")
                && contains(bytes, b"default")
        }
        RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR => {
            source_may_declare_props(bytes) && contains(bytes, b"type")
        }
        RULE_NO_RESERVED_PROPS => {
            source_may_declare_props(bytes) && contains_any(bytes, RESERVED_PROP_NEEDLES)
        }
        RULE_RETURN_IN_EMITS_VALIDATOR => {
            source_may_declare_emits(bytes) && contains_any(bytes, BLOCK_FUNCTION_NEEDLES)
        }
        RULE_REQUIRE_VALID_DEFAULT_PROP => {
            source_may_declare_props(bytes)
                && contains(bytes, b"type")
                && contains(bytes, b"default")
        }
        RULE_VALID_DEFINE_PROPS => contains(bytes, b"defineProps"),
        RULE_VALID_DEFINE_EMITS => contains(bytes, b"defineEmits"),
        RULE_VALID_DEFINE_OPTIONS => contains(bytes, b"defineOptions"),
        RULE_VALID_NEXT_TICK => contains(bytes, b"nextTick") || contains(bytes, b"$nextTick"),
        RULE_NO_REF_AS_OPERAND => source_may_contain_ref_operand(source),
        RULE_RETURN_IN_COMPUTED_PROPERTY => {
            contains(bytes, b"computed") && contains_any(bytes, BLOCK_FUNCTION_NEEDLES)
        }
        RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD => {
            source_may_define_component_options(bytes) && contains(bytes, b"computed")
        }
        RULE_NO_DUPLICATE_ATTR_INHERITANCE => true,
        RULE_NO_MULTIPLE_SLOT_ARGS => {
            contains(bytes, b"slots") || contains(bytes, b"$slots") || contains(bytes, b"useSlots")
        }
        RULE_NO_UNSTABLE_NESTED_COMPONENTS => {
            source_may_define_component_options(bytes)
                && (contains(bytes, b"components") || contains(bytes, b"defineComponent"))
        }
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

const COMPILER_MACRO_NEEDLES: &[&[u8]] = &[
    b"defineProps",
    b"defineEmits",
    b"defineExpose",
    b"defineModel",
    b"defineOptions",
    b"defineSlots",
    b"withDefaults",
    b"defineArt",
];
const INTERNAL_IMPORT_NEEDLES: &[&[u8]] = &[
    b"/dist/",
    b"/src/",
    b"/esm/",
    b"vue.esm",
    b"vue.cjs",
    b"vue.runtime",
];
const RESERVED_IDENTIFIER_NEEDLES: &[&[u8]] = &[
    b"__props",
    b"__emit",
    b"__expose",
    b"__sfc__",
    b"__sfc_main",
    b"__injectCSSVars__",
    b"_ctx",
    b"_cache",
    b"_setupState",
    b"_hoisted_",
    b"_createBlock",
    b"_createVNode",
    b"_createElementVNode",
    b"_resolveComponent",
    b"_resolveDirective",
    b"_withCtx",
    b"_openBlock",
];
const RESERVED_PROP_NEEDLES: &[&[u8]] = &[b"key", b"ref", b"ref_for", b"ref_key", b"is", b"$"];
const BLOCK_FUNCTION_NEEDLES: &[&[u8]] = &[b"=> {", b"=>{", b"function", b"get("];
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    memmem::find(haystack, needle).is_some()
}

fn contains_any(haystack: &[u8], needles: &[&[u8]]) -> bool {
    needles.iter().any(|needle| contains(haystack, needle))
}

fn source_may_declare_props(bytes: &[u8]) -> bool {
    contains(bytes, b"defineProps")
        || (source_may_define_component_options(bytes) && contains(bytes, b"props"))
}

fn source_may_declare_emits(bytes: &[u8]) -> bool {
    contains(bytes, b"defineEmits")
        || (source_may_define_component_options(bytes) && contains(bytes, b"emits"))
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
    use super::{is_nuxt_config_filename, script_rule_may_match};
    use crate::linter::script_rules::registry::{
        RULE_NO_ARROW_FUNCTIONS_IN_WATCH, RULE_NO_IMPORT_COMPILER_MACROS, RULE_VALID_DEFINE_PROPS,
    };

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

    #[test]
    fn skips_common_script_setup_imports_without_macro_imports() {
        assert!(!script_rule_may_match(
            RULE_NO_IMPORT_COMPILER_MACROS,
            "import { ref, computed } from 'vue'"
        ));
        assert!(script_rule_may_match(
            RULE_NO_IMPORT_COMPILER_MACROS,
            "import { defineProps } from 'vue'"
        ));
        assert!(script_rule_may_match(
            RULE_NO_IMPORT_COMPILER_MACROS,
            "import { defineProps } from\n'vue'"
        ));
        assert!(script_rule_may_match(
            RULE_NO_IMPORT_COMPILER_MACROS,
            "import { defineProps } from /* compiler macros */ 'vue'"
        ));
    }

    #[test]
    fn keeps_export_default_with_trivia_for_component_options_rules() {
        assert!(script_rule_may_match(
            RULE_NO_ARROW_FUNCTIONS_IN_WATCH,
            "export /* component */ default { watch: { value: () => {} } }"
        ));
    }

    #[test]
    fn skips_define_props_rule_without_define_props_call() {
        assert!(!script_rule_may_match(
            RULE_VALID_DEFINE_PROPS,
            "import { ref } from 'vue'\nconst count = ref(0)"
        ));
        assert!(script_rule_may_match(
            RULE_VALID_DEFINE_PROPS,
            "defineProps<{ value: string }>()"
        ));
    }
}
