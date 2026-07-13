use super::*;
use vize_atelier_jsx::JsxSyntaxProduct;
use vize_croquis::CroquisDocumentProduct;
use vize_module::ModuleSyntaxProduct;
use vize_relief::{ReliefProduct, TransformedReliefProduct};

#[test]
fn configured_graph_preserves_the_project_vue_dialect() {
    let compilation = configured_compilation(Shared::new(Linter::new()), VueVersion::V2).unwrap();

    assert_eq!(
        compilation.input::<VueDialectInput>(),
        Some(&VueVersion::V2)
    );
}

#[test]
fn production_graph_requests_parse_and_complete_semantic_products() {
    let mut compilation =
        configured_compilation(Shared::new(Linter::new()), VueVersion::V3).unwrap();
    let source = compilation
        .add_source(
            "Component.vue",
            "<script setup>const value = 1</script><template>{{ value }}</template>",
        )
        .unwrap();

    let plan = compilation
        .plan_for::<PatinaDocumentReportProduct>(source)
        .unwrap();

    assert!(plan.contains::<ReliefProduct>());
    assert!(plan.contains::<CroquisDocumentProduct>());
    assert!(!plan.contains::<TransformedReliefProduct>());
    let outcome = query_snapshot(&compilation.snapshot(), source).unwrap();
    assert_eq!(outcome.semantics.unwrap().sources().len(), 2);
}

#[test]
fn malformed_sfc_is_cached_once_and_still_produces_patina_diagnostics() {
    let mut compilation =
        configured_compilation(Shared::new(Linter::new()), VueVersion::V3).unwrap();
    let source = compilation
        .add_source(
            "Malformed.vue",
            "<template><div /></template><template><span /></template>",
        )
        .unwrap();
    let snapshot = compilation.snapshot();
    let mut session = snapshot.query_session();

    let lint = session
        .query::<PatinaDocumentReportProduct>(source)
        .unwrap();
    assert!(lint.value().error_count > 0);
    assert!(
        lint.value()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "parser/sfc")
    );
    assert_eq!(
        session
            .counters()
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
            .executions(),
        1
    );
    session
        .query::<vize_atelier_sfc::SfcDescriptorProduct>(source)
        .unwrap();
    let counters = session
        .counters()
        .for_product::<vize_atelier_sfc::SfcDescriptorProduct>();
    assert_eq!(counters.executions(), 1);
    assert_eq!(counters.cache_hits(), 1);
}

#[test]
fn jsx_graph_uses_owned_syntax_and_never_plans_relief() {
    let mut compilation =
        configured_compilation(Shared::new(Linter::new()), VueVersion::V3).unwrap();
    let source = compilation
        .add_source("View.tsx", "const View = (): JSX.Element => <img />;")
        .unwrap();
    let plan = compilation
        .plan_for::<PatinaDocumentReportProduct>(source)
        .unwrap();

    assert!(plan.contains::<JsxSyntaxProduct>());
    assert!(plan.contains::<CroquisDocumentProduct>());
    assert!(!plan.contains::<ReliefProduct>());
    let outcome = query_snapshot(&compilation.snapshot(), source).unwrap();
    assert!(outcome.semantics.is_some());
}

#[test]
fn lint_and_cross_file_share_the_same_semantic_revision() {
    let graph = LintArtifactGraph::new(
        Shared::new(Linter::new()),
        VueVersion::V3,
        [
            (
                Path::new("App.vue"),
                "<script setup>import Child from './Child.vue'</script><template><Child /></template>",
            ),
            (Path::new("Child.vue"), "<template><p>child</p></template>"),
        ],
    )
    .unwrap();

    let app_lint = graph.query(0).unwrap();
    let child_lint = graph.query(1).unwrap();
    let cross_file = graph.query_cross_file(0).unwrap();

    assert_shared_vue_closure_executed(&app_lint.trace);
    assert_shared_vue_closure_executed(&child_lint.trace);
    assert_shared_vue_execution_counters(&app_lint.counters, 1);
    assert_shared_vue_execution_counters(&child_lint.counters, 1);
    assert!(cross_file.trace.cache_hit::<CroquisDocumentProduct>());
    assert_shared_vue_closure_not_executed(&cross_file.trace);
    assert_shared_vue_execution_counters(&cross_file.counters, 0);
    assert_eq!(
        cross_file
            .counters
            .for_product::<CroquisDocumentProduct>()
            .cache_hits(),
        2
    );
}

#[test]
fn fixed_source_recomputes_selectively_then_cross_file_reuses_it() {
    let graph = LintArtifactGraph::new(
        Shared::new(Linter::new()),
        VueVersion::V3,
        [
            (Path::new("App.vue"), "<template><main /></template>"),
            (Path::new("Child.vue"), "<template><p>child</p></template>"),
        ],
    )
    .unwrap();
    graph.query(0).unwrap();
    graph.query(1).unwrap();

    graph
        .revise_sources(&[(0, "<template><main id=\"app\" /></template>")])
        .unwrap();
    let fixed = graph.query(0).unwrap();
    let cross_file = graph.query_cross_file(0).unwrap();

    let app = graph.source(0).unwrap();
    let child = graph.source(1).unwrap();
    assert_shared_vue_closure_executed(&fixed.trace);
    assert_shared_vue_execution_counters(&fixed.counters, 1);
    assert!(
        cross_file
            .trace
            .cache_hit_for_source::<CroquisDocumentProduct>(app)
    );
    assert!(
        cross_file
            .trace
            .cache_hit_for_source::<CroquisDocumentProduct>(child)
    );
    assert_shared_vue_closure_not_executed(&cross_file.trace);
    assert_shared_vue_execution_counters(&cross_file.counters, 0);
    assert_eq!(
        cross_file
            .counters
            .for_product::<CroquisDocumentProduct>()
            .cache_hits(),
        2
    );
}

