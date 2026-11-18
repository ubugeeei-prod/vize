use vize_croquis::macros::{
    DEFINE_EMITS, DEFINE_EXPOSE, DEFINE_MODEL, DEFINE_PROPS, DEFINE_SLOTS, WITH_DEFAULTS,
};

pub(crate) const USE_TEMPLATE_REF: &str = "useTemplateRef";

/// Names declared by the generated setup-scope helper block.
///
/// This includes Vue compiler macros plus runtime helper shims that are modeled
/// inside `__setup()`. It is intentionally broader than `COMPILER_MACRO_NAMES`.
pub(crate) const SETUP_SCOPE_HELPER_NAMES: &[&str] = &[
    DEFINE_PROPS,
    DEFINE_EMITS,
    DEFINE_EXPOSE,
    DEFINE_MODEL,
    DEFINE_SLOTS,
    WITH_DEFAULTS,
    USE_TEMPLATE_REF,
];

/// Shared type-helper text used both by the per-file embedded preamble and by
/// the hoisted ambient helpers file. Declared as a macro so the exact same
/// bytes can be spliced into both constants at compile time.
/// Kebab-case conversion accumulates its output before recursing, allowing
/// TypeScript to eliminate the recursion for long authored prop/event keys.
/// The shared preamble names its own prop-key text; embedded files keep theirs.
macro_rules! vue_type_aliases_text {
    () => {
        vue_type_aliases_text!("declarations/prop-keys.d.ts.txt")
    };
    ($prop_keys:literal) => {
        concat!(
            include_str!("declarations/core-types.d.ts.txt"),
            include_str!("declarations/vue3-types.d.ts.txt"),
            include_str!($prop_keys),
            "\n",
            include_str!("../component_prop_helpers.txt")
        )
    };
}

// Select the array type before its tuple element so generic object iteration
// retains its constraint (#6040).
macro_rules! v_for_list_decls_text {
    () => {
        include_str!("declarations/for-list.d.ts.txt")
    };
}

// Preserve the original generic hook instead of extracting its value parameter.
// Vue supplies up to four hook arguments; the tail follows the hook's arity so
// shorter hooks still receive contextual value checking without TS2554/TS2556.
macro_rules! directive_helpers_text {
    () => {
        include_str!("declarations/directives.d.ts.txt")
    };
}

// Keep the data-attribute mapping inline in the TypeScript asset: diagnostics
// should expand the public shape rather than name a nested implementation alias.
macro_rules! vue_type_helpers_text {
    () => {
        vue_type_helpers_text!("declarations/prop-keys.d.ts.txt")
    };
    ($prop_keys:literal) => {
        concat!(
            vue_type_aliases_text!($prop_keys),
            include_str!("declarations/native-elements.d.ts.txt"),
            include_str!("declarations/component-global-attributes.d.ts.txt"),
            v_for_list_decls_text!(),
            directive_helpers_text!()
        )
    };
}

macro_rules! content_mapper_type_helpers_text {
    ($types:expr, $module:expr) => {
        concat!($types, include_str!("declarations/native-elements.d.ts.txt"), "import type { VizeComponentAttrCamel as $ComponentAttrCamel, VizeComponentGlobalHtmlAttrs as $ComponentGlobalHtmlAttrs } from \"", $module, "\";\n", v_for_list_decls_text!(), directive_helpers_text!())
    };
}

/// Emit-overload helper text shared between the per-file embedded emission and
/// the hoisted ambient helpers file.
macro_rules! emit_overload_helpers_text {
    () => {
        include_str!("declarations/emit-overloads.d.ts.txt")
    };
}

pub const VUE_TYPE_HELPERS: &str = vue_type_helpers_text!();
pub(crate) const CONTENT_MAPPER_TYPE_HELPERS: &str = content_mapper_type_helpers_text!(
    include_str!("declarations/content-mapper-type-imports.d.ts.txt"),
    "vize/content-mapper-helpers"
);
pub(crate) const LEGACY_CONTENT_MAPPER_TYPE_HELPERS: &str =
    content_mapper_type_helpers_text!(vue_type_aliases_text!(), "vize/content-mapper-helpers");
pub(crate) const EMIT_OVERLOAD_HELPERS: &str = emit_overload_helpers_text!();
/// Vue's `EmitsToProps` keys handlers as `on${Capitalize<K>}` (`onScroll-top`);
/// the camelized key is the spelling templates bind, so both are accepted.
pub(crate) const EMIT_PROPS_HELPER: &str = include_str!("declarations/emit-props.d.ts.txt");

pub const SHARED_PREAMBLE_FILE_NAME: &str = "__vize_helpers.d.ts";

/// The program-wide checker preamble. vue-tsc declares its props fallback only
/// for files that leave `checkUnknownProps` off, so the fallback is a segment.
macro_rules! shared_preamble_text {
    ($props_fallback:expr) => {
        concat!(
            include_str!("declarations/shared-header.d.ts.txt"),
            $props_fallback,
            include_str!("declarations/shared-helpers-banner.d.ts.txt"),
            vue_type_helpers_text!("declarations/shared-prop-keys.d.ts.txt"),
            include_str!("declarations/shared-template-refs.d.ts.txt"),
            include_str!("declarations/shared-slot-resolver.d.ts.txt"),
            include_str!("declarations/shared-slot-payload.d.ts.txt"),
            emit_overload_helpers_text!(),
            include_str!("declarations/shared-emit-props.d.ts.txt"),
            include_str!("declarations/shared-emit-inference.d.ts.txt"),
            include_str!("declarations/shared-listener-args.d.ts.txt"),
            include_str!("declarations/component-checks.d.ts.txt"),
            include_str!("declarations/shared-macros.d.ts.txt"),
            include_str!("pattern_matching.d.ts")
        )
    };
}

/// Preamble of a program with at least one file that leaves `checkUnknownProps` off.
pub const SHARED_PREAMBLE_DTS: &str =
    shared_preamble_text!(include_str!("declarations/props-fallback.d.ts.txt"));
/// Preamble of a program whose every file checks unknown props (#11628).
pub const SHARED_PREAMBLE_STRICT_DTS: &str = shared_preamble_text!("");

pub const DECLARATION_HELPERS_DTS: &str = concat!(
    "// Shared helper types for vize-generated declaration files.\n",
    "// Generated by vize\n",
    vue_type_aliases_text!(),
    emit_overload_helpers_text!(),
    include_str!("declarations/emit-props.d.ts.txt"),
    // Generic component signatures name this; emitted declarations keep it
    // open instead of depending on Vue's `NativeElements` (native_props.rs).
    "type $ComponentGlobalHtmlAttrs = { [key: string]: unknown };\n",
);
