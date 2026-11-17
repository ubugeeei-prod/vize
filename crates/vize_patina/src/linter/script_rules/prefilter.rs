//! Byte-level prefilters that decide whether a script block can possibly match
//! an enabled built-in rule before the block is parsed.
//!
//! Split out of the parent module so it stays focused on rule dispatch: the
//! per-rule substring gate ([`script_rule_may_match`]) and the ecosystem-rule
//! fast path both live here.

use vize_atelier_sfc::SfcDescriptor;

mod js_source;
mod markers;
mod ref_operand;

use super::registry::{
    RULE_DEFINE_EMITS_DECLARATION, RULE_DEFINE_PROPS_DECLARATION, RULE_NO_ARROW_FUNCTIONS_IN_WATCH,
    RULE_NO_ASYNC_IN_COMPUTED, RULE_NO_COMPUTED_PROPERTIES_IN_DATA,
    RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION, RULE_NO_DEPRECATED_DESTROYED_LIFECYCLE,
    RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API, RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API,
    RULE_NO_DEPRECATED_EVENTS_API, RULE_NO_DEPRECATED_MODEL_DEFINITION,
    RULE_NO_DEPRECATED_PROPS_DEFAULT_THIS, RULE_NO_DUPE_KEYS, RULE_NO_DUPLICATE_ATTR_INHERITANCE,
    RULE_NO_EXPOSE_AFTER_AWAIT, RULE_NO_IMPORT_COMPILER_MACROS, RULE_NO_INTERNAL_IMPORTS,
    RULE_NO_LIFECYCLE_AFTER_AWAIT, RULE_NO_MULTIPLE_SLOT_ARGS,
    RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO, RULE_NO_REF_AS_OPERAND,
    RULE_NO_REQUIRED_PROP_WITH_DEFAULT, RULE_NO_RESERVED_IDENTIFIERS, RULE_NO_RESERVED_KEYS,
    RULE_NO_RESERVED_PROPS, RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS,
    RULE_NO_SIDE_EFFECTS_IN_COMPUTED, RULE_NO_UNSTABLE_NESTED_COMPONENTS,
    RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD, RULE_NO_WATCH_AFTER_AWAIT,
    RULE_NUXT_CONFIG_KEYS_ORDER, RULE_NUXT_NO_CONFIG_TEST_KEY,
    RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES, RULE_NUXT_PREFER_IMPORT_META,
    RULE_ONE_COMPONENT_PER_FILE, RULE_ORDER_IN_COMPONENTS, RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_PREFER_COMPUTED, RULE_PREFER_DEFINE_EMITS_BINDING, RULE_PREFER_IMPORT_FROM_VUE,
    RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR, RULE_REQUIRE_PROP_TYPES, RULE_REQUIRE_RENDER_RETURN,
    RULE_REQUIRE_SLOTS_AS_FUNCTIONS, RULE_REQUIRE_TYPED_OBJECT_PROP, RULE_REQUIRE_TYPED_REF,
    RULE_REQUIRE_VALID_DEFAULT_PROP, RULE_RETURN_IN_COMPUTED_PROPERTY,
    RULE_RETURN_IN_EMITS_VALIDATOR, RULE_VALID_DEFINE_EMITS, RULE_VALID_DEFINE_OPTIONS,
    RULE_VALID_DEFINE_PROPS, RULE_VALID_NEXT_TICK, RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
    RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
};
use super::{Linter, active_builtin_script_rule_entries};
pub(super) use markers::BlockMarkers;
use ref_operand::source_may_contain_ref_operand;

#[cfg(test)]
pub(super) fn script_rule_may_match(rule_name: &str, source: &str) -> bool {
    script_rule_may_match_in(rule_name, &BlockMarkers::new(source))
}

