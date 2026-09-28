use super::parse;
use vize_l0::Allocator;
use vize_relief::errors::ErrorCode;

#[test]
fn duplicate_if_directive_is_a_parse_error() {
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, r#"<p v-if="ok" v-if="!ok">x</p>"#);
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == ErrorCode::DuplicateAttribute)
            .count(),
        1
    );
}

#[test]
fn distinct_directive_arguments_are_not_duplicates() {
    let allocator = Allocator::new();
    let (_, errors) = parse(
        &allocator,
        r#"<button :aria-label="label" :aria-disabled="disabled" @click="a" @keydown="b" />"#,
    );
    assert!(
        errors
            .iter()
            .all(|error| error.code != ErrorCode::DuplicateAttribute)
    );
}

#[test]
fn static_and_computed_arguments_are_distinct() {
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, r#"<slot :name="selected" :[name]="value" />"#);
    assert!(
        errors
            .iter()
            .all(|error| error.code != ErrorCode::DuplicateAttribute)
    );
}

#[test]
fn repeated_directive_argument_is_a_duplicate() {
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, r#"<p :title="a" :title="b" />"#);
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == ErrorCode::DuplicateAttribute)
            .count(),
        1
    );
}

#[test]
fn repeated_event_listeners_are_merged_without_parse_errors() {
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, r#"<button @click="first" @click="second" />"#);
    assert!(
        errors
            .iter()
            .all(|error| error.code != ErrorCode::DuplicateAttribute)
    );
}

#[test]
fn repeated_object_bind_spreads_are_not_duplicate_attributes() {
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, r#"<slot v-bind="first" v-bind="second" />"#);
    assert!(
        errors
            .iter()
            .all(|error| error.code != ErrorCode::DuplicateAttribute)
    );
}

#[test]
fn duplicate_attribute_in_wide_tag_is_case_insensitive() {
    let allocator = Allocator::new();
    let (_, errors) = parse(
        &allocator,
        "<div a1 a2 a3 a4 a5 a6 a7 a8 a9 a10 a11 a12 A12 a13 />",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == ErrorCode::DuplicateAttribute)
            .count(),
        1
    );
}
