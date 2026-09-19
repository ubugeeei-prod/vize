use super::parse_script_setup;
use vize_relief::BindingType;

#[test]
fn implicit_props_do_not_replace_authored_setup_bindings() {
    for props in [
        "defineProps<{ foo: number }>();",
        "defineProps({ foo: Number });",
        "defineProps(['foo']);",
    ] {
        for source in [
            vize_carton::cstr!("const foo = ''; {props}"),
            vize_carton::cstr!("{props} const foo = '';"),
        ] {
            let result = parse_script_setup(&source);
            assert_eq!(
                result.bindings.get("foo"),
                Some(BindingType::LiteralConst),
                "{source}"
            );
            assert!(result.macros.props().iter().any(|prop| prop.name == "foo"));
        }
    }
}

#[test]
fn transparent_typescript_wrappers_preserve_static_const_bindings() {
    let result = parse_script_setup(
        r#"
const option = { text: 'Login info', value: 'LOGIN_INFO' } as const
const items = ([{ value: 1 }] as const)
const constrained = { value: 'plain' } satisfies { value: string }
const callback = (() => 1) as () => number
const actualRef = ref(0)
const unknownCall = useOption()
"#,
    );

    for name in ["option", "items", "constrained", "callback"] {
        assert_eq!(
            result.bindings.get(name),
            Some(BindingType::SetupConst),
            "{name} should retain its static const classification"
        );
    }
    assert_eq!(
        result.bindings.get("actualRef"),
        Some(BindingType::SetupRef)
    );
    assert_eq!(
        result.bindings.get("unknownCall"),
        Some(BindingType::SetupMaybeRef)
    );
}
