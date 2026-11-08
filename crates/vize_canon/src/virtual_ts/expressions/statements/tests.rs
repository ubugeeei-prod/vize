use super::guarded_callback_reads;
use crate::virtual_ts::generate_virtual_ts;
use vize_carton::{CompactString, String, append};
use vize_croquis::{
    Analyzer, AnalyzerOptions, ScopeId, TemplateExpression, TemplateExpressionKind,
};

fn expression(content: &str, guard: Option<&str>) -> TemplateExpression {
    TemplateExpression {
        content: CompactString::new(content),
        kind: TemplateExpressionKind::VOn,
        start: 0,
        end: content.len() as u32,
        scope_id: ScopeId::new(0),
        vif_guard: guard.map(CompactString::new),
    }
}

#[test]
fn projected_callbacks_read_only_free_names_also_used_by_the_guard() {
    let reads = guarded_callback_reads(&expression(
        "(event) => (handler || (() => fallback))(event, actions)",
        Some("actions && handler && actions.size"),
    ));
    assert_eq!(
        reads.iter().map(CompactString::as_str).collect::<Vec<_>>(),
        ["actions", "handler"]
    );
    assert!(
        guarded_callback_reads(&expression("(handler) => handler", Some("handler"))).is_empty()
    );
    assert!(guarded_callback_reads(&expression("handler", Some("handler"))).is_empty());
    assert!(guarded_callback_reads(&expression("() => handler", None)).is_empty());
}

#[test]
fn giant_plain_interpolation_runs_have_bounded_control_flow_graphs() {
    let mut template = String::from("<main>\n");
    for index in 0..600 {
        append!(template, "  <article>{{{{ total + {index} }}}}</article>\n");
    }
    template.push_str("</main>");
    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template.as_str());
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup("const total = 1;");
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some("const total = 1;"), Some(&root), 0);
    let template_scope = output
        .code
        .split_once("// ========== Template Scope (inherits from setup) ==========")
        .expect("template scope")
        .1
        .split_once("// Reference setup bindings (used in template/CSS v-bind)")
        .expect("binding anchors")
        .0;

    assert_eq!(
        template_scope.matches("  void (() => {\n").count(),
        3,
        "600 reads need three independently checked chunks:\n{template_scope}"
    );
    assert_eq!(
        template_scope.matches("// Interpolation").count(),
        600,
        "every authored interpolation stays checked:\n{template_scope}"
    );
}

#[test]
fn guarded_listener_projection_does_not_own_nested_parameter_diagnostics() {
    let script = "import Child from './Child.vue';\nconst ready = true;\nlet ids: string[] = [];";
    let template = r#"<div v-if="ready"><Child @change="
          v => ids = v.map(
            x => x.id
          )
        " /></div>"#;
    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let listener_source_range = summary
        .template_expressions
        .iter()
        .find(|expression| expression.kind == TemplateExpressionKind::VOn)
        .map(|expression| expression.start as usize..expression.end as usize)
        .expect("the template should contain one component listener");
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert_eq!(
        output
            .code
            .matches(
                "// @ts-ignore Inference-only listener value; the typed event scope owns diagnostics."
            )
            .count(),
        1,
        "the render-time copy must not report nested implicit-any diagnostics:\n{}",
        output.code
    );
    assert_eq!(
        output
            .mapping
            .spans()
            .iter()
            .filter(|mapping| mapping.src_range == listener_source_range)
            .count(),
        1,
        "only the typed event scope may map the authored listener:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("// @ts-ignore Inference-only nested callback parameter.")
            .count(),
        1,
        "a later-line nested callback parameter needs its own suppression:\n{}",
        output.code
    );
}
