//! Tests for scope chain management.

use super::{
    BindingType, CallbackScopeData, ClientOnlyScopeData, CompactString, EventHandlerScopeData,
    ExternalModuleScopeData, JsGlobalScopeData, NonScriptSetupScopeData, ScopeBinding, ScopeChain,
    ScopeKind, ScriptSetupScopeData, UniversalScopeData, VForScopeData, VSlotScopeData,
    VueGlobalScopeData,
};
use crate::scope::types::JsRuntime;
use insta::assert_snapshot;
use vize_carton::append;

#[test]
fn test_scope_chain_basic() {
    let mut chain = ScopeChain::new();

    // Add binding to root scope
    chain.add_binding(
        CompactString::new("foo"),
        ScopeBinding::new(BindingType::SetupRef, 0),
    );

    assert!(chain.is_defined("foo"));
    assert!(!chain.is_defined("bar"));

    // Enter a new scope
    chain.enter_scope(ScopeKind::Function);
    chain.add_binding(
        CompactString::new("bar"),
        ScopeBinding::new(BindingType::SetupLet, 10),
    );

    // Can see both foo and bar
    assert!(chain.is_defined("foo"));
    assert!(chain.is_defined("bar"));

    // Exit scope
    chain.exit_scope();

    // Can only see foo now
    assert!(chain.is_defined("foo"));
    assert!(!chain.is_defined("bar"));
}

#[test]
fn test_scope_shadowing() {
    let mut chain = ScopeChain::new();

    chain.add_binding(
        CompactString::new("x"),
        ScopeBinding::new(BindingType::SetupRef, 0),
    );

    chain.enter_scope(ScopeKind::Block);
    chain.add_binding(
        CompactString::new("x"),
        ScopeBinding::new(BindingType::SetupLet, 10),
    );

    // Should find the inner binding
    let (scope, binding) = chain.lookup("x").unwrap();
    assert_eq!(scope.kind, ScopeKind::Block);
    assert_eq!(binding.binding_type, BindingType::SetupLet);
}

#[test]
fn test_v_for_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("item"),
            key_alias: Some(CompactString::new("key")),
            index_alias: Some(CompactString::new("index")),
            source: CompactString::new("items"),
            key_expression: Some(CompactString::new("item.id")),
        },
        0,
        100,
    );

    assert!(chain.is_defined("item"));
    assert!(chain.is_defined("key"));
    assert!(chain.is_defined("index"));
    assert!(!chain.is_defined("items")); // source is not a binding
}

#[test]
fn test_v_slot_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_v_slot_scope(
        VSlotScopeData {
            name: CompactString::new("default"),
            props_pattern: Some(CompactString::new("{ item, index }")),
            prop_names: vize_carton::smallvec![
                CompactString::new("item"),
                CompactString::new("index")
            ],
        },
        0,
        100,
    );

    assert!(chain.is_defined("item"));
    assert!(chain.is_defined("index"));
}

#[test]
fn test_event_handler_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new("click"),
            has_implicit_event: true,
            param_names: vize_carton::smallvec![],
            handler_expression: None,
            target_component: None,
        },
        0,
        50,
    );

    // $event should be available
    assert!(chain.is_defined("$event"));
}

#[test]
fn test_event_handler_scope_with_params() {
    let mut chain = ScopeChain::new();

    // @click="(e, extra) => handle(e, extra)"
    chain.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new("click"),
            has_implicit_event: false,
            param_names: vize_carton::smallvec![
                CompactString::new("e"),
                CompactString::new("extra")
            ],
            handler_expression: None,
            target_component: None,
        },
        0,
        50,
    );

    // Explicit params should be available
    assert!(chain.is_defined("e"));
    assert!(chain.is_defined("extra"));
    // $event should NOT be available (explicit params used)
    assert!(!chain.is_defined("$event"));
}

