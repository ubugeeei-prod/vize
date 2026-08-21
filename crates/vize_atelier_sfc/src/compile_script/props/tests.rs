use super::{
    extract_emit_names_from_type, extract_prop_types_from_type_with_context, ts_type_to_js_type,
};
use vize_carton::{FxHashMap, ToCompactString};

#[test]
fn recursive_type_alias_prop_does_not_overflow() {
    // Mirrors nuxt-ui `CheckboxGroup`/`RadioGroup`: a props type whose member
    // resolves through a self-referential alias (`DotPathKeys`) and a
    // conditional/mutually-recursive alias (`NestedItem`). Before the cycle
    // guard in `ts_type_to_js_type_from_ast`, resolving the runtime type of
    // such a prop re-parsed the same alias body forever and overflowed the
    // native stack, aborting `vize build`.
    let mut type_aliases: FxHashMap<vize_carton::String, vize_carton::String> =
        FxHashMap::default();
    type_aliases.insert(
        "NestedItem".to_compact_string(),
        "T extends Array<infer I> ? NestedItem<I> : T".to_compact_string(),
    );
    type_aliases.insert(
        "DotPathKeys".to_compact_string(),
        "{ [K in keyof T & string]: K | `${K}.${DotPathKeys<T[K]>}` }[keyof T & string]"
            .to_compact_string(),
    );

    let type_args = "{ items?: NestedItem<string>; keys?: DotPathKeys<Record<string, any>> }";

    let props = extract_prop_types_from_type_with_context(type_args, None, Some(&type_aliases));

    let items = props
        .iter()
        .find(|(name, _)| name == "items")
        .expect("items prop should be extracted");
    // Recursive structural types have no runtime constructor.
    assert_eq!(items.1.js_type.as_str(), "null");
    assert!(items.1.optional);

    let keys = props
        .iter()
        .find(|(name, _)| name == "keys")
        .expect("keys prop should be extracted");
    assert_eq!(keys.1.js_type.as_str(), "null");
}

#[test]
fn extract_emit_names_keeps_quoted_colon_keys() {
    let names = extract_emit_names_from_type(
        r#"{
              "update:open": [value: boolean]
              'select:item': [id: string]
              close: []
            }"#,
    );

    assert_eq!(names, vec!["update:open", "select:item", "close"]);
}

#[test]
fn extract_emit_names_ignores_payload_literal_union_in_call_signature() {
    let names = extract_emit_names_from_type("{ (e: 'change', mode: 'x' | 'y'): void }");

    assert_eq!(names, vec!["change"]);
}

#[test]
fn extract_emit_names_ignores_payload_literal_union_in_function_type() {
    let names = extract_emit_names_from_type("(e: 'change', mode: 'x' | 'y') => void");

    assert_eq!(names, vec!["change"]);
}

#[test]
fn extract_emit_names_supports_custom_first_parameter_name() {
    let names = extract_emit_names_from_type("{ (evt: 'change', val: 'a' | 'b'): void }");

    assert_eq!(names, vec!["change"]);
}

#[test]
fn extract_emit_names_ignores_payload_literals_across_multiple_call_signatures() {
    let names = extract_emit_names_from_type(
        r#"{
              (e: 'change', mode: 'x' | 'y'): void
              (evt: 'submit', val: 'a' | 'b'): void
            }"#,
    );

    assert_eq!(names, vec!["change", "submit"]);
}

#[test]
fn generic_union_with_array_falls_back_to_unknown_runtime_type() {
    assert_eq!(ts_type_to_js_type("T | T[]"), "null");
}

#[test]
fn leading_union_separator_does_not_add_null_runtime_type() {
    let ty = r#"
          | { type: "link"; href: string }
          | { type: "button"; onClick: () => void }
        "#;

    assert_eq!(ts_type_to_js_type(ty), "Object");
}

#[test]
fn function_or_function_array_prop_keeps_runtime_union() {
    let ty = "((event: MouseEvent) => void | Promise<void>) | Array<((event: MouseEvent) => void | Promise<void>)>";

    assert_eq!(ts_type_to_js_type(ty), "[Function, Array]");
}

