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
fn const_and_declare_enums_stay_type_only() {
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

    assert!(!result.bindings.contains("ConstStatus"));
    assert!(!result.bindings.contains("AmbientStatus"));
}
