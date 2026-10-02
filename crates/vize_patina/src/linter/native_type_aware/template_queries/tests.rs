use super::{TypeAwareDocument, generated_offset_for_content};
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
