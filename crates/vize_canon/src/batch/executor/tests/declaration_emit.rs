use super::super::strip_internal_vue_declaration_fields;

#[test]
fn strips_internal_fallthrough_fields_from_vue_declarations() {
    let source = "\
declare const __vize_component__: {
    readonly __vizeComponentMarker: true;
    readonly __vizeRawProps?: Props;
    readonly __vizeHasFallthroughProps: true;
    readonly __vizeFallthroughProps?: Partial<__VizeNativeElement<\"button\">>;
} & __VizeComponentConstructor;
";

    let stripped = strip_internal_vue_declaration_fields(source).unwrap();

    assert_eq!(
        stripped,
        "\
declare const __vize_component__: {
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
"
    );
}

#[test]
fn strips_multiline_internal_fallthrough_fields_from_vue_declarations() {
    let source = "\
declare const __vize_component__: {
    readonly __vizeComponentMarker: true;
    readonly __vizeHasFallthroughProps: true;
    readonly __vizeFallthroughProps?:
        Partial<__VizeNativeElement<\"button\">> &
        Partial<__VizeNativeElement<\"a\">>;
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
";

    let stripped = strip_internal_vue_declaration_fields(source).unwrap();

    assert_eq!(
        stripped,
        "\
declare const __vize_component__: {
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
"
    );
}

#[test]
fn strips_internal_marker_from_raw_props_vue_declarations() {
    let source = "\
declare const __vize_component__: {
    readonly __vizeComponentMarker: true;
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
";

    let stripped = strip_internal_vue_declaration_fields(source).unwrap();

    assert_eq!(
        stripped,
        "\
declare const __vize_component__: {
    readonly __vizeRawProps?: Props;
} & __VizeComponentConstructor;
"
    );
}