#[test]
fn test_callback_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_callback_scope(
        CallbackScopeData {
            param_names: vize_carton::smallvec![CompactString::new("item")],
            context: CompactString::new(":class callback"),
        },
        0,
        50,
    );

    // Callback param should be available
    assert!(chain.is_defined("item"));
    assert_eq!(chain.current_scope().kind, ScopeKind::Callback);
}

#[test]
fn test_nested_v_for() {
    let mut chain = ScopeChain::new();

    // Outer v-for
    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("row"),
            key_alias: None,
            index_alias: Some(CompactString::new("rowIndex")),
            source: CompactString::new("rows"),
            key_expression: None,
        },
        0,
        200,
    );

    // Inner v-for
    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("cell"),
            key_alias: None,
            index_alias: Some(CompactString::new("cellIndex")),
            source: CompactString::new("row.cells"),
            key_expression: None,
        },
        50,
        150,
    );

    // All bindings should be visible
    assert!(chain.is_defined("row"));
    assert!(chain.is_defined("rowIndex"));
    assert!(chain.is_defined("cell"));
    assert!(chain.is_defined("cellIndex"));

    // Exit inner
    chain.exit_scope();

    // Inner bindings no longer visible
    assert!(chain.is_defined("row"));
    assert!(chain.is_defined("rowIndex"));
    assert!(!chain.is_defined("cell"));
    assert!(!chain.is_defined("cellIndex"));
}

#[test]
fn test_nested_callback_in_v_for() {
    let mut chain = ScopeChain::new();

    // v-for="item in items"
    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("item"),
            key_alias: None,
            index_alias: Some(CompactString::new("index")),
            source: CompactString::new("items"),
            key_expression: None,
        },
        0,
        200,
    );

    // @click="(e) => handleClick(item, e)"
    chain.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new("click"),
            has_implicit_event: false,
            param_names: vize_carton::smallvec![CompactString::new("e")],
            handler_expression: None,
            target_component: None,
        },
        50,
        100,
    );

    // Both v-for bindings and event params should be visible
    assert!(chain.is_defined("item"));
    assert!(chain.is_defined("index"));
    assert!(chain.is_defined("e"));
}

#[test]
fn test_script_setup_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::ScriptSetup);

    // Add some bindings in script setup
    chain.add_binding(
        CompactString::new("counter"),
        ScopeBinding::new(BindingType::SetupRef, 10),
    );
    chain.add_binding(
        CompactString::new("message"),
        ScopeBinding::new(BindingType::SetupConst, 20),
    );

    assert!(chain.is_defined("counter"));
    assert!(chain.is_defined("message"));
}

#[test]
fn test_non_script_setup_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_non_script_setup_scope(
        NonScriptSetupScopeData {
            is_ts: false,
            has_define_component: true,
        },
        0,
        500,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::NonScriptSetup);
}

#[test]
fn test_universal_scope() {
    let mut chain = ScopeChain::new();

    // Script setup scope (runs on both server and client)
    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );

    // Enter universal scope (e.g., setup() content before lifecycle hooks)
    chain.enter_universal_scope(
        UniversalScopeData {
            context: CompactString::new("setup"),
        },
        10,
        400,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::Universal);

    // Universal code should be able to access parent script setup bindings
    chain.exit_scope(); // Exit universal
    chain.add_binding(
        CompactString::new("sharedData"),
        ScopeBinding::new(BindingType::SetupReactiveConst, 50),
    );
    chain.enter_universal_scope(
        UniversalScopeData {
            context: CompactString::new("setup"),
        },
        60,
        400,
    );

    assert!(chain.is_defined("sharedData"));
}

