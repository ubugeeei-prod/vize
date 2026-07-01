use super::parse_script_setup;

#[test]
fn parse_non_exported_interface_extends_props() {
    let result = parse_script_setup(
        r#"
            interface BaseProps {
                required?: boolean
            }

            interface FooProps extends BaseProps {
                label: string
            }

            defineProps<FooProps>()
        "#,
    );

    let props = result.types.extract_properties("FooProps");
    let names = props
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, ["required", "label"]);
}
