//! A downstream-style literal from v0.429.1 must remain constructible.

use vize_l1_to_l2::{DomEmitMode, DomEmitOptions};

#[test]
fn published_dom_emit_options_literal_still_compiles() {
    let old_literal = DomEmitOptions {
        mode: DomEmitMode::Function,
        runtime_module_name: "vue",
        runtime_global_name: "Vue",
        prefix_identifiers: false,
        hoist_static: true,
        inline: false,
        component_name: None,
        cache_handlers: false,
        hoisted_scope_id: None,
        scope_id: None,
        is_ts: false,
        comments: false,
        experimental_in_tag_comments: false,
        custom_element_patterns: &[],
        custom_element_predicate: None,
        bindings: None,
    };
    assert_eq!(old_literal, DomEmitOptions::DEFAULT);
}