#[test]
fn test_client_only_scope() {
    let mut chain = ScopeChain::new();

    // Script setup scope
    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );

    // Add binding in script setup
    chain.add_binding(
        CompactString::new("count"),
        ScopeBinding::new(BindingType::SetupRef, 10),
    );

    // Enter onMounted (client-only)
    chain.enter_client_only_scope(
        ClientOnlyScopeData {
            hook_name: CompactString::new("onMounted"),
        },
        100,
        200,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::ClientOnly);

    // Should be able to access parent bindings
    assert!(chain.is_defined("count"));

    chain.exit_scope();

    // Enter onBeforeUnmount (client-only)
    chain.enter_client_only_scope(
        ClientOnlyScopeData {
            hook_name: CompactString::new("onBeforeUnmount"),
        },
        250,
        300,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::ClientOnly);
    assert!(chain.is_defined("count"));
}

#[test]
fn test_js_global_universal_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Universal,
            globals: vize_carton::smallvec![
                CompactString::new("console"),
                CompactString::new("Math"),
                CompactString::new("Object"),
                CompactString::new("Array"),
            ],
        },
        0,
        0,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::JsGlobalUniversal);

    // All JS globals should be defined
    assert!(chain.is_defined("console"));
    assert!(chain.is_defined("Math"));
    assert!(chain.is_defined("Object"));
    assert!(chain.is_defined("Array"));

    // Check binding type
    let (_, binding) = chain.lookup("console").unwrap();
    assert_eq!(binding.binding_type, BindingType::JsGlobalUniversal);
}

#[test]
fn test_js_global_browser_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Browser,
            globals: vize_carton::smallvec![
                CompactString::new("window"),
                CompactString::new("document"),
                CompactString::new("navigator"),
                CompactString::new("localStorage"),
            ],
        },
        0,
        0,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::JsGlobalBrowser);

    // All browser globals should be defined
    assert!(chain.is_defined("window"));
    assert!(chain.is_defined("document"));
    assert!(chain.is_defined("navigator"));
    assert!(chain.is_defined("localStorage"));

    // Check binding type - should be browser-specific
    let (_, binding) = chain.lookup("window").unwrap();
    assert_eq!(binding.binding_type, BindingType::JsGlobalBrowser);
}

#[test]
fn test_js_global_node_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Node,
            globals: vize_carton::smallvec![
                CompactString::new("process"),
                CompactString::new("Buffer"),
                CompactString::new("__dirname"),
                CompactString::new("require"),
            ],
        },
        0,
        0,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::JsGlobalNode);

    // All Node.js globals should be defined
    assert!(chain.is_defined("process"));
    assert!(chain.is_defined("Buffer"));
    assert!(chain.is_defined("__dirname"));
    assert!(chain.is_defined("require"));

    // Check binding type - should be Node-specific
    let (_, binding) = chain.lookup("process").unwrap();
    assert_eq!(binding.binding_type, BindingType::JsGlobalNode);
}

#[test]
fn test_vue_global_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_vue_global_scope(
        VueGlobalScopeData {
            globals: vize_carton::smallvec![
                CompactString::new("$refs"),
                CompactString::new("$emit"),
                CompactString::new("$slots"),
                CompactString::new("$attrs"),
            ],
        },
        0,
        0,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::VueGlobal);

    // All Vue globals should be defined
    assert!(chain.is_defined("$refs"));
    assert!(chain.is_defined("$emit"));
    assert!(chain.is_defined("$slots"));
    assert!(chain.is_defined("$attrs"));

    // Check binding type
    let (_, binding) = chain.lookup("$refs").unwrap();
    assert_eq!(binding.binding_type, BindingType::VueGlobal);
}

#[test]
fn test_external_module_scope() {
    let mut chain = ScopeChain::new();

    chain.enter_external_module_scope(
        ExternalModuleScopeData {
            source: CompactString::new("vue"),
            is_type_only: false,
        },
        0,
        50,
    );

    assert_eq!(chain.current_scope().kind, ScopeKind::ExternalModule);

    // Add imports from external module
    chain.add_binding(
        CompactString::new("ref"),
        ScopeBinding::new(BindingType::ExternalModule, 10),
    );
    chain.add_binding(
        CompactString::new("computed"),
        ScopeBinding::new(BindingType::ExternalModule, 15),
    );

    assert!(chain.is_defined("ref"));
    assert!(chain.is_defined("computed"));

    let (_, binding) = chain.lookup("ref").unwrap();
    assert_eq!(binding.binding_type, BindingType::ExternalModule);
}

