use super::{emit_component_default_export, generic::slot_resolver_field};
use vize_carton::String;

#[test]
fn filename_export_does_not_shadow_authored_setup_globals() {
    for (name, authored_script) in [
        ("Error", "throw new Error(message);"),
        ("Map", "const cache = new Map<string, number>();"),
    ] {
        let mut output = String::from("// generated setup helpers\n");
        emit_component_default_export(&mut output, Some(name), false, Some(authored_script));
        assert_eq!(
            output,
            format!(
                "// generated setup helpers\ndeclare const {name}VueComponent: typeof __vize_component__;\nexport default {name}VueComponent;\n"
            )
        );
    }
}

#[test]
fn filename_export_keeps_an_authored_setup_binding() {
    let mut output = String::from("// generated setup helpers\n");
    emit_component_default_export(&mut output, Some("Error"), true, Some("const Error = 1;"));
    assert_eq!(
        output,
        "// generated setup helpers\ndeclare const Error: typeof __vize_component__;\nexport default Error;\n"
    );
}

#[test]
fn filename_export_fallback_avoids_authored_identifiers() {
    let mut output = String::from("// generated setup helpers\n");
    emit_component_default_export(
        &mut output,
        Some("Error"),
        false,
        Some("void Error; void ErrorVueComponent;"),
    );
    assert_eq!(
        output,
        "// generated setup helpers\ndeclare const ErrorVueComponent_: typeof __vize_component__;\nexport default ErrorVueComponent_;\n"
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
                "__vizeResolveSlots?: <{raw}>(props: Partial<Props<{}>> & Record<string, unknown>) => __VizeSlots<{}>; ",
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
