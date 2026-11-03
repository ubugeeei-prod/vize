use super::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};

#[test]
fn default_options_project_the_shipped_codegen_defaults() {
    assert_eq!(
        DomEmitOptions::default(),
        DomEmitOptions {
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
        }
    );
}

#[test]
fn render_signatures_match_the_shipped_lane_per_mode() {
    assert_eq!(
        DomEmitMode::Function.render_signature(false),
        "function render(_ctx, _cache, $props, $setup, $data, $options) {"
    );
    assert_eq!(
        DomEmitMode::Module.render_signature(false),
        "export function render(_ctx, _cache) {"
    );
    assert_eq!(
        DomEmitMode::Module.render_signature(true),
        "export function render(_ctx, _cache, $props, $setup, $data, $options) {"
    );
}

#[test]
fn binding_table_lookups_and_alias_order() {
    let table = BindingTable::new(
        [
            ("msg", BindingKind::SetupRef),
            ("Comp", BindingKind::SetupConst),
            ("msg", BindingKind::SetupLet),
        ],
        [("local", "prop-key")],
        true,
    );
    assert_eq!(table.kind("msg"), Some(BindingKind::SetupLet));
    assert_eq!(table.kind("Comp"), Some(BindingKind::SetupConst));
    assert_eq!(table.kind("other"), None);
    let named = (table.contains("Comp"), table.contains("other"));
    assert_eq!(named, (true, false));
    let mut aliases = table.aliases();
    assert_eq!(aliases.next(), Some(("local", "prop-key")));
    assert_eq!(aliases.next(), None);
    assert!(table.is_script_setup());
}

#[test]
fn non_inline_prefixes_match_the_shipped_binding_types() {
    assert_eq!(
        BindingKind::SetupRef.non_inline_template_prefix(),
        "$setup."
    );
    assert_eq!(BindingKind::Props.non_inline_template_prefix(), "$props.");
    assert_eq!(BindingKind::Data.non_inline_template_prefix(), "$data.");
    assert_eq!(
        BindingKind::Options.non_inline_template_prefix(),
        "$options."
    );
    assert_eq!(BindingKind::VueGlobal.non_inline_template_prefix(), "_ctx.");
    assert_eq!(
        BindingKind::ExternalModule.non_inline_template_prefix(),
        "$setup."
    );
}