#[test]
fn test_nested_ssr_scopes() {
    let mut chain = ScopeChain::new();

    // Root: Universal JS Global
    chain.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Universal,
            globals: vize_carton::smallvec![CompactString::new("console")],
        },
        0,
        0,
    );

    // Vue global
    chain.enter_vue_global_scope(
        VueGlobalScopeData {
            globals: vize_carton::smallvec![CompactString::new("$emit")],
        },
        0,
        0,
    );

    // Script setup
    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );

    chain.add_binding(
        CompactString::new("count"),
        ScopeBinding::new(BindingType::SetupRef, 10),
    );

    // Universal scope (setup logic)
    chain.enter_universal_scope(
        UniversalScopeData {
            context: CompactString::new("setup-body"),
        },
        20,
        400,
    );

    // Client-only scope (onMounted)
    chain.enter_client_only_scope(
        ClientOnlyScopeData {
            hook_name: CompactString::new("onMounted"),
        },
        100,
        200,
    );

    // All scopes should be accessible
    assert!(chain.is_defined("console")); // JS global
    assert!(chain.is_defined("$emit")); // Vue global
    assert!(chain.is_defined("count")); // Script setup binding

    // Current scope is client-only
    assert_eq!(chain.current_scope().kind, ScopeKind::ClientOnly);
}

#[test]
fn test_scope_chain_snapshot() {
    let mut chain = ScopeChain::new();

    // Build a complex scope chain
    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );
    chain.add_binding(
        CompactString::new("count"),
        ScopeBinding::new(BindingType::SetupRef, 10),
    );

    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("item"),
            key_alias: Some(CompactString::new("key")),
            index_alias: None,
            source: CompactString::new("items"),
            key_expression: None,
        },
        100,
        200,
    );

    // Snapshot the scope chain structure
    let mut output = String::new();
    for scope in chain.iter() {
        append!(
            output,
            "Scope {} ({:?}): {} bindings\n",
            scope.id.as_u32(),
            scope.kind,
            scope.binding_count()
        );
        for (name, binding) in scope.bindings() {
            append!(output, "  - {}: {:?}\n", name, binding.binding_type);
        }
    }

    assert_snapshot!("scope_chain_structure", output);
}

