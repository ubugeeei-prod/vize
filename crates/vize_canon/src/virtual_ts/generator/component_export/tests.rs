use super::{ComponentNames, default_export, generic::slot_resolver_field, public_name};
use vize_carton::String;

const SETUP: &str = "// generated setup helpers\n";

fn value_name(name: &str, setup_binding: bool, authored_script: &str) -> String {
    public_name(
        SETUP,
        String::from(name),
        "VueComponent",
        setup_binding,
        Some(authored_script),
    )
}

#[test]
fn filename_export_does_not_shadow_authored_setup_globals() {
    for (name, authored_script) in [
        ("Error", "throw new Error(message);"),
        ("Map", "const cache = new Map<string, number>();"),
    ] {
        assert_eq!(
            value_name(name, false, authored_script),
            [name, "VueComponent"].concat().as_str()
        );
    }
}

#[test]
fn filename_export_keeps_an_authored_setup_binding() {
    assert_eq!(value_name("Error", true, "const Error = 1;"), "Error");
}

#[test]
fn filename_export_fallback_avoids_authored_identifiers() {
    assert_eq!(
        value_name("Error", false, "void Error; void ErrorVueComponent;"),
        "ErrorVueComponent_"
    );
}

#[test]
fn named_export_returns_its_declared_name() {
    let names = ComponentNames {
        value: String::from("Child"),
        type_name: String::from("ChildComponent"),
    };
    let mut output = String::from(SETUP);
    let name = default_export(&mut output, Some(&names)).expect("named export");
    assert_eq!(output.get(name), Some("Child"));
    assert_eq!(
        output,
        "// generated setup helpers\ndeclare const Child: typeof __vize_component__;\nexport default Child;\n"
    );
    let mut batch = String::from(SETUP);
    assert_eq!(default_export(&mut batch, None), None);
    assert_eq!(
        batch,
        "// generated setup helpers\nexport default __vize_component__;\n"
    );
}

#[test]
fn slot_resolver_retains_authored_defaults_instead_of_guessing_synthetic_ones() {
    for raw in [
        "T extends { id: string; } = any",
        "T = any",
        "T extends (value: string) => void = any",
        "A extends Record<string, any> = any, B = A",
        "A = any, B extends A = any",
        "const T extends Tab",
        "T = string",
        "const T extends 'a,b' = 'a,b'",
    ] {
        let plan = crate::virtual_ts::generic_parameters::GenericParameters::parse(raw);
        assert!(plan.diagnostics.is_empty(), "{raw}");
        let actual = slot_resolver_field(plan.authored(), &plan.names, true, "Props");
        assert_eq!(
            actual,
            vize_carton::cstr!(
                "__vizeResolveSlots?: <{raw}>(props: Partial<Props<{}>> & Record<string, unknown>) => $Slots<{}>; ",
                plan.names,
                plan.names
            )
        );
        assert_eq!(
            slot_resolver_field(plan.authored(), &plan.names, false, "Props"),
            ""
        );
    }
}
