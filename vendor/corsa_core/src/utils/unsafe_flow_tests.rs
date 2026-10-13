use super::unsafe_flow::{SimpleType, has_unsafe_any_flow, parse_type_text};

#[test]
fn parses_nested_containers() {
    assert_eq!(
        parse_type_text("Promise<Array<any>>"),
        SimpleType::Generic {
            base: "Promise".into(),
            args: vec![SimpleType::Generic {
                base: "Array".into(),
                args: vec![SimpleType::Any],
            }],
        }
    );
}

#[test]
fn flags_direct_any_assignment() {
    assert!(has_unsafe_any_flow(&["any"], &["string"]));
}

#[test]
fn allows_unknown_targets() {
    assert!(!has_unsafe_any_flow(&["any"], &["unknown"]));
}

#[test]
fn flags_generic_any_assignment() {
    assert!(has_unsafe_any_flow(&["Set<any>"], &["Set<string>"]));
}

#[test]
fn flags_promise_any_returns() {
    assert!(has_unsafe_any_flow(&["Promise<any>"], &["Promise<string>"]));
}

#[test]
fn flags_unions_that_include_any() {
    assert!(has_unsafe_any_flow(&["string | any"], &["string"]));
}

#[test]
fn inferred_targets_still_flag_any_flows() {
    assert!(has_unsafe_any_flow(&["any[]"], &[] as &[&str]));
}

#[test]
fn keeps_specific_flows_allowed() {
    assert!(!has_unsafe_any_flow(
        &["Promise<string>"],
        &["Promise<string>"]
    ));
}