#[test]
fn test_snapshot_complex_nested_scopes() {
    let mut chain = ScopeChain::new();

    // JS Global
    chain.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Universal,
            globals: vize_carton::smallvec![
                CompactString::new("console"),
                CompactString::new("setTimeout"),
                CompactString::new("fetch"),
            ],
        },
        0,
        0,
    );

    // Vue Global
    chain.enter_vue_global_scope(
        VueGlobalScopeData {
            globals: vize_carton::smallvec![
                CompactString::new("$refs"),
                CompactString::new("$emit"),
                CompactString::new("$attrs"),
            ],
        },
        0,
        0,
    );

    // Script Setup
    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: Some(CompactString::new("<T extends object>")),
        },
        0,
        1000,
    );

    chain.add_binding(
        CompactString::new("count"),
        ScopeBinding::new(BindingType::SetupRef, 10),
    );
    chain.add_binding(
        CompactString::new("items"),
        ScopeBinding::new(BindingType::SetupReactiveConst, 50),
    );
    chain.add_binding(
        CompactString::new("handleClick"),
        ScopeBinding::new(BindingType::SetupConst, 100),
    );

    // v-for scope
    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("item"),
            key_alias: Some(CompactString::new("key")),
            index_alias: Some(CompactString::new("index")),
            source: CompactString::new("items"),
            key_expression: Some(CompactString::new("item.id")),
        },
        200,
        400,
    );

    // Nested v-slot scope
    chain.enter_v_slot_scope(
        VSlotScopeData {
            name: CompactString::new("default"),
            props_pattern: Some(CompactString::new("{ row, col }")),
            prop_names: vize_carton::smallvec![
                CompactString::new("row"),
                CompactString::new("col")
            ],
        },
        250,
        350,
    );

    // Event handler inside
    chain.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new("click"),
            has_implicit_event: false,
            param_names: vize_carton::smallvec![CompactString::new("e")],
            handler_expression: None,
            target_component: None,
        },
        300,
        340,
    );

    let mut output = String::new();
    output.push_str("=== Complex Nested Scopes ===\n\n");

    output.push_str("-- All scopes (root to current) --\n");
    for (depth, scope) in chain.iter().enumerate() {
        let indent = "  ".repeat(depth);
        append!(
            output,
            "{}{:?} (id={})\n",
            indent,
            scope.kind,
            scope.id.as_u32(),
        );
        for (name, binding) in scope.bindings() {
            append!(
                output,
                "{}  • {}: {:?} at offset {}\n",
                indent,
                name,
                binding.binding_type,
                binding.declaration_offset
            );
        }
    }

    output.push_str("\n-- Lookup test --\n");
    for name in ["count", "item", "row", "e", "console", "unknown"] {
        if let Some((scope, binding)) = chain.lookup(name) {
            append!(
                output,
                "{}: found in {:?} (scope {}), type={:?}\n",
                name,
                scope.kind,
                scope.id.as_u32(),
                binding.binding_type
            );
        } else {
            append!(output, "{name}: not found\n");
        }
    }

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_scope_transitions() {
    let mut chain = ScopeChain::new();

    let mut output = String::new();
    output.push_str("=== Scope Transitions ===\n\n");

    // Track scope changes
    let log_state = |chain: &ScopeChain, output: &mut String, action: &str| {
        append!(
            *output,
            "[{}] current={:?}\n",
            action,
            chain.current_scope().kind,
        );
    };

    log_state(&chain, &mut output, "initial");

    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: false,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );
    log_state(&chain, &mut output, "enter script_setup");

    chain.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("item"),
            key_alias: None,
            index_alias: None,
            source: CompactString::new("list"),
            key_expression: None,
        },
        100,
        300,
    );
    log_state(&chain, &mut output, "enter v_for");

    chain.enter_v_slot_scope(
        VSlotScopeData {
            name: CompactString::new("default"),
            props_pattern: None,
            prop_names: vize_carton::smallvec![],
        },
        150,
        250,
    );
    log_state(&chain, &mut output, "enter v_slot");

    chain.exit_scope();
    log_state(&chain, &mut output, "exit v_slot");

    chain.exit_scope();
    log_state(&chain, &mut output, "exit v_for");

    chain.exit_scope();
    log_state(&chain, &mut output, "exit script_setup");

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_binding_types() {
    let mut chain = ScopeChain::new();

    chain.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: None,
        },
        0,
        500,
    );

    // Add various binding types
    let bindings = [
        ("refBinding", BindingType::SetupRef),
        ("letBinding", BindingType::SetupLet),
        ("constBinding", BindingType::SetupConst),
        ("reactiveConstBinding", BindingType::SetupReactiveConst),
        ("maybeRefBinding", BindingType::SetupMaybeRef),
        ("literalConstBinding", BindingType::LiteralConst),
        ("propsBinding", BindingType::Props),
    ];

    for (i, (name, binding_type)) in bindings.iter().enumerate() {
        chain.add_binding(
            CompactString::new(*name),
            ScopeBinding::new(*binding_type, i as u32 * 10),
        );
    }

    let mut output = String::new();
    output.push_str("=== Binding Types ===\n\n");

    for (name, _binding_type) in &bindings {
        let (_, binding) = chain.lookup(name).unwrap();
        append!(output, "{}: {:?}\n", name, binding.binding_type);
    }

    assert_snapshot!(output);
}