#[test]
fn raw_module_revision_keeps_its_identity_for_cross_file_analysis() {
    let graph = LintArtifactGraph::new(
        Shared::new(Linter::new()),
        VueVersion::V3,
        [
            (Path::new("App.vue"), "<template><main /></template>"),
            (Path::new("state.ts"), "export const count = 0"),
        ],
    )
    .unwrap();
    let source = graph.source(1).unwrap();

    graph
        .revise_sources(&[(1, "export const count = 1")])
        .unwrap();
    let snapshot = graph.current_snapshot().unwrap();

    assert_eq!(
        snapshot.source(source).unwrap().text(),
        "export const count = 1"
    );
    assert!(
        graph
            .query_cross_file(0)
            .unwrap()
            .artifact
            .layout_for_source(source)
            .is_some()
    );
}

#[test]
fn raw_module_and_html_use_their_own_frontend_products() {
    let script = "import { ref } from '@vue/reactivity'; export const count = ref(0);";
    let html = r#"<button v-on:click="save">Save</button>"#;
    let graph = LintArtifactGraph::new(
        Shared::new(
            Linter::with_preset(vize_patina::LintPreset::Opinionated)
                .with_additional_rules(vec!["script/prefer-import-from-vue".into()]),
        ),
        VueVersion::V3,
        [
            (Path::new("state.ts"), script),
            (Path::new("index.html"), html),
        ],
    )
    .unwrap();

    let module = graph.query(0).unwrap();
    let template = graph.query(1).unwrap();

    assert!(module.trace.executed::<ModuleSyntaxProduct>());
    assert!(!module.trace.executed::<ReliefProduct>());
    assert!(!module.trace.executed::<CroquisDocumentProduct>());
    assert!(module.semantics.is_none());
    assert_eq!(
        module
            .counters
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        1
    );
    assert!(template.trace.executed::<ReliefProduct>());
    assert!(!template.trace.executed::<ModuleSyntaxProduct>());
    assert_eq!(
        template
            .counters
            .for_product::<ReliefProduct>()
            .executions(),
        1
    );
    assert_eq!(
        vize_carton::cstr!("{:?}", module.result.diagnostics),
        vize_carton::cstr!(
            "{:?}",
            Linter::with_preset(vize_patina::LintPreset::Opinionated)
                .with_additional_rules(vec!["script/prefer-import-from-vue".into()])
                .lint_script(script, "state.ts")
                .diagnostics
        )
    );
    assert_eq!(
        vize_carton::cstr!("{:?}", template.result.diagnostics),
        vize_carton::cstr!(
            "{:?}",
            Linter::with_preset(vize_patina::LintPreset::Opinionated)
                .with_additional_rules(vec!["script/prefer-import-from-vue".into()])
                .lint_standalone_html(html, "index.html")
                .diagnostics
        )
    );
}

#[test]
fn revising_raw_module_invalidates_only_its_artifact_closure() {
    let graph = LintArtifactGraph::new(
        Shared::new(Linter::new()),
        VueVersion::V3,
        [
            (Path::new("state.ts"), "export const count = 0"),
            (Path::new("index.html"), "<main>stable</main>"),
        ],
    )
    .unwrap();
    let module_source = graph.source(0).unwrap();
    let html_source = graph.source(1).unwrap();
    graph.query(0).unwrap();
    graph.query(1).unwrap();

    graph
        .revise_sources(&[(0, "export const count = 1")])
        .unwrap();
    let changed = graph.query(0).unwrap();
    let unrelated = graph.query(1).unwrap();

    assert_eq!(graph.source(0).unwrap(), module_source);
    assert_eq!(graph.source(1).unwrap(), html_source);
    assert!(changed.trace.executed::<ModuleSyntaxProduct>());
    assert_eq!(
        changed
            .counters
            .for_product::<ModuleSyntaxProduct>()
            .executions(),
        1
    );
    assert!(unrelated.trace.cache_hit::<PatinaDocumentReportProduct>());
    assert!(!unrelated.trace.executed::<ReliefProduct>());
}

fn assert_shared_vue_closure_executed(trace: &vize_atlas::ExecutionTrace) {
    assert!(trace.executed::<vize_atelier_sfc::SfcDescriptorProduct>());
    assert!(trace.executed::<ReliefProduct>());
    assert!(trace.executed::<CroquisDocumentProduct>());
}

fn assert_shared_vue_closure_not_executed(trace: &vize_atlas::ExecutionTrace) {
    assert!(!trace.executed::<vize_atelier_sfc::SfcDescriptorProduct>());
    assert!(!trace.executed::<ReliefProduct>());
    assert!(!trace.executed::<CroquisDocumentProduct>());
}

fn assert_shared_vue_execution_counters(counters: &vize_atlas::ExecutionCounters, executions: u64) {
    assert_eq!(
        counters
            .for_product::<vize_atelier_sfc::SfcDescriptorProduct>()
            .executions(),
        executions
    );
    assert_eq!(
        counters.for_product::<ReliefProduct>().executions(),
        executions
    );
    assert_eq!(
        counters
            .for_product::<CroquisDocumentProduct>()
            .executions(),
        executions
    );
}
