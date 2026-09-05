//! Declared per-case verdicts for the `@vue/babel-plugin-jsx` differential
//! oracle. See `../babel_compat_oracle.rs` for what a verdict means and
//! `../BABEL_COMPAT_INVENTORY.md` for the prose form of the same table,
//! including the global divergences that are deliberately *not* repeated on
//! every row (module shape, block tree, always-on patch flags).
//!
//! Reasons here are short labels; the inventory row carries the full prose.

use super::Verdict;
use super::Verdict::{Deferred as Todo, Equivalent as Same};

/// One entry per corpus case, in corpus order.
pub const VERDICTS: &[(&str, Verdict)] = &[
    // -- options ---------------------------------------------------------
    ("options/transform_on_off", Same),
    ("options/transform_on_on", Same),
    ("options/pragma", Same),
    ("options/merge_props_default", Same),
    ("options/merge_props_false", Same),
    ("options/is_custom_element_default", Same),
    ("options/is_custom_element_fn", Same),
    // Closed by #3391: Babel compat distinguishes possible slot objects at
    // runtime by default, and preserves the lone expression as a raw
    // default-slot child when `enableObjectSlots` is disabled.
    ("options/object_slots_default", Same),
    ("options/object_slots_false", Same),
    ("options/resolve_type_off", Same),
    (
        "options/resolve_type_on",
        Todo("type-driven props/emits needs #1497 / #1502"),
    ),
    // -- elements --------------------------------------------------------
    ("elements/intrinsic", Same),
    ("elements/component_pascal", Same),
    ("elements/unknown_lowercase", Same),
    ("elements/dashed_lowercase", Same),
    ("elements/svg_tag", Same),
    ("elements/mathml_tag", Same),
    // Closed by #3421: a member tag names a component value, so it lowers to
    // `resolveDynamicComponent`, which passes a non-string through unchanged.
    ("elements/member_tag", Same),
    // Closed by #3421: an unknown tag namespace is rejected, as babel does.
    ("elements/namespaced_tag", Same),
    ("elements/fragment", Same),
    // Closed by #3421: the nested fragment's children are spliced into the
    // parent, mounting the same DOM as babel's nested `Fragment` vnode.
    ("elements/nested_fragment_child", Same),
    // -- props -----------------------------------------------------------
    ("props/static_attr", Same),
    ("props/boolean_attr", Same),
    ("props/dynamic_attr", Same),
    ("props/dashed_attrs", Same),
    ("props/xlink_camel", Same),
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
    ("directives/v_model_arg", Same),
    ("directives/v_model_modifier_array", Same),
    ("directives/v_model_underscore", Same),
    ("directives/v_model_arg_underscore", Same),
    ("directives/v_model_component", Same),
    ("directives/v_model_component_arg_mods", Same),
    // Closed by #3391: a dynamic argument on a component lowers to computed
    // prop and update-listener keys, as babel emits.
    ("directives/v_model_component_dynamic_arg", Same),
    // Closed by #3418: `v-models` expands to one model binding per entry.
    ("directives/v_models", Same),
    ("directives/v_models_mods", Same),
    ("directives/v_show_element", Same),
    ("directives/v_show_component", Same),
    ("directives/v_html", Same),
    ("directives/v_html_with_children", Same),
    ("directives/v_text", Same),
    ("directives/v_custom_arg", Same),
    ("directives/v_custom_array", Same),
    ("directives/v_dashed_custom", Same),
    // -- slots -----------------------------------------------------------
    ("slots/object_children", Same),
    ("slots/render_prop_child", Same),
    ("slots/scoped_param", Same),
    // #3418 lowers the object-literal form; #3467 forwards an opaque slots
    // value as a spread (or as the whole children argument when nothing else
    // contributes slots), with no `_` flag and a DYNAMIC_SLOTS vnode flag.
    ("slots/v_slots_with_children", Same),
    ("slots/v_slots_only", Same),
    ("slots/v_slots_object_literal", Same),
    ("slots/v_slots_object_with_children", Same),
    ("slots/element_children_default", Same),
    ("slots/dynamic_slot_name", Same),
    // -- children --------------------------------------------------------
    ("children/static_text", Same),
    ("children/text_interp_mix", Same),
    ("children/comment_only", Same),
    ("children/empty_expr", Same),
    ("children/spread_child", Same),
    ("children/logical_and", Same),
    ("children/ternary", Same),
    ("children/map_list", Same),
    // -- optimize (babel optimize:true vs Vize's always-optimized default)
    ("optimize/static", Same),
    ("optimize/class_only", Same),
    ("optimize/style_only", Same),
    ("optimize/text_only", Same),
    ("optimize/class_and_props", Same),
    ("optimize/spread", Same),
    ("optimize/ref", Same),
    ("optimize/key", Same),
    ("optimize/event", Same),
    ("optimize/component_props", Same),
    ("optimize/v_model_input", Same),
    ("optimize/slots_stability", Same),
    ("optimize/scoped_slot_stability", Same),
    // Babel emits no `_` flag beside a forwarded slots object even under
    // `optimize: true`, and neither does Vize (#3467).
    ("optimize/v_slots_stability", Same),
    ("optimize/fragment", Same),
    ("optimize/map_list", Same),
    // -- errors ----------------------------------------------------------
    // Closed by #3420: both sides now reject a non-assignable target.
    ("errors/v_model_non_lval", Same),
    ("errors/v_model_no_value", Same),
    // Closed by #3418: both sides now reject a `v-models` that is not a
    // two-dimensional array.
    ("errors/v_models_not_array", Same),
    ("errors/v_models_entry_not_array", Same),
    // Closed by #3391: Babel VDOM mode forwards the primitive literals babel
    // passes straight into the vnode's children argument.
    ("errors/v_slots_not_object", Same),
    ("errors/v_slots_static_template", Same),
    // Authored children make Babel spread the primitive after the default slot.
    ("errors/v_slots_not_object_with_children", Same),
];
