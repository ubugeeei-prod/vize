#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::parse_script_setup;
use vize_relief::BindingType;

#[test]
fn runtime_enum_is_a_template_binding() {
    let result = parse_script_setup(
        r#"
enum Status {
    Pending,
    Ready,
}
"#,
    );

    assert_eq!(result.bindings.get("Status"), Some(BindingType::SetupConst));
}

#[test]
fn runtime_enum_binding_span_is_captured() {
    let source = "enum MyEnum { Ready }\n";
    let result = parse_script_setup(source);

    let (start, end) = result.binding_spans["MyEnum"];
    assert_eq!(&source[start as usize..end as usize], "MyEnum");
}

#[test]
fn const_enums_are_runtime_bindings_and_declare_enums_stay_type_only() {
    let result = parse_script_setup(
        r#"
const enum ConstStatus {
    Ready,
}
declare enum AmbientStatus {
    Ready,
}
"#,
    );

    assert_eq!(
        result.bindings.get("ConstStatus"),
        Some(BindingType::SetupConst)
    );
    assert!(!result.bindings.contains("AmbientStatus"));
}

#[test]
fn computed_type_keys_stay_with_setup_enums() {
    let result = parse_script_setup(
        r#"
enum Keys {
    A = "A",
    B = "B",
}
type Foo = {
    [Keys.A]: number | null;
    [Keys.B]: number;
};
type Handler = {
    [Keys.A](): void;
};
"#,
    );

    let foo = result
        .type_exports
        .iter()
        .find(|export| export.name.as_str() == "Foo")
        .expect("Foo type export");
    assert!(
        !foo.hoisted,
        "a computed type key cannot outlive its setup-scope enum"
    );
    let handler = result
        .type_exports
        .iter()
        .find(|export| export.name.as_str() == "Handler")
        .expect("Handler type export");
    assert!(
        !handler.hoisted,
        "a computed method key cannot outlive its setup-scope enum"
    );
}

#[test]
fn imported_computed_type_keys_remain_hoistable() {
    let result = parse_script_setup(
        r#"
import { Keys } from "./keys";
type Foo = {
    [Keys.A]: number;
};
"#,
    );

    let foo = result
        .type_exports
        .iter()
        .find(|export| export.name.as_str() == "Foo")
        .expect("Foo type export");
    assert!(
        foo.hoisted,
        "module-scope imports must not move otherwise hoistable types into setup"
    );
}

#[test]
fn types_naming_setup_classes_stay_with_them() {
    // typescript-eslint v8.72.2 no-misused-promises invalid[66] in
    // `<script setup>` (#12988): the heritage class lives in setup scope.
    let result = parse_script_setup(
        r#"
import { Imported } from "./imported";
class MyClass {
    setThing(): void {}
}
interface ExtendsMyClass extends MyClass {
    setThing(): Promise<void>;
}
type Instance = MyClass;
type Indirect = { value: Instance };
interface ExtendsImported extends Imported {}
type Plain = { value: string };
"#,
    );

    let hoisted = |name: &str| {
        result
            .type_exports
            .iter()
            .find(|export| export.name.as_str() == name)
            .unwrap_or_else(|| panic!("{name} type export"))
            .hoisted
    };
    assert!(!hoisted("ExtendsMyClass"));
    assert!(!hoisted("Instance"));
    assert!(!hoisted("Indirect"));
    assert!(hoisted("ExtendsImported"));
    assert!(hoisted("Plain"));
}
