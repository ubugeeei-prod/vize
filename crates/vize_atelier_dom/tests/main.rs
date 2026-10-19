//! The Atelier DOM integration tests, linked as one binary instead of one per file.

#[expect(
    dead_code,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; the feature-gated binaries use the rest"
)]
mod support;

mod branch_keys;
mod conditional_branch_roots;
mod conditional_v_model_quirks;
mod davinci_teardown_without_stack_guard;
mod dev_root_fragment;
mod dom_snapshot;
mod dropped_comment_text_runs;
mod dynamic_template_ref;
mod element_nesting_depth;
mod expression_nesting_guard;
mod invalid_expression_diagnostics;
mod l2_bind_modifiers;
mod l2_binding_metadata;
mod l2_builtins;
mod l2_cache_handlers;
mod l2_cache_slot_order;
mod l2_cached_props_multiline;
mod l2_cloak;
mod l2_colon;
mod l2_component_name;
mod l2_constant_style_binding;
mod l2_constant_text_run;
mod l2_corpus_residuals;
mod l2_dirs;
mod l2_dom;
mod l2_dom_namespace;
mod l2_dynamic;
mod l2_dynamic_bind_keys;
mod l2_dynamic_bind_options;
mod l2_dynamic_von_keys;
mod l2_expression_residuals;
mod l2_for;
mod l2_format_residuals;
mod l2_fragments;
mod l2_handler_parity;
mod l2_helper_composition;
mod l2_helper_order;
mod l2_hoist_order;
mod l2_html;
mod l2_if_for;
mod l2_inline;
mod l2_inline_root_hoist;
mod l2_inline_root_hoist_components;
mod l2_inline_template_ref;
mod l2_is_ts;
mod l2_keyed_component_block;
mod l2_memo;
mod l2_merged_const_handler;
mod l2_model;
mod l2_module_mode;
mod l2_multiline_props;
mod l2_native_on;
mod l2_once;
mod l2_option_audit;
mod l2_option_families;
mod l2_option_matrix;
mod l2_outlets;
mod l2_patch_flags;
mod l2_patch_flags_legacy;
mod l2_patch_flags_static;
mod l2_prefix_identifiers;
mod l2_recent_patch_flags;
mod l2_refs;
mod l2_rendering_pragmas;
mod l2_root_hoist;
mod l2_scope_id;
mod l2_sfc_comment_selector;
mod l2_show;
mod l2_slots;
mod l2_static_cache;
mod l2_static_class_patch;
mod l2_static_style;
mod l2_static_vnode_hoist;
mod l2_style_merge;
mod l2_template_vfor_text;
mod l2_template_wrapper_component_props;
mod l2_text;
mod l2_text_facts;
mod l2_tpl;
mod l2_unref_helper_order;
mod l2_v_for_child_props;
mod l2_v_pre;
mod l2_von;
mod l2_vslots;
mod l2_whitespace_text_vnode;
mod nested_interactive_recovery;
mod nested_scope_bindings;
mod production_capture;
mod public_experimental_options_literal;
mod setup_component_unref;
mod setup_dynamic_argument_selection;
mod slot_outlet_hoist;
mod table_control_flow;
mod template_syntax_quirks_recovery;
