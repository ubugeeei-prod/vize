use super::Croquis;
use crate::BindingType;
use crate::hoist::HoistLevel;
use crate::macros::{
    EmitDefinition, ExposeDefinition, MacroKind, ModelDefinition, PropDefinition, SlotsDefinition,
};
use crate::provide::{InjectPattern, ProvideKey};
use crate::race::RaceConditionRiskKind;
use crate::reactivity::{ReactiveKind, ReactivityLoss, ReactivityLossKind};
use crate::scope::{EventHandlerScopeData, ScopeId, VForScopeData, VSlotScopeData};
use crate::setup_context::SetupContextViolationKind;
use vize_carton::{CompactString, ToCompactString, smallvec};

#[test]
fn semantic_summary_collects_core_croquis_counts() {
    let mut croquis = Croquis::new();
    croquis.bindings.add("count", BindingType::SetupRef);
    croquis
        .bindings
        .props_aliases
        .insert(CompactString::new("message"), CompactString::new("msg"));

    let symbol_id = croquis.symbols.add_symbol(
        "count".to_compact_string(),
        BindingType::SetupRef,
        ScopeId::ROOT,
        10,
    );
    croquis.symbols.add_reference("count", 42);
    assert!(croquis.symbols.get(symbol_id).is_some());

    croquis.scopes.enter_v_for_scope(
        VForScopeData {
            value_alias: CompactString::new("item"),
            value_bindings: smallvec![CompactString::new("item")],
            key_alias: None,
            index_alias: Some(CompactString::new("index")),
            source: CompactString::new("items"),
            key_expression: Some(CompactString::new("item.id")),
        },
        1,
        20,
    );
    croquis.scopes.exit_scope();
    croquis.scopes.enter_v_slot_scope(
        VSlotScopeData {
            name: CompactString::new("default"),
            props_pattern: None,
            prop_names: smallvec![CompactString::new("row")],
            component: Some(CompactString::new("Child")),
        },
        21,
        40,
    );
    croquis.scopes.exit_scope();
    croquis.scopes.enter_event_handler_scope(
        EventHandlerScopeData {
            event_name: CompactString::new("click"),
            has_implicit_event: true,
            param_names: smallvec![],
            handler_expression: Some(CompactString::new("onClick($event)")),
            target_component: None,
        },
        41,
        50,
    );

    croquis
        .macros
        .add_call("defineProps", MacroKind::DefineProps, 0, 12, None, None);
    croquis.macros.add_prop(PropDefinition {
        name: CompactString::new("msg"),
        prop_type: None,
        required: true,
        default_value: None,
    });
    croquis.macros.add_emit(EmitDefinition {
        name: CompactString::new("save"),
        payload_type: None,
    });
    croquis
        .macros
        .add_emit_call(CompactString::new("save"), false, 13, 24);
    croquis.macros.add_model(ModelDefinition {
        name: CompactString::new("modelValue"),
        local_name: CompactString::new("model"),
        model_type: None,
        required: false,
        default_value: None,
    });
    croquis.macros.add_expose(ExposeDefinition {
        name: CompactString::new("focus"),
        expose_type: None,
    });
    croquis.macros.add_slot(SlotsDefinition {
        name: CompactString::new("default"),
        props_type: None,
    });
    croquis
        .macros
        .add_top_level_await(CompactString::new("load()"), 25, 31);

    croquis.hoists.add_hoist(
        HoistLevel::Constant,
        CompactString::new("_hoisted_1"),
        0,
        10,
    );
    croquis
        .reactivity
        .register(CompactString::new("count"), ReactiveKind::Ref, 10);
    croquis.reactivity.add_loss(ReactivityLoss {
        kind: ReactivityLossKind::ReactiveSpread {
            source_name: CompactString::new("state"),
        },
        start: 30,
        end: 40,
    });
    croquis.race_conditions.record(
        RaceConditionRiskKind::ScheduledMutation {
            scheduler_name: CompactString::new("setTimeout"),
            mutated_targets: vec![CompactString::new("count")],
        },
        50,
        70,
    );
    croquis.provide_inject.add_provide(
        ProvideKey::String(CompactString::new("theme")),
        CompactString::new("theme"),
        None,
        None,
        71,
        80,
    );
    croquis.provide_inject.add_inject(
        ProvideKey::String(CompactString::new("theme")),
        CompactString::new("theme"),
        None,
        None,
        InjectPattern::ObjectDestructure(vec![CompactString::new("color")]),
        None,
        81,
        90,
    );
    croquis.provide_inject.add_composable(
        CompactString::new("useTheme"),
        CompactString::new("./theme"),
        None,
        true,
        true,
        true,
        91,
        100,
    );
    croquis.setup_context.record_violation(
        SetupContextViolationKind::ModuleLevelState,
        CompactString::new("ref"),
        101,
        105,
    );

    let summary = croquis.semantic_summary();

    assert_eq!(summary.script_binding_count, 1);
    assert_eq!(summary.prop_alias_count, 1);
    assert_eq!(summary.symbol_count, 1);
    assert_eq!(summary.symbol_reference_count, 1);
    assert_eq!(summary.unused_symbol_count, 0);
    assert_eq!(summary.v_for_scope_count, 1);
    assert_eq!(summary.v_slot_scope_count, 1);
    assert_eq!(summary.event_handler_scope_count, 1);
    assert_eq!(summary.template_scope_count, 3);
    assert!(summary.scope_binding_count >= 4);
    assert_eq!(summary.macro_call_count, 1);
    assert_eq!(summary.prop_definition_count, 1);
    assert_eq!(summary.emit_definition_count, 1);
    assert_eq!(summary.emit_call_count, 1);
    assert_eq!(summary.model_definition_count, 1);
    assert_eq!(summary.exposed_binding_count, 1);
    assert_eq!(summary.slot_definition_count, 1);
    assert_eq!(summary.top_level_await_count, 1);
    assert_eq!(summary.hoist_count, 1);
    assert_eq!(summary.reactive_source_count, 1);
    assert_eq!(summary.reactivity_loss_count, 1);
    assert_eq!(summary.race_condition_count, 1);
    assert_eq!(summary.provide_count, 1);
    assert_eq!(summary.inject_count, 1);
    assert_eq!(summary.destructured_inject_count, 1);
    assert_eq!(summary.composable_count, 1);
    assert_eq!(summary.setup_context_violation_count, 1);
}
