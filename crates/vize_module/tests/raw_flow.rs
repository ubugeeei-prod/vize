use vize_atlas::{Compilation, ProductId};
use vize_flow::{ControlEdgeKind, FlowProduct, TerminatorKind};
use vize_module::{ModuleSyntaxProduct, register_raw_providers};

#[test]
fn raw_typescript_owns_facts_and_real_cfg() {
    let mut compilation = Compilation::new();
    register_raw_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "branch.ts",
            "import { x } from './x'; export function f(n: number) { while (n) { if (x) return n; n--; } throw Error(); }",
        )
        .unwrap();
    let syntax = compilation.query::<ModuleSyntaxProduct>(source).unwrap();
    assert_eq!(
        syntax.value().modules[0].imports[0].specifier.as_ref(),
        "./x"
    );
    assert!(!syntax.value().modules[0].declarations.is_empty());
    assert!(!syntax.value().modules[0].references.is_empty());
    let flow = compilation.query::<FlowProduct>(source).unwrap();
    let graph = flow.value();
    assert!(
        graph
            .control_edges()
            .any(|edge| edge.kind() == ControlEdgeKind::LoopBack)
    );
    assert!(
        graph
            .control_edges()
            .any(|edge| edge.kind() == ControlEdgeKind::TrueBranch)
    );
    assert!(
        graph
            .control_edges()
            .any(|edge| edge.kind() == ControlEdgeKind::Return)
    );
    assert!(
        graph
            .control_edges()
            .any(|edge| edge.kind() == ControlEdgeKind::Exception)
    );
    assert!(graph.nodes().any(|node| {
        matches!(
            node.kind(),
            vize_flow::NodeKind::Terminator(TerminatorKind::Return)
        )
    }));
    assert!(graph.nodes().any(|node| {
        matches!(
            node.kind(),
            vize_flow::NodeKind::Terminator(TerminatorKind::Throw)
        )
    }));
    let return_provenance = graph
        .nodes()
        .find(|node| {
            matches!(
                node.kind(),
                vize_flow::NodeKind::Terminator(TerminatorKind::Return)
            )
        })
        .unwrap()
        .provenance();
    assert_eq!(
        graph
            .control_edges()
            .find(|edge| edge.kind() == ControlEdgeKind::Return)
            .unwrap()
            .provenance(),
        return_provenance
    );
}

#[test]
fn raw_module_plan_has_no_template_semantic_products() {
    let mut compilation = Compilation::new();
    register_raw_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source("plain.js", "if (ok) run();")
        .unwrap();
    let plan = compilation.plan_for::<FlowProduct>(source).unwrap();
    let products = plan.products();
    assert_eq!(
        products,
        &[
            ProductId::of::<ModuleSyntaxProduct>(),
            ProductId::of::<FlowProduct>()
        ]
    );
}

#[test]
fn nested_functions_and_dead_blocks_do_not_enter_root_reachability() {
    let mut compilation = Compilation::new();
    register_raw_providers(&mut compilation).unwrap();
    let source = compilation
        .add_source(
            "boundaries.ts",
            "function nested() { return 1; } try { throw Error(); } finally { cleanup(); } unreachable();",
        )
        .unwrap();
    let flow = compilation.query::<FlowProduct>(source).unwrap();
    let graph = flow.value();
    let reachability = graph.reachability();
    let function_target = graph
        .control_edges()
        .find(|edge| edge.kind() == ControlEdgeKind::FunctionEntry)
        .map(|edge| edge.to())
        .expect("OXC exposes the nested function subgraph");
    let dead_target = graph
        .control_edges()
        .find(|edge| edge.kind() == ControlEdgeKind::Unreachable)
        .map(|edge| edge.to())
        .expect("OXC exposes the dead-code relationship");
    assert!(!reachability.contains(function_target));
    assert!(!reachability.contains(dead_target));
    assert!(!graph.reverse_postorder().contains(&function_target));
    assert!(!graph.dominators().is_reachable(dead_target));
}
