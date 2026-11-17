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

fn prop_types(result: &super::ScriptParseResult) -> Vec<(&str, Option<&str>, bool)> {
    result
        .macros
        .props()
        .iter()
        .map(|prop| (prop.name.as_str(), prop.prop_type.as_deref(), prop.required))
        .collect()
}

/// #13112: Vue infers any other constructor value as what it builds.
#[test]
fn constructor_values_type_props_from_the_value() {
    let result = parse_script_setup(
        r#"
            import { Counter } from "./Counter";
            import * as shapes from "./shapes";
            defineProps({
                counter: { type: Counter },
                required: { type: Counter, required: true },
                shorthand: Counter,
                member: { type: shapes.Counter },
                either: { type: [Counter, String] },
                deep: [shapes.nested.Counter, Number],
                optionalChain: { type: shapes?.Counter },
                computed: { type: shapes["Counter"] },
                call: { type: makeCounter() },
            })
        "#,
    );
    assert_eq!(
        prop_types(&result),
        [
            ("counter", Some("__RuntimePropCtor<typeof Counter>"), false),
            ("required", Some("__RuntimePropCtor<typeof Counter>"), true),
            (
                "shorthand",
                Some("__RuntimePropCtor<typeof Counter>"),
                false
            ),
            (
                "member",
                Some("__RuntimePropCtor<typeof shapes.Counter>"),
                false
            ),
            (
                "either",
                Some("__RuntimePropCtor<typeof Counter> | string"),
                false
            ),
            (
                "deep",
                Some("__RuntimePropCtor<typeof shapes.nested.Counter> | number"),
                false
            ),
            ("optionalChain", None, false),
            ("computed", None, false),
            ("call", None, false),
        ]
    );
}

#[test]
fn builtin_constructors_keep_their_spelling() {
    let result = parse_script_setup(
        "defineProps({ s: String, n: Number, b: Boolean, a: Array, o: Object, d: Date, f: Function, both: [String, Date] })",
    );
    assert_eq!(
        prop_types(&result),
        [
            ("s", Some("string"), false),
            ("n", Some("number"), false),
            ("b", Some("boolean"), false),
            ("a", Some("unknown[]"), false),
            ("o", Some("Record<string, any>"), false),
            ("d", Some("Date"), false),
            ("f", Some("(...args: any[]) => any"), false),
            ("both", Some("string | Date"), false),
        ]
    );
}

/// The props type is module scoped; Vue rejects setup locals in `defineProps`.
#[test]
fn setup_local_constructor_values_stay_untyped() {
    let result = parse_script_setup(
        r#"
            import { Counter } from "./Counter";
            class Local {}
            defineProps({
                local: { type: Local },
                later: { type: Later },
                mixed: { type: [Counter, Local] },
                imported: { type: Counter },
                global: { type: Map },
            })
            function Later() { return 1 }
        "#,
    );
    assert_eq!(
        prop_types(&result),
        [
            ("local", None, false),
            ("later", None, false),
            ("mixed", None, false),
            ("imported", Some("__RuntimePropCtor<typeof Counter>"), false),
            ("global", Some("__RuntimePropCtor<typeof Map>"), false),
        ]
    );
}
