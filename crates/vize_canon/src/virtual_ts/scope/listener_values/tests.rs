use super::group_expressions;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis, TemplateExpressionKind};

fn analyze(handler: &str) -> Croquis {
    let template = format!(
        "<div v-if=\"actions && handler\"><Child v-if=\"actions && handler && actions.size\" :actions=\"actions\" @pick=\"{handler}\" /></div><slot v-if=\"loading\" /><slot v-else />"
    );
    let allocator = vize_carton::Allocator::default();
    let (root, _) = vize_armature::parse(&allocator, &template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(
        "const actions = new Map(); const handler = (value: string) => value; const loading = false;",
    );
    analyzer.analyze_template(&root);
    analyzer.finish()
}

#[test]
fn actual_reference_read_retains_its_expression_identity_in_render_order() {
    let summary = analyze("handler");
    let expression = summary
        .template_expressions
        .iter()
        .find(|expression| expression.kind == TemplateExpressionKind::VOn)
        .unwrap();
    let parent = summary
        .scopes
        .get_scope(expression.scope_id)
        .unwrap()
        .parent()
        .unwrap();
    let grouped = group_expressions(&summary, true);
    let render = &grouped[&parent.as_u32()];
    let index = render
        .iter()
        .position(|row| std::ptr::eq(*row, expression))
        .unwrap();
    assert!(
        render[..index]
            .iter()
            .any(|row| row.kind == TemplateExpressionKind::VIf)
    );
    assert!(
        render[index + 1..]
            .iter()
            .any(|row| row.content == "loading")
    );
    assert!(render.windows(2).all(|pair| pair[0].start <= pair[1].start));
    assert!(
        grouped[&expression.scope_id.as_u32()]
            .iter()
            .any(|row| std::ptr::eq(*row, expression))
    );
    assert!(
        !group_expressions(&summary, false)[&parent.as_u32()]
            .iter()
            .any(|row| row.kind == TemplateExpressionKind::VOn)
    );
}

#[test]
fn callback_bodies_and_shadowed_parameters_do_not_become_render_reads() {
    for handler in ["() => {}", "(handler) => handler", "handler('pick')"] {
        let summary = analyze(handler);
        let grouped = group_expressions(&summary, true);
        let expression = summary
            .template_expressions
            .iter()
            .find(|row| row.kind == TemplateExpressionKind::VOn)
            .unwrap();
        let parent = summary
            .scopes
            .get_scope(expression.scope_id)
            .unwrap()
            .parent()
            .unwrap();
        assert!(
            !grouped[&parent.as_u32()]
                .iter()
                .any(|row| std::ptr::eq(*row, expression)),
            "{handler}"
        );
    }
}
