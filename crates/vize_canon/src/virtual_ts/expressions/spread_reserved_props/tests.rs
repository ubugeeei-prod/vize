use super::*;

fn props(names: &[&str]) -> TemplateBindingAccess<'static> {
    TemplateBindingAccess::from_props(names.iter().copied().map(Into::into).collect())
}

fn rewrite(expression: &str, names: &TemplateBindingAccess) -> Option<SpreadRewrite> {
    rewrite_reserved_spread_references(expression, names, &ScopeChain::new(), ScopeId::ROOT)
}

#[test]
fn no_reserved_spelling_takes_the_parser_free_fast_path() {
    let names = props(&["as"]);
    assert!(!has_reserved_name_candidate("{ value: bag.count }", &names));
    assert_eq!(rewrite("{ value: bag.count }", &names), None);
}

#[test]
fn escaped_reserved_identifier_reaches_semantic_rewriting() {
    for (expression, expected) in [
        (r"fr\u006fm", r#"props["fr\u006fm"]"#),
        (r"fr\u{6f}m", r#"props["fr\u{6f}m"]"#),
    ] {
        let rewritten = rewrite(expression, &props(&["from"]))
            .expect("escaped reserved read should be rewritten");
        assert_eq!(rewritten.code, expected);
        let authored = rewritten
            .segments
            .iter()
            .find(|segment| segment.source == (0..expression.len()))
            .expect("escaped read should keep its authored mapping");
        assert_eq!(
            rewritten.code.get(authored.generated.clone()),
            Some(expression)
        );
    }
}

#[test]
fn escaped_identifier_roles_stay_owned_by_semantics() {
    let names = props(&["from"]);
    for expression in [
        r"item.fr\u006fm",
        r"{ fr\u006fm: 1 }",
        r"value as fr\u006fm",
        r"((fr\u006fm) => fr\u006fm)(1)",
    ] {
        assert_eq!(rewrite(expression, &names), None, "{expression}");
    }
}

#[test]
fn rewrites_only_unbound_value_references() {
    let expression = "{ as, tag: as, member: item.as, computed: item[as], template: `${as}`, cast: count as /* as */ number, kind: typeof count, local: ((as) => as)('x') }";
    let rewritten = rewrite(expression, &props(&["as", "typeof"]))
        .expect("unbound references should be rewritten");

    assert_eq!(
        rewritten.code,
        "{ as: props[\"as\"], tag: props[\"as\"], member: item.as, computed: item[props[\"as\"]], template: `${props[\"as\"]}`, cast: count as /* as */ number, kind: typeof count, local: ((as) => as)('x') }",
    );
}

#[test]
fn preserves_commented_generic_method_accessor_generator_and_async_keys() {
    let expression = "{ /* as */ as /* key */ <T>(value: T) { return value }, get as() { return 'x' }, set as(value) {}, *as<T>() { yield 1 }, async as<T>() {}, async *as<T>() { yield 1 } }";
    assert_eq!(rewrite(expression, &props(&["as"])), None);
}

#[test]
fn preserves_reserved_operators_comments_and_type_references() {
    let expression = "{ type: typeof value, member: key in value, check: value instanceof Type, made: new Type(), cast: value as typeof as, run: async function* () { await value; yield value } }";
    let names = props(&["typeof", "in", "instanceof", "new", "as", "await", "yield"]);
    assert_eq!(rewrite(expression, &names), None);
}

#[test]
fn rewrite_segments_keep_offsets_after_an_expanded_shorthand_exact() {
    let expression = "{ as, value: bag.missing }";
    let rewritten = rewrite(expression, &props(&["as"])).expect("shorthand should be rewritten");
    let source_missing = expression.find("missing").unwrap();
    let generated_missing = rewritten.code.find("missing").unwrap();
    let segment = rewritten
        .segments
        .iter()
        .find(|segment| segment.generated.contains(&generated_missing))
        .expect("missing must have an unchanged mapping segment");

    assert_eq!(
        generated_missing - segment.generated.start,
        source_missing - segment.source.start,
    );
}

#[test]
fn reserved_property_navigation_maps_only_authored_bytes() {
    for (expression, expected) in [
        ("from", vec![(7..11, 0..4)]),
        (
            "{ from }",
            vec![(0..2, 0..2), (2..6, 2..6), (15..19, 2..6), (21..23, 6..8)],
        ),
        (
            "`雪 ${from}`",
            vec![(0..7, 0..7), (14..18, 7..11), (20..22, 11..13)],
        ),
    ] {
        let rewritten = rewrite(expression, &props(&["from"]))
            .expect("reserved reads must keep their symbol identity");
        let actual = rewritten
            .segments
            .iter()
            .map(|segment| (segment.generated.clone(), segment.source.clone()))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{expression}");
        for segment in &rewritten.segments {
            assert_eq!(
                rewritten.code.get(segment.generated.clone()),
                expression.get(segment.source.clone()),
                "{expression}"
            );
        }
    }
}