/// Whether `rule_name` can match the block `markers` searches.
pub(super) fn script_rule_may_match_in(rule_name: &str, markers: &BlockMarkers<'_>) -> bool {
    match rule_name {
        RULE_NO_ASYNC_IN_COMPUTED => {
            contains(markers, b"computed")
                && contains_any(
                    markers,
                    &[
                        b"async",
                        b"await",
                        b"$nextTick",
                        b"fetch",
                        b"setTimeout",
                        b"setInterval",
                        b"requestAnimationFrame",
                        b"setImmediate",
                        b"queueMicrotask",
                        b"Promise",
                        b".then",
                        b".catch",
                        b".finally",
                    ],
                )
        }
        RULE_NO_IMPORT_COMPILER_MACROS => {
            markers.imports_from_vue() && contains_any(markers, COMPILER_MACRO_NEEDLES)
        }
        RULE_NO_INTERNAL_IMPORTS => {
            contains(markers, b"vue") && contains_any(markers, INTERNAL_IMPORT_NEEDLES)
        }
        RULE_NO_RESERVED_IDENTIFIERS => contains_any(markers, RESERVED_IDENTIFIER_NEEDLES),
        RULE_NO_DUPE_KEYS => {
            markers.may_define_component_options()
                || contains_any(markers, &[b"defineProps", b"defineModel"])
        }
        RULE_NO_RESERVED_KEYS => {
            markers.may_define_component_options() || contains(markers, b"defineProps")
        }
        RULE_NO_POTENTIAL_COMPONENT_OPTION_TYPO => markers.may_define_component_options(),
        RULE_NO_SIDE_EFFECTS_IN_COMPUTED => {
            (markers.may_define_component_options()
                && contains(markers, b"computed")
                && contains(markers, b"this"))
                || ((markers.imports_from_vue()
                    || contains(markers, b"@vue/composition-api")
                    || (contains(markers, b"import") && contains(markers, b"\\")))
                    && (contains(markers, b"computed") || contains(markers, b"\\")))
        }
        RULE_NO_ARROW_FUNCTIONS_IN_WATCH => {
            markers.may_define_component_options()
                && contains(markers, b"watch")
                && contains(markers, b"=>")
        }
        RULE_NO_REQUIRED_PROP_WITH_DEFAULT => {
            // Typed defaults express requiredness in the type and may use an
            // assignment pattern rather than a literal `default` member. A
            // normal sibling script can own that local type while the setup
            // block owns the default, so retain type-declaration blocks for
            // cross-block resolution too.
            contains(markers, b"defineProps")
                || contains(markers, b"interface")
                || contains(markers, b"type ")
                || (source_may_declare_props(markers)
                    && contains(markers, b"required")
                    && contains(markers, b"default"))
        }
        RULE_REQUIRE_PROP_TYPE_CONSTRUCTOR => source_may_declare_props(markers),
        RULE_NO_RESERVED_PROPS => {
            source_may_declare_props(markers) && contains_any(markers, RESERVED_PROP_NEEDLES)
        }
        RULE_RETURN_IN_EMITS_VALIDATOR => {
            // Method shorthand and trivia before an arrow body have no fixed
            // function spelling. The existing AST producer owns that decision.
            // Setup calls can refer to a normal-script const without a brace
            // in this block. Escaped identifiers are decoded by the AST; the
            // conservative escape candidate never proves macro ownership.
            contains(markers, b"defineEmits")
                || contains(markers, b"\\u")
                || (source_may_declare_emits(markers) && contains(markers, b"{"))
        }
        RULE_REQUIRE_VALID_DEFAULT_PROP => {
            // `withDefaults` and `defineModel<T>` carry a default without the
            // words `type` and `default` both appearing in the script.
            contains(markers, b"defineModel")
                || contains(markers, b"withDefaults")
                || contains(markers, b"defineProps")
                || (source_may_declare_props(markers) && contains(markers, b"default"))
        }
        // A sibling `export default { props }` decides whether an empty macro
        // declares nothing, so that block is parsed too.
        RULE_VALID_DEFINE_PROPS => contains(markers, b"defineProps") || contains(markers, b"props"),
        RULE_VALID_DEFINE_EMITS => contains(markers, b"defineEmits") || contains(markers, b"emits"),
        RULE_PREFER_DEFINE_EMITS_BINDING => {
            contains(markers, b"defineEmits") || contains(markers, b"\\u")
        }
        RULE_VALID_DEFINE_OPTIONS => contains(markers, b"defineOptions"),
        RULE_VALID_NEXT_TICK => contains(markers, b"nextTick") || contains(markers, b"$nextTick"),
        RULE_NO_LIFECYCLE_AFTER_AWAIT | RULE_NO_WATCH_AFTER_AWAIT => {
            contains(markers, b"await") && contains(markers, b"setup")
        }
        RULE_NO_EXPOSE_AFTER_AWAIT => contains(markers, b"await") && contains(markers, b"xpose"),
        RULE_NO_REF_AS_OPERAND => source_may_contain_ref_operand(markers.source()),
        RULE_RETURN_IN_COMPUTED_PROPERTY => {
            contains(markers, b"computed") && contains(markers, b"{")
        }
        RULE_NO_USE_COMPUTED_PROPERTY_LIKE_METHOD => {
            markers.may_define_component_options() && contains(markers, b"computed")
        }
        RULE_NO_DUPLICATE_ATTR_INHERITANCE => true,
        RULE_NO_MULTIPLE_SLOT_ARGS => {
            contains(markers, b"slots")
                || contains(markers, b"$slots")
                || contains(markers, b"useSlots")
        }
        RULE_NO_UNSTABLE_NESTED_COMPONENTS => {
            markers.may_define_component_options()
                && (contains(markers, b"components") || contains(markers, b"defineComponent"))
        }
        RULE_PINIA_PREFER_STORE_TO_REFS => contains(markers, b"Store"),
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH => {
            (contains(markers, b".push") || contains(markers, b".replace"))
                && (contains(markers, b"'/") || contains(markers, b"\"/"))
                && (contains(markers, b"router") || contains(markers, b"Router"))
        }
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT => {
            contains(markers, b"toMatchSnapshot") && contains(markers, b".html")
        }
        RULE_NUXT_NO_PAGE_META_RUNTIME_VALUES => contains(markers, b"definePageMeta"),
        RULE_NUXT_NO_CONFIG_TEST_KEY | RULE_NUXT_CONFIG_KEYS_ORDER => contains(markers, b"export"),
        RULE_PREFER_COMPUTED => contains(markers, b"watch"),
        RULE_PREFER_IMPORT_FROM_VUE => contains(markers, b"@vue/"),
        RULE_NUXT_PREFER_IMPORT_META => contains(markers, b"process"),
        RULE_NO_RESTRICTED_GLOBALS => {
            contains(markers, b"process")
                || contains(markers, b"localStorage")
                || contains(markers, b"sessionStorage")
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
fn contains(markers: &BlockMarkers<'_>, needle: &'static [u8]) -> bool {
    markers.contains(needle)
}

fn contains_any(markers: &BlockMarkers<'_>, needles: &[&'static [u8]]) -> bool {
    needles.iter().any(|needle| markers.contains(needle))
}

fn source_may_declare_props(markers: &BlockMarkers<'_>) -> bool {
    contains(markers, b"defineProps")
        || (markers.may_define_component_options() && contains(markers, b"props"))
}

fn source_may_declare_emits(markers: &BlockMarkers<'_>) -> bool {
    contains(markers, b"defineEmits")
        || (markers.may_define_component_options() && contains(markers, b"emits"))
}

/// Whether a built-in script rule applies to the current file.
///
/// The Nuxt config ordering rule mirrors an upstream rule that is activated by
/// config overrides. Native presets do not have an override layer, so preserve
/// that boundary here instead of treating every default export as Nuxt config.
pub(super) fn script_rule_applies_to_filename(rule_name: &str, filename: &str) -> bool {
    rule_name != RULE_NUXT_CONFIG_KEYS_ORDER || is_nuxt_config_filename(filename)
}

/// Rules that ask for type syntax (`defineProps<T>()`, `ref<T>()`) have no
/// JavaScript fix, so they only apply to TypeScript blocks, as in
/// eslint-plugin-vue.
pub(super) fn script_rule_applies_to_language(rule_name: &str, typescript: bool) -> bool {
    typescript
        || !matches!(
            rule_name,
            RULE_DEFINE_PROPS_DECLARATION
                | RULE_DEFINE_EMITS_DECLARATION
                | RULE_REQUIRE_TYPED_OBJECT_PROP
                | RULE_REQUIRE_TYPED_REF
        )
}

/// Whether a standalone script file is TypeScript.
pub(super) fn is_typescript_filename(filename: &str) -> bool {
    [".ts", ".tsx", ".mts", ".cts"]
        .iter()
        .any(|extension| filename.ends_with(extension))
}

/// Whether a standalone script filename is one of Vue's JSX component hosts.
pub(super) fn is_jsx_filename(filename: &str) -> bool {
    [".jsx", ".tsx"]
        .iter()
        .any(|extension| filename.ends_with(extension))
}

/// Vue 2 migration rules and `require-prop-types` inspect component code only:
/// an SFC or JSX module (as in eslint-plugin-vue) or one that calls
/// `defineComponent`. A `$on`, `data: {}` or `props` object in any other module
/// is not a Vue component.
pub(super) fn script_rule_applies_to_module(
    rule_name: &str,
    markers: &BlockMarkers<'_>,
    filename: &str,
) -> bool {
    !matches!(
        rule_name,
        RULE_NO_COMPUTED_PROPERTIES_IN_DATA
            | RULE_NO_DEPRECATED_DATA_OBJECT_DECLARATION
            | RULE_NO_DEPRECATED_DESTROYED_LIFECYCLE
            | RULE_NO_DEPRECATED_DOLLAR_LISTENERS_API
            | RULE_NO_DEPRECATED_DOLLAR_SCOPEDSLOTS_API
            | RULE_NO_DEPRECATED_EVENTS_API
            | RULE_NO_DEPRECATED_MODEL_DEFINITION
            | RULE_NO_DEPRECATED_PROPS_DEFAULT_THIS
            | RULE_ONE_COMPONENT_PER_FILE
            | RULE_ORDER_IN_COMPONENTS
            | RULE_REQUIRE_PROP_TYPES
            | RULE_REQUIRE_RENDER_RETURN
            | RULE_REQUIRE_SLOTS_AS_FUNCTIONS
    ) || [".vue", ".jsx", ".tsx"]
        .iter()
        .any(|extension| filename.ends_with(extension))
        || markers.contains(b"defineComponent")
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
    active_builtin_script_rule_entries(linter, true)
        .all(|entry| is_ecosystem_script_rule(entry.rule_name))
}

fn source_may_match_ecosystem_rule(source: &str) -> bool {
    let markers = BlockMarkers::new(source);
    [
        RULE_PINIA_PREFER_STORE_TO_REFS,
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH,
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
    ]
    .into_iter()
    .any(|rule_name| script_rule_may_match_in(rule_name, &markers))
}

#[cfg(test)]
mod tests;