#[test]
fn extract_props_keeps_function_or_function_array_runtime_union() {
    let props = extract_prop_types_from_type_with_context(
        "{ onClick?: ((event: MouseEvent) => void | Promise<void>) | Array<((event: MouseEvent) => void | Promise<void>)> }",
        None,
        None,
    );

    let on_click = props
        .iter()
        .find(|(name, _)| name == "onClick")
        .expect("onClick prop should be extracted");
    assert_eq!(on_click.1.js_type.as_str(), "[Function, Array]");
}

#[test]
fn extract_props_keeps_runtime_union_after_omit_intersection() {
    let mut interfaces: FxHashMap<vize_carton::String, vize_carton::String> = FxHashMap::default();
    interfaces.insert(
        "LinkProps".to_compact_string(),
        "{ raw?: boolean; disabled?: boolean }".to_compact_string(),
    );
    interfaces.insert(
        "ButtonProps".to_compact_string(),
        "Omit<LinkProps, 'raw'> & { onClick?: ((event: MouseEvent) => void | Promise<void>) | Array<((event: MouseEvent) => void | Promise<void>)> }".to_compact_string(),
    );
    let props = extract_prop_types_from_type_with_context("ButtonProps", Some(&interfaces), None);

    let on_click = props
        .iter()
        .find(|(name, _)| name == "onClick")
        .expect("onClick prop should be extracted");
    assert_eq!(on_click.1.js_type.as_str(), "[Function, Array]");
}

#[test]
fn indexed_access_to_known_object_member_keeps_runtime_type() {
    let mut interfaces: FxHashMap<vize_carton::String, vize_carton::String> = FxHashMap::default();
    interfaces.insert(
        "TableColumnCtx".to_compact_string(),
        "{ showOverflowTooltip?: boolean | TableOverflowTooltipOptions; tooltipFormatter?: TableOverflowTooltipFormatter<T> }".to_compact_string(),
    );
    interfaces.insert(
        "TableColumnProps".to_compact_string(),
        "{ showOverflowTooltip?: TableColumnCtx<T>['showOverflowTooltip']; tooltipFormatter?: TableColumnCtx<T>['tooltipFormatter'] }".to_compact_string(),
    );
    let mut type_aliases: FxHashMap<vize_carton::String, vize_carton::String> =
        FxHashMap::default();
    type_aliases.insert(
        "TableOverflowTooltipOptions".to_compact_string(),
        "Partial<Omit<UseTooltipProps, 'content'>>".to_compact_string(),
    );
    type_aliases.insert(
        "TableOverflowTooltipFormatter".to_compact_string(),
        "(data: { row: T }) => string".to_compact_string(),
    );

    let props = extract_prop_types_from_type_with_context(
        "TableColumnProps<T>",
        Some(&interfaces),
        Some(&type_aliases),
    );

    let show_overflow = props
        .iter()
        .find(|(name, _)| name == "showOverflowTooltip")
        .expect("showOverflowTooltip prop should be extracted");
    assert_eq!(show_overflow.1.js_type.as_str(), "[Boolean, Object]");

    let formatter = props
        .iter()
        .find(|(name, _)| name == "tooltipFormatter")
        .expect("tooltipFormatter prop should be extracted");
    assert_eq!(formatter.1.js_type.as_str(), "Function");
}

#[test]
fn indexed_access_to_union_and_keyof_keys_keeps_runtime_union() {
    let mut interfaces: FxHashMap<vize_carton::String, vize_carton::String> = FxHashMap::default();
    interfaces.insert(
        "ValueCtx".to_compact_string(),
        "{ enabled?: boolean; label?: string; count?: number }".to_compact_string(),
    );
    interfaces.insert(
        "ValueProps".to_compact_string(),
        "{ explicit?: ValueCtx['enabled' | 'label']; every?: ValueCtx[keyof ValueCtx] }"
            .to_compact_string(),
    );

    let props = extract_prop_types_from_type_with_context("ValueProps", Some(&interfaces), None);

    let explicit = props
        .iter()
        .find(|(name, _)| name == "explicit")
        .expect("explicit prop should be extracted");
    assert_eq!(explicit.1.js_type.as_str(), "[Boolean, String]");

    let every = props
        .iter()
        .find(|(name, _)| name == "every")
        .expect("every prop should be extracted");
    assert_eq!(every.1.js_type.as_str(), "[Boolean, String, Number]");
}
