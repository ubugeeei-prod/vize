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

#[test]
fn infers_object_factory_assertion_return_type() {
    let result = parse_script_setup(
        r#"
            interface Settings {
                country: boolean
                platform: boolean
            }

            const props = defineProps({
                settings: {
                    type: Object as () => Settings,
                    default: () => ({ country: true, platform: true }),
                },
            })
        "#,
    );

    let settings = result
        .macros
        .props()
        .iter()
        .find(|prop| prop.name == "settings")
        .expect("settings runtime prop");
    assert_eq!(settings.prop_type.as_deref(), Some("Settings"));
    assert_eq!(
        settings.default_value.as_deref(),
        Some("() => ({ country: true, platform: true })")
    );
}

#[test]
fn quoted_computed_and_cooked_runtime_option_keys_preserve_metadata() {
    let plain = parse_script_setup(
        "defineProps({ label: { type: String, required: true, default: 'Authored' }, count: { type: Number, required: true, default: 0 }, flag: { type: Boolean, required: false, default: false } })",
    );
    let quoted = parse_script_setup(
        r#"defineProps({
      label: { "type": String, 'required': true, "default": 'Authored' },
      count: { ["type"]: Number, ['required']: true, ["default"]: 0 },
      flag: { "ty\u0070e": Boolean, "requ\u0069red": false, "def\u0061ult": false }
    })"#,
    );
    let packet = |result: &super::ScriptParseResult| {
        result
            .macros
            .props()
            .iter()
            .map(|prop| {
                (
                    prop.name.clone(),
                    prop.prop_type.clone(),
                    prop.required,
                    prop.default_value.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(packet(&quoted), packet(&plain));
    assert_eq!(quoted.macros.props().len(), 3);
    assert_eq!(
        quoted.types.literal_member_parse_calls(),
        0,
        "existing runtime AST keys do not demand a second parser"
    );
}

#[test]
fn runtime_options_refuse_dynamic_computed_names_without_executing_them() {
    let result = parse_script_setup(
        r#"defineProps({
      unknown: { [typeKey]: String, [requiredKey]: true, [defaultKey]: 'wrong' },
      expression: { ['ty' + 'pe']: Number, [required()]: true, [getDefault()]: 3 },
      mixed: { [unknownKey]: 'opaque', ['type']: Boolean, ['default']: false }
    })"#,
    );
    for name in ["unknown", "expression"] {
        let prop = result
            .macros
            .props()
            .iter()
            .find(|prop| prop.name == name)
            .expect(name);
        assert!(prop.prop_type.is_none());
        assert!(!prop.required);
        assert!(prop.default_value.is_none());
    }
    let mixed = result
        .macros
        .props()
        .iter()
        .find(|prop| prop.name == "mixed")
        .expect("mixed");
    assert_eq!(mixed.prop_type.as_deref(), Some("boolean"));
    assert_eq!(mixed.default_value.as_deref(), Some("false"));
}
