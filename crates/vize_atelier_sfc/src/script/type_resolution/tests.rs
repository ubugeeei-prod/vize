use super::*;
use vize_carton::cstr;

fn build(source: &str) -> String {
    let name_end = source
        .find('<')
        .or_else(|| source.find(" extends"))
        .unwrap_or(source.find('{').unwrap());
    let body_start = source.find('{').unwrap();
    build_interface_type_source(source, name_end, body_start, source.len())
}

#[test]
fn merges_heritage_clause_of_plain_interface() {
    let merged = build("interface Props extends Base { a: string }");
    assert_eq!(merged.as_str(), "Base & { a: string }");
}

#[test]
fn merges_heritage_clause_of_generic_interface() {
    // Regression: the type-parameter constraint `extends` used to be
    // mistaken for the heritage clause, dropping the inherited members
    // (e.g. nuxt-ui DashboardSidebarProps extends Pick<UseResizableProps, ...>).
    let merged =
        build("interface Props<T extends string = string> extends Pick<P, 'side'> { mode?: T }");
    assert_eq!(merged.as_str(), "Pick<P, 'side'> & { mode?: T }");
}

#[test]
fn generic_interface_without_heritage_returns_body() {
    let merged = build("interface Props<T extends string = string> { mode?: T }");
    assert_eq!(merged.as_str(), "{ mode?: T }");
}

#[test]
fn heritage_scan_survives_arrow_types_in_generic_defaults() {
    let merged = build("interface Props<F = () => void> extends Base { cb?: F }");
    assert_eq!(merged.as_str(), "Base & { cb?: F }");
}

#[test]
fn pick_resolves_members_with_jsdoc_comments() {
    // Regression: JSDoc comments were glued to member keys, so Pick's key
    // filter matched nothing and the inherited props vanished
    // (e.g. nuxt-ui UseResizableProps members all carry JSDoc).
    let mut aliases = FxHashMap::default();
    aliases.insert(
        String::from("P"),
        String::from(
            "{\n  /**\n   * The side, e.g. 'left' | 'right'.\n   * @defaultValue 'left'\n   */\n  side?: 'left' | 'right'\n  // line comment\n  resizable?: boolean\n}",
        ),
    );
    let resolved = resolve_type_to_object_body(
        "Pick<P, 'side' | 'resizable'>",
        &FxHashMap::default(),
        &aliases,
    )
    .unwrap();
    assert_eq!(
        resolved.as_str(),
        "side?: 'left' | 'right'; resizable?: boolean"
    );
}

#[test]
fn pick_filters_newline_separated_members() {
    let mut interfaces = FxHashMap::default();
    let mut aliases = FxHashMap::default();
    aliases.insert(
        String::from("UseResizableProps"),
        String::from("{\n  /**\n   * The id.\n   * @defaultValue useId()\n   */\n  id?: string\n  /**\n   * The side.\n   * @defaultValue 'left'\n   */\n  side?: 'left' | 'right'\n  minSize?: number\n  storage?: 'cookie' | 'local'\n  unit?: '%' | 'rem' | 'px'\n}"),
    );
    interfaces.insert(
        String::from("DashboardSidebarProps"),
        String::from("Pick<UseResizableProps, 'id' | 'side' | 'minSize'> & {\n  mode?: T\n}"),
    );
    let r = resolve_type_args("DashboardSidebarProps<T>", &interfaces, &aliases);
    assert_eq!(
        r.as_str(),
        "{ id?: string; side?: 'left' | 'right'; minSize?: number; mode?: T }"
    );
}

#[test]
fn strip_comments_preserves_string_literals() {
    let stripped = strip_comments("a?: 'http://x' /* c */ | 'b' // tail");
    assert_eq!(stripped.as_str(), "a?: 'http://x'  | 'b' ");
}

#[test]
fn unresolved_property_shapes_preserve_value_type_boundaries() {
    let aliases = FxHashMap::from_iter([
        (cstr!("Props"), String::from("{ value?: Missing }")),
        (
            cstr!("Inherited"),
            String::from("Missing & { value?: string }"),
        ),
        (cstr!("BaseKeys"), String::from("'first' | 'second'")),
        (cstr!("ChainedKeys"), String::from("BaseKeys")),
        (cstr!("Keys"), String::from("'first' | 'second'")),
        (cstr!("MissingAlias"), String::from("MissingKeys")),
        (cstr!("CycleA"), String::from("CycleB")),
        (cstr!("CycleB"), String::from("CycleA")),
        (
            cstr!("Conditional"),
            String::from("T extends { __output: infer R } ? R : never"),
        ),
        (
            cstr!("InferProps"),
            String::from("T extends { __output: infer R } ? R : never"),
        ),
        (
            cstr!("ResolvedConditional"),
            String::from("InferProps<{ __output: { title: string; count: number } }>"),
        ),
    ]);
    for expression in [
        "{ value?: Missing }",
        "Props",
        "Record<'value', Missing>",
        "Record<Keys, Missing>",
        "Record<ChainedKeys, Missing>",
        "ResolvedConditional",
        "Partial<{ title: string }> extends object ? { title: string } : never",
        "{ kind: 'a' } extends object ? { first: string } | { second: number } : never",
    ] {
        assert_eq!(
            unresolved_property_shape_references(expression, &FxHashMap::default(), &aliases),
            Vec::<String>::new()
        );
    }
    for expression in [
        "Missing",
        "Inherited",
        "Partial<Missing>",
        "Record<MissingKeys, string>",
        "Record<MissingAlias, string>",
        "Record<CycleA, string>",
        "Unknown | Missing",
        "{ value?: string } | Missing",
        "Conditional",
        "T extends object ? U extends object ? { value: string } : never : never",
    ] {
        let expected = if expression == "Unknown | Missing" {
            vec![cstr!("Unknown"), cstr!("Missing")]
        } else if expression == "Partial<Missing>" {
            vec![cstr!("Missing")]
        } else if matches!(
            expression,
            "Record<MissingKeys, string>" | "Record<MissingAlias, string>"
        ) {
            vec![cstr!("MissingKeys")]
        } else if expression == "Record<CycleA, string>" {
            vec![cstr!("CycleA")]
        } else if expression == "Conditional" {
            vec![String::from("T extends { __output: infer R } ? R : never")]
        } else if expression.starts_with("T extends object") {
            vec![expression.to_compact_string()]
        } else {
            vec![cstr!("Missing")]
        };
        assert_eq!(
            unresolved_property_shape_references(expression, &FxHashMap::default(), &aliases),
            expected
        );
    }

    assert_eq!(
        resolve_type_to_object_body(
            "Record<ChainedKeys, string>",
            &FxHashMap::default(),
            &aliases,
        )
        .as_deref(),
        Some("first: string; second: string")
    );
    assert_eq!(
        resolve_type_to_object_body("ResolvedConditional", &FxHashMap::default(), &aliases,)
            .as_deref(),
        Some("title: string; count: number")
    );
    assert_eq!(
        resolve_type_to_object_body(
            "Partial<{ title: string }> extends object ? { title: string } : never",
            &FxHashMap::default(),
            &aliases,
        )
        .as_deref(),
        Some("title: string")
    );
}
