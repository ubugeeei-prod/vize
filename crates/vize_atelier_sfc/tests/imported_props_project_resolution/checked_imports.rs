//! Qualified imported macro references must fail rather than lose props.

use super::{compile_source, project};

#[test]
fn unresolved_namespace_import_in_a_macro_is_not_silently_erased() {
    let project = project();
    let error = compile_source(project.path(), "<script setup lang=\"ts\">import type * as Types from '@missing/Props';defineProps<Types.Foo>();</script>").expect_err("unresolved qualified type");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
    assert_eq!(
        error.message,
        "Cannot resolve imported macro type `Types` from `@missing/Props`."
    );
}

#[test]
fn property_value_types_do_not_need_to_resolve_the_props_shape() {
    use super::{RuntimeProp, RuntimeProps};
    use oxc_ast_visit::Visit;
    let project = project();
    for (declaration, shape, required) in [
        ("", "{ value?: Leaf }", false),
        ("type Props = { value?: Leaf };", "Props", false),
        ("", "Record<'value', Leaf>", true),
    ] {
        let source = vize_carton::cstr!(
            "<script setup lang=\"ts\">import type {{ Leaf }} from '@missing/Leaf'; {declaration} defineProps<{shape}>();</script>"
        );
        let result =
            compile_source(project.path(), &source).expect("value type is not the property shape");
        assert_eq!(result.errors.len(), 0);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &result.code, oxc_span::SourceType::mjs()).parse();
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut props = RuntimeProps::default();
        props.visit_program(&parsed.program);
        assert_eq!(
            props.0,
            vec![RuntimeProp {
                name: "value".into(),
                constructor: None,
                required: Some(required)
            }]
        );
    }
}
