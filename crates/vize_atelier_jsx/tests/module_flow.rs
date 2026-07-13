use vize_atelier_jsx::register_atlas_providers;
use vize_atlas::Compilation;
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
