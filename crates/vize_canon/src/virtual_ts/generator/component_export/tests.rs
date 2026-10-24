use super::{emit_component_default_export, generic::strip_synthetic_any_defaults};
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
fn synthetic_any_defaults_are_stripped_without_touching_constraints() {
    assert_eq!(
        strip_synthetic_any_defaults("T extends { id: string; } = any").as_str(),
        "T extends { id: string; }"
    );
    assert_eq!(strip_synthetic_any_defaults("T = any").as_str(), "T");
    assert_eq!(
        strip_synthetic_any_defaults("T extends (value: string) => void = any").as_str(),
        "T extends (value: string) => void"
    );
    assert_eq!(
        strip_synthetic_any_defaults("A extends Record<string, any> = any, B = A").as_str(),
        "A extends Record<string, any>, B = A"
    );
    assert_eq!(
        strip_synthetic_any_defaults("A = any, B extends A = any").as_str(),
        "A, B extends A"
    );
    assert_eq!(
        strip_synthetic_any_defaults("const T extends Tab").as_str(),
        "const T extends Tab"
    );
    assert_eq!(
        strip_synthetic_any_defaults("T = string").as_str(),
        "T = string"
    );
}
