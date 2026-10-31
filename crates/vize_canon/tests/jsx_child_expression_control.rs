//! Neutral expression projection consumes the authored expression, independently of rendering.
use vize_canon::{
    projection::{project_template_expression_document, project_template_expressions},
    virtual_ts::{ProjectionMapping, VizeMapping},
};
use vize_l0::{Allocator, Box, Span, Vec};
use vize_l2::{
    expr::ExprRef,
    op::{InterpolationOp, Op, Region},
};
#[test]
fn template_and_v_text_keep_all_original_unicode_expression_ranges() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/typechecker/jsx-child-expression-control-6890/template.html.txt"
    );
    let mapping = project_template_expressions(source);
    let mut from = 0;
    let mut generated = 0;
    for expression in ["state.value", "missing", "state.value"] {
        let start = from + source.get(from..).unwrap().find(expression).unwrap();
        let end = start + expression.len();
        assert_eq!(
            mapping.diagnostic_range_to_authored(generated, generated + expression.len()),
            Some((start, end))
        );
        generated += expression.len() + 1;
        from = end;
    }
    assert_eq!(mapping.spans().len(), 3);
}
#[test]
fn typed_child_value_projects_the_same_full_expression_and_span_as_template_text() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/typechecker/jsx-child-expression-control-6890/Original.jsx.txt"
    );
    let owned = Allocator::new();
    let allocator = &owned;
    let start = source.find("state.value").unwrap();
    let span = Span::new(start as u32, (start + 11) as u32);
    for child in [false, true] {
        let payload = Box::new_in(
            InterpolationOp {
                expression: ExprRef::parse_js_in(allocator, "state.value", span),
                span,
            },
            &allocator,
        );
        let root = Region {
            ops: Vec::from_iter_in(
                [if child {
                    Op::JsxChildValue(payload)
                } else {
                    Op::Interpolation(payload)
                }],
                &allocator,
            ),
        };
        let document = project_template_expression_document(&root);
        assert_eq!(document.as_str(), "state.value");
        let mapping = ProjectionMapping::from_emit_document(&document);
        assert_eq!(mapping.spans().len(), 1);
        assert_eq!(
            mapping.diagnostic_range_to_authored(0, 11),
            Some((start, start + 11))
        );
    }
}

/// A real planned batch SFC input is consumed through the existing projection
/// service. This is deliberately not a batch diagnostic/adapter qualification.
#[test]
fn planned_batch_sfc_input_preserves_all_projection_ranges_without_batch_execution() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/typechecker/jsx-child-expression-control-6890/Layout.vue.txt"
    );
    let descriptor =
        vize_atelier_sfc::parse_sfc(source, vize_atelier_sfc::SfcParseOptions::default())
            .expect("genuine authored SFC");
    let template = descriptor.template.as_ref().expect("template block");
    let body: &str = template.content.as_ref();
    let base = template.loc.start;
    assert_eq!(source.get(base..base + body.len()), Some(body));
    let projection = project_template_expressions(body);
    // The existing L1 text lowering coalesces adjacent interpolation tokens
    // into one opaque canonical template run. Preserve that complete span;
    // this consumer must not invent independent inner-expression links.
    let merged = "{{ state.value }}{{ missing }}";
    let start = body
        .find(merged)
        .expect("unchanged adjacent interpolation run");
    let text = "state.value";
    let text_start = body.rfind(text).expect("separate v-text binding");
    let text_generated = merged.len() + 1;
    assert_eq!(
        projection.spans(),
        [
            VizeMapping::new(0..merged.len(), start..start + merged.len()),
            VizeMapping::new(
                text_generated..text_generated + text.len(),
                text_start..text_start + text.len(),
            ),
        ]
    );
    for (generated, authored) in [
        (0..merged.len(), start..start + merged.len()),
        (
            text_generated..text_generated + text.len(),
            text_start..text_start + text.len(),
        ),
    ] {
        let relative = projection
            .diagnostic_range_to_authored(generated.start, generated.end)
            .expect("whole producer-owned span");
        assert_eq!(
            (relative.0 + base, relative.1 + base),
            (authored.start + base, authored.end + base)
        );
    }
}
