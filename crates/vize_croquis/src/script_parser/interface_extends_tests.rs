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

#[test]
fn multiline_generic_interface_heritage_stays_one_clause() {
    let result = parse_script_setup(
        r#"
            import type { BaseProps } from './base'

            interface FooProps extends Omit<
                BaseProps,
                'modelValue' | 'defaultValue'
            > {
                label: string
            }

            defineProps<FooProps>()
        "#,
    );

    assert_eq!(
        result.types.definitions().interface_extends("FooProps"),
        [
            "Omit<\n                BaseProps,\n                'modelValue' | 'defaultValue'\n            >"
        ]
    );
}

#[test]
fn parse_define_props_type_reference_keeps_union_members_nested() {
    let result = parse_script_setup(
        r#"
interface Props {
  isOpened: boolean
  interaction?:
    | { text: string; to: string; event?: never }
    | { text: string; event: () => void; to?: never }
}

const props = defineProps<Props>()
"#,
    );

    let prop_names = result
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(prop_names, ["isOpened", "interaction"]);
}
