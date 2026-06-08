use super::parse_v_for_scope_expression;

#[test]
fn parse_scope_expression_keeps_destructured_value_pattern() {
    let aliases = parse_v_for_scope_expression("{ id, name: label } in items").unwrap();

    assert_eq!(aliases.value_pattern.as_str(), "{ id, name: label }");
    assert_eq!(
        aliases
            .value_bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        vec!["id", "label"]
    );
    assert!(aliases.key_alias.is_none());
    assert!(aliases.index_alias.is_none());
}

#[test]
fn parse_scope_expression_splits_tuple_around_destructured_value() {
    let aliases = parse_v_for_scope_expression("({ id, meta: { slug } }, index) in items").unwrap();

    assert_eq!(aliases.value_pattern.as_str(), "{ id, meta: { slug } }");
    assert_eq!(
        aliases
            .value_bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        vec!["id", "slug"]
    );
    assert_eq!(aliases.key_alias.as_deref(), Some("index"));
    assert!(aliases.index_alias.is_none());
}
