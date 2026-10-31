//! Neutral expression projection consumes the authored expression, independently of rendering.
use vize_canon::{
    projection::{project_template_expression_document, project_template_expressions},
    virtual_ts::ProjectionMapping,
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
