use vize_atelier_jsx::{JsxLang, JsxSyntaxProduct, register_atlas_providers};
use vize_atlas::Compilation;
use vize_croquis::CroquisDocumentProduct;
use vize_flow::{ControlEdgeKind, FlowProduct};
use vize_module::ModuleSyntaxProduct;

#[test]
fn tsx_flow_contains_module_and_render_control_from_one_syntax_product() {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "View.tsx",
            "import Card from './Card'; export function View({ items }) { for (const item of items) { if (!item) continue; } return <Card>{items.map(item => item && <b>{item}</b>)}</Card>; }",
        )
        .unwrap();
    let modules = compilation.query::<ModuleSyntaxProduct>(source).unwrap();
    assert_eq!(
        modules.value().modules[0].imports[0].specifier.as_ref(),
        "./Card"
    );
    let flow = compilation.query::<FlowProduct>(source).unwrap();
    assert!(
        flow.value()
            .control_edges()
            .any(|edge| edge.kind() == ControlEdgeKind::LoopBack)
    );
    assert!(flow.value().sources().count() >= 2);
}

#[test]
fn tsx_croquis_preserves_script_semantics_and_jsx_facts() {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "View.tsx?component",
            "import Card from './Card'; type Count = number; const count: Count = 1; export const View = () => <Card value={count} />;",
        )
        .unwrap();
    let syntax = compilation.query::<JsxSyntaxProduct>(source).unwrap();
    let document = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    let semantic = document.value().semantic_snapshot();
    let scope_ids: std::collections::HashSet<_> =
        semantic.scopes.iter().map(|scope| scope.id).collect();

    assert_eq!(syntax.value().lang, JsxLang::Tsx);
    assert!(syntax.value().diagnostics.is_empty());
    assert_eq!(semantic.summary.import_statement_count, 1);
    assert!(semantic.summary.script_binding_count > 0);
    assert!(
        semantic
            .bindings
            .iter()
            .any(|binding| binding.name == "count")
    );
    assert!(
        semantic
            .component_usages
            .iter()
            .any(|usage| usage.name == "Card" && scope_ids.contains(&usage.scope_id))
    );
    assert!(
        semantic
            .template_expressions
            .iter()
            .all(|expression| scope_ids.contains(&expression.scope_id))
    );
}

#[test]
fn tsx_croquis_assigns_nested_jsx_to_the_callback_scope() {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "List.tsx",
            "import Child from './Child'; const items = [1]; export const List = () => items.map(item => <Child value={item} />);",
        )
        .unwrap();
    let document = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    let semantic = document.value().semantic_snapshot();
    let usage = semantic
        .component_usages
        .iter()
        .find(|usage| usage.name == "Child")
        .unwrap();
    let scope = semantic
        .scopes
        .iter()
        .find(|scope| scope.id == usage.scope_id)
        .unwrap();

    assert_eq!(scope.kind, "callback", "scopes={:?}", semantic.scopes);
    assert!(scope.bindings.iter().any(|binding| binding.name == "item"));
    assert!(
        !semantic
            .bindings
            .iter()
            .any(|binding| binding.name == "item")
    );
}

#[test]
fn tsx_croquis_projects_component_props_events_and_default_slot() {
    let mut compilation = Compilation::new();
    register_atlas_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "Card.tsx",
            "import Child from './Child'; const value = 1; export const Card = () => <Child value={value} onClick={() => value}>label</Child>;",
        )
        .unwrap();
    let document = compilation.query::<CroquisDocumentProduct>(source).unwrap();
    let semantic = document.value().semantic_snapshot();
    let usage = semantic
        .component_usages
        .iter()
        .find(|usage| usage.name == "Child")
        .unwrap();

    assert_eq!(semantic.summary.passed_prop_count, 1);
    assert_eq!(semantic.summary.event_listener_count, 1);
    assert_eq!(semantic.summary.slot_usage_count, 1);
    assert_eq!(usage.props[0].name, "value");
    assert_eq!(usage.events[0].name, "click");
    assert_eq!(usage.slots[0].name, "default");
}
