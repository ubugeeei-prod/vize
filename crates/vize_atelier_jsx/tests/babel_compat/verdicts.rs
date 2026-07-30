//! Declared per-case verdicts for the `@vue/babel-plugin-jsx` differential
//! oracle. See `../babel_compat_oracle.rs` for what a verdict means and
//! `../BABEL_COMPAT_INVENTORY.md` for the prose form of the same table,
//! including the global divergences that are deliberately *not* repeated on
//! every row (module shape, block tree, always-on patch flags).
//!
//! Reasons here are short labels; the inventory row carries the full prose.

use super::Verdict;
use super::Verdict::{Deferred as Todo, Divergent as Diff, Equivalent as Same};

/// One entry per corpus case, in corpus order.
pub const VERDICTS: &[(&str, Verdict)] = &[
    // -- options ---------------------------------------------------------
    ("options/transform_on_off", Same),
    ("options/transform_on_on", Diff("no transformOn option")),
    ("options/pragma", Diff("no pragma option")),
    ("options/merge_props_default", Same),
    ("options/merge_props_false", Diff("no mergeProps option")),
    ("options/is_custom_element_default", Same),
    (
        "options/is_custom_element_fn",
        Diff("no isCustomElement option"),
    ),
    (
        "options/object_slots_default",
        Diff("lone expression child stringified"),
    ),
    (
        "options/object_slots_false",
        Diff("lone expression child stringified"),
    ),
    ("options/resolve_type_off", Same),
    (
        "options/resolve_type_on",
        Todo("type-driven props/emits needs #1497 / #1502"),
    ),
    // -- elements --------------------------------------------------------
    ("elements/intrinsic", Same),
    ("elements/component_pascal", Same),
    (
        "elements/unknown_lowercase",
        Diff("unknown tag stays intrinsic"),
    ),
    ("elements/dashed_lowercase", Same),
    ("elements/svg_tag", Same),
    ("elements/mathml_tag", Diff("unknown tag stays intrinsic")),
    (
        "elements/member_tag",
        Diff("member tag resolved as a name string"),
    ),
    (
        "elements/namespaced_tag",
        Diff("namespaced tag accepted, babel rejects"),
    ),
    ("elements/fragment", Same),
    (
        "elements/nested_fragment_child",
        Diff("nested fragment resolved by name"),
    ),
    // -- props -----------------------------------------------------------
    ("props/static_attr", Same),
    (
        "props/boolean_attr",
        Diff("valueless attribute lowers to \"\" not true"),
    ),
    ("props/dynamic_attr", Same),
    ("props/dashed_attrs", Same),
    (
        "props/xlink_camel",
        Diff("xlinkHref not rewritten to xlink:href"),
    ),
    ("props/xlink_colon", Same),
    ("props/class_dynamic", Same),
    ("props/class_static_and_dynamic", Same),
    ("props/style_dynamic", Same),
    ("props/style_merge_with_spread", Same),
    ("props/spread_only", Same),
    ("props/spread_then_static", Same),
    ("props/on_merge_with_spread", Same),
    ("props/key", Same),
    ("props/ref", Same),
    ("props/ref_in_for", Same),
    ("props/dollar_prefixed", Same),
    // -- events ----------------------------------------------------------
    ("events/plain", Same),
    ("events/capture", Same),
    ("events/once", Same),
    ("events/capture_passive", Same),
    // -- directives ------------------------------------------------------
    ("directives/v_model_input", Same),
    (
        "directives/v_model_arg",
        Diff("v-model arg rejected on elements"),
    ),
    ("directives/v_model_modifier_array", Same),
    ("directives/v_model_underscore", Same),
    (
        "directives/v_model_arg_underscore",
        Diff("v-model arg rejected on elements"),
    ),
    ("directives/v_model_component", Same),
    ("directives/v_model_component_arg_mods", Same),
    ("directives/v_models", Diff("v-models not implemented")),
    ("directives/v_models_mods", Diff("v-models not implemented")),
    ("directives/v_show_element", Same),
    ("directives/v_show_component", Same),
    ("directives/v_html", Same),
    ("directives/v_html_with_children", Same),
    (
        "directives/v_text",
        Diff("v-text wrapped in toDisplayString"),
    ),
    ("directives/v_custom_arg", Same),
    (
        "directives/v_custom_array",
        Diff("custom-directive array form not unpacked"),
    ),
    ("directives/v_dashed_custom", Same),
    // -- slots -----------------------------------------------------------
    ("slots/object_children", Same),
    (
        "slots/render_prop_child",
        Diff("non-JSX render-prop body dropped"),
    ),
    ("slots/scoped_param", Same),
    (
        "slots/v_slots_with_children",
        Diff("v-slots not implemented"),
    ),
    ("slots/v_slots_only", Diff("v-slots not implemented")),
    ("slots/element_children_default", Same),
    (
        "slots/dynamic_slot_name",
        Todo("computed slot names warn and drop; needs dynamic-slot lowering"),
    ),
    // -- children --------------------------------------------------------
    ("children/static_text", Same),
    ("children/text_interp_mix", Same),
    ("children/comment_only", Same),
    ("children/empty_expr", Same),
    ("children/spread_child", Diff("spread child stringified")),
    ("children/logical_and", Same),
    ("children/ternary", Same),
    ("children/map_list", Same),
    // -- optimize (babel optimize:true vs Vize's always-optimized default)
    ("optimize/static", Same),
    ("optimize/class_only", Same),
    ("optimize/style_only", Same),
    (
        "optimize/text_only",
        Diff("interpolated child becomes a TEXT child"),
    ),
    (
        "optimize/class_and_props",
        Diff("interpolated child becomes a TEXT child"),
    ),
    ("optimize/spread", Same),
    ("optimize/ref", Same),
    ("optimize/key", Same),
    ("optimize/event", Same),
    ("optimize/component_props", Same),
    ("optimize/v_model_input", Same),
    ("optimize/slots_stability", Same),
    ("optimize/scoped_slot_stability", Same),
    (
        "optimize/v_slots_stability",
        Diff("v-slots not implemented"),
    ),
    ("optimize/fragment", Same),
    ("optimize/map_list", Same),
    // -- errors ----------------------------------------------------------
    // Closed by #3420: both sides now reject a non-assignable target.
    ("errors/v_model_non_lval", Same),
    ("errors/v_model_no_value", Same),
    (
        "errors/v_models_not_array",
        Diff("v-models not implemented"),
    ),
    ("errors/v_slots_not_object", Diff("v-slots not implemented")),
];
