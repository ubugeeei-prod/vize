use super::{
    ComponentRegistration, ComponentUsage, Croquis, CroquisSemanticSummary, ElementIdInfo,
    ElementIdKind, EventListener, ImportStatementInfo, InvalidExport, InvalidExportKind,
    PassedProp, ReExportInfo, SlotUsage, TemplateExpression, TemplateExpressionKind, TypeExport,
    TypeExportKind, UndefinedRef,
};
use crate::scope::ScopeId;
use vize_carton::{CompactString, smallvec};

#[test]
fn semantic_summary_collects_template_and_export_counts() {
    let mut croquis = Croquis::new();
    croquis.template_info.root_element_count = 2;
    croquis.template_info.uses_attrs = true;
    croquis.template_info.inherit_attrs_disabled = true;
    croquis.used_components.insert(CompactString::new("Child"));
    croquis.used_directives.insert(CompactString::new("focus"));
    croquis
        .unused_bindings
        .push(CompactString::new("unusedBinding"));
    croquis.undefined_refs.push(UndefinedRef {
        name: CompactString::new("missing"),
        offset: 12,
        context: CompactString::new("interpolation"),
    });
    croquis.component_registrations.push(ComponentRegistration {
        name: CompactString::new("Child"),
        local_name: CompactString::new("Child"),
    });
    croquis.component_usages.push(ComponentUsage {
        name: CompactString::new("Child"),
        start: 0,
        end: 20,
        props: smallvec![PassedProp {
            name: CompactString::new("title"),
            name_is_dynamic: false,
            value: Some(CompactString::new("message")),
            start: 1,
            end: 8,
            is_dynamic: true,
        }],
        events: smallvec![EventListener {
            name: CompactString::new("save"),
            name_is_dynamic: false,
            handler: Some(CompactString::new("save")),
            modifiers: smallvec![],
            start: 9,
            end: 14,
        }],
        slots: smallvec![SlotUsage {
            name: CompactString::new("default"),
            name_is_dynamic: false,
            scope_vars: smallvec![CompactString::new("row")],
            start: 15,
            end: 20,
            has_scope: true,
        }],
        has_spread_attrs: true,
        spread_props: vize_carton::SmallVec::new(),
        scope_id: ScopeId::ROOT,
        vif_guard: Some(CompactString::new("visible")),
    });
    croquis.template_expressions.push(TemplateExpression {
        content: CompactString::new("visible"),
        kind: TemplateExpressionKind::VIf,
        start: 0,
        end: 7,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    croquis.template_expressions.push(TemplateExpression {
        content: CompactString::new("model"),
        kind: TemplateExpressionKind::VModel,
        start: 8,
        end: 13,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    croquis.element_ids.push(ElementIdInfo {
        value: CompactString::new("field"),
        start: 21,
        end: 26,
        is_static: true,
        in_loop: false,
        scope_id: ScopeId::ROOT,
        kind: ElementIdKind::Id,
    });
    croquis.element_ids.push(ElementIdInfo {
        value: CompactString::new("field"),
        start: 27,
        end: 32,
        is_static: false,
        in_loop: false,
        scope_id: ScopeId::ROOT,
        kind: ElementIdKind::AriaReference,
    });
    croquis.type_exports.push(TypeExport {
        name: CompactString::new("Props"),
        kind: TypeExportKind::Interface,
        start: 33,
        end: 40,
        hoisted: true,
    });
    croquis.invalid_exports.push(InvalidExport {
        name: CompactString::new("value"),
        kind: InvalidExportKind::Const,
        start: 41,
        end: 50,
    });
    croquis
        .import_statements
        .push(ImportStatementInfo { start: 51, end: 60 });
    croquis.re_exports.push(ReExportInfo { start: 61, end: 70 });
    croquis
        .binding_spans
        .insert(CompactString::new("model"), (71, 76));

    let summary = CroquisSemanticSummary::from_croquis(&croquis);

    assert_eq!(summary.used_component_count, 1);
    assert_eq!(summary.component_registration_count, 1);
    assert_eq!(summary.component_usage_count, 1);
    assert_eq!(summary.passed_prop_count, 1);
    assert_eq!(summary.event_listener_count, 1);
    assert_eq!(summary.slot_usage_count, 1);
    assert_eq!(summary.spread_attr_component_count, 1);
    assert_eq!(summary.used_directive_count, 1);
    assert_eq!(summary.template_expression_count, 2);
    assert_eq!(summary.v_if_expression_count, 1);
    assert_eq!(summary.v_model_expression_count, 1);
    assert_eq!(summary.element_id_count, 2);
    assert_eq!(summary.static_element_id_count, 1);
    assert_eq!(summary.dynamic_element_id_count, 1);
    assert_eq!(summary.id_definition_count, 1);
    assert_eq!(summary.id_reference_count, 1);
    assert_eq!(summary.undefined_ref_count, 1);
    assert_eq!(summary.unused_binding_count, 1);
    assert_eq!(summary.type_export_count, 1);
    assert_eq!(summary.invalid_export_count, 1);
    assert_eq!(summary.import_statement_count, 1);
    assert_eq!(summary.re_export_count, 1);
    assert_eq!(summary.binding_span_count, 1);
    assert!(summary.has_multiple_roots);
    assert!(summary.uses_attrs);
    assert!(!summary.binds_attrs_explicitly);
    assert!(summary.inherit_attrs_disabled);
    assert!(!summary.may_lose_fallthrough_attrs());
}
