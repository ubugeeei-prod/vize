use super::parse_script_setup;

#[test]
fn parenthesizes_function_union_members() {
    let result = parse_script_setup(
        r#"
            const props = defineProps({
                data: { type: [Array, Function], required: true },
                formatter: Function,
                labelOrPredicate: [
                    String,
                    Function as PropType<(value: string) => boolean>,
                ],
            })
        "#,
    );

    let props = result.macros.props();
    assert_eq!(props.len(), 3);
    assert_eq!(
        props
            .iter()
            .find(|prop| prop.name == "data")
            .and_then(|prop| prop.prop_type.as_deref()),
        Some("unknown[] | ((...args: any[]) => any)")
    );
    assert_eq!(
        props
            .iter()
            .find(|prop| prop.name == "formatter")
            .and_then(|prop| prop.prop_type.as_deref()),
        Some("(...args: any[]) => any"),
        "a standalone function type does not need an extra pair of parentheses"
    );
    assert_eq!(
        props
            .iter()
            .find(|prop| prop.name == "labelOrPredicate")
            .and_then(|prop| prop.prop_type.as_deref()),
        Some("string | ((value: string) => boolean)"),
        "PropType function members need the same union precedence protection"
    );
}
