use super::{TypeAwareDocument, generated_offset_for_content, generated_offset_for_text};
use crate::linter::native_type_aware::expression_bindings;
use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping, VizeSubSpan};
use vize_l0::String;

#[test]
fn camelized_shorthand_probes_the_value_instead_of_the_native_check() {
    for value in ["ariaLabel", "__props.ariaLabel", "__vizeUnref(ariaLabel"] {
        let generated = format!("const __vize_native_prop_check_10 = ({value});");
        let check_start = generated.find("__vize_native_prop_check_10").unwrap();
        let value_start = generated.find(" = (").unwrap() + " = (".len();
        let value_end = value_start + value.len();
        let document = TypeAwareDocument {
            content: String::from(generated),
            mapping: ProjectionMapping::from_spans(vec![VizeMapping {
                gen_range: 0..value_end,
                src_range: 10..20,
                sub_spans: vec![
                    VizeSubSpan {
                        gen_range: check_start..check_start + "__vize_native_prop_check_10".len(),
                        src_range: 10..20,
                    },
                    VizeSubSpan {
                        gen_range: value_start..value_end,
                        src_range: 10..20,
                    },
                ],
            }]),
        };
        assert_eq!(
            generated_offset_for_content(&document, 10, "aria-label", "ariaLabel"),
            Some((value_end - 1) as u32),
            "value mapping for {value}"
        );
    }
}

#[test]
fn real_shorthand_projection_queries_camelized_expression_values() {
    let source = include_str!("../../../../tests/fixtures/same-name-native-types.vue");
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let template = descriptor.template.as_ref().unwrap();
    let script = descriptor.script_setup.as_ref().unwrap();
    let allocator = vize_l0::Allocator::default();
    let (root, errors) = vize_armature::Parser::new(&allocator, &template.content).parse();
    assert!(errors.is_empty());
    let document = super::super::document::project_type_aware(
        &descriptor,
        &script.content,
        Some(&root),
        script.loc.start as u32,
        template.loc.start as u32,
        "CamelizedShorthand.vue",
    );
    let (queries, _) = super::collect_template_query_sets(
        &document,
        &root,
        template.loc.start as u32,
        true,
        false,
    );
    assert_eq!(queries.len(), 3);
    for (query, expected) in queries
        .iter()
        .zip(["ariaLabel", "ariaLabelledby", "ariaDescription"])
    {
        let end = query.generated_offset as usize + 1;
        assert_eq!(
            document.content.get(end - expected.len()..end),
            Some(expected)
        );
    }
}

#[test]
fn a_nested_callee_uses_the_authored_initializer_and_the_full_expression_is_inferred() {
    let text = "  const __vize_handler_0: unknown = (() => emit('retry'));\n";
    let expression = "() => emit('retry')";
    let start = text.find(expression).unwrap();
    let name = text.find("__vize_handler_0").unwrap();
    let mut document = TypeAwareDocument {
        content: String::from(text),
        mapping: ProjectionMapping::from_spans(vec![VizeMapping {
            gen_range: 0..text.len(),
            src_range: 100..100 + expression.len(),
            sub_spans: vec![
                VizeSubSpan {
                    gen_range: name..name + "__vize_handler_0".len(),
                    src_range: 95..99,
                },
                VizeSubSpan {
                    gen_range: start..start + expression.len(),
                    src_range: 100..100 + expression.len(),
                },
            ],
        }]),
    };
    expression_bindings::bind_template_expressions(&mut document);
    let callee = generated_offset_for_text(&document, 106, "emit").expect("callee probe");
    assert_eq!(
        document.content.get(callee as usize..callee as usize + 1),
        Some("t")
    );
    let full = generated_offset_for_text(&document, 100, expression).expect("expression probe");
    let binding =
        expression_bindings::binding_offset(&document.content, full).expect("inferred binding");
    assert_eq!(
        document.content.get(binding as usize..binding as usize + 1),
        Some("0")
    );
    assert!(
        document
            .content
            .starts_with("  const __expr_0 = (() => emit('retry'));\n")
    );
}
