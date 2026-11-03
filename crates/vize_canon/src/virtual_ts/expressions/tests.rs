use super::reserved_props::rewrite_reserved_template_binding;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;

fn reserved_props() -> TemplateBindingAccess<'static> {
    TemplateBindingAccess::from_props(
        ["static", "default", "class"]
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}

#[test]
fn rewrites_reserved_prop_identifier() {
    assert_eq!(
        rewrite_reserved_template_binding("static", &reserved_props()).as_deref(),
        Some("props[\"static\"]")
    );
}

#[test]
fn rewrites_reserved_prop_inside_object_value() {
    assert_eq!(
        rewrite_reserved_template_binding("{ active: static }", &reserved_props()).as_deref(),
        Some("{ active: props[\"static\"] }")
    );
}

#[test]
fn rewrites_reserved_prop_shorthand() {
    assert_eq!(
        rewrite_reserved_template_binding("{ static, class }", &reserved_props()).as_deref(),
        Some("{ static: props[\"static\"], class: props[\"class\"] }")
    );
}

#[test]
fn leaves_property_keys_and_member_accesses_alone() {
    assert_eq!(
        rewrite_reserved_template_binding(
            "{ static: true, value: props.static, nested: item.default, active: static }",
            &reserved_props(),
        )
        .as_deref(),
        Some(
            "{ static: true, value: props.static, nested: item.default, active: props[\"static\"] }",
        )
    );
}

#[test]
fn leaves_literals_and_regexes_alone() {
    assert_eq!(
        rewrite_reserved_template_binding(
            "'static' + /static/.test(value) + `class` + static",
            &reserved_props(),
        )
        .as_deref(),
        Some("'static' + /static/.test(value) + `class` + props[\"static\"]")
    );
}

#[test]
fn preserves_non_ascii_literals_when_rewriting_reserved_props() {
    assert_eq!(
        rewrite_reserved_template_binding(
            "static ? 'アカウント' : `コンテンツ管理`",
            &reserved_props(),
        )
        .as_deref(),
        Some("props[\"static\"] ? 'アカウント' : `コンテンツ管理`")
    );
}

#[test]
fn ignores_non_reserved_props() {
    let props = TemplateBindingAccess::from_props(["count"].into_iter().map(Into::into).collect());
    assert_eq!(rewrite_reserved_template_binding("count + 1", &props), None);
}

#[test]
fn preserves_typescript_as_assertions_when_prop_is_named_as() {
    let props = TemplateBindingAccess::from_props(["as"].into_iter().map(Into::into).collect());

    assert_eq!(
        rewrite_reserved_template_binding("(value as any)", &props),
        None
    );
    assert_eq!(
        rewrite_reserved_template_binding("{ ['--demo-value' as any]: value }", &props),
        None
    );
    assert_eq!(
        rewrite_reserved_template_binding(
            "{ focusin: (event: FocusEvent) => onFocus(event.target as HTMLElement) }",
            &props,
        ),
        None
    );
}

#[test]
fn still_rewrites_as_when_used_as_template_prop_identifier() {
    let props = TemplateBindingAccess::from_props(["as"].into_iter().map(Into::into).collect());

    assert_eq!(
        rewrite_reserved_template_binding("as", &props).as_deref(),
        Some("props[\"as\"]")
    );
    assert_eq!(
        rewrite_reserved_template_binding("{ as, tag: as }", &props).as_deref(),
        Some("{ as: props[\"as\"], tag: props[\"as\"] }")
    );
}

#[test]
fn reserved_props_inside_template_literal_substitutions_are_rewritten() {
    let props = TemplateBindingAccess::from_props(["from"].into_iter().map(Into::into).collect());
    assert_eq!(
        rewrite_reserved_template_binding("`Times in ${label(zone, from)}`", &props).as_deref(),
        Some("`Times in ${label(zone, props[\"from\"])}`")
    );
}

#[test]
fn combined_reserved_rewrites_map_both_keys_to_authored_bytes() {
    use super::reserved_props::map_rewritten_template_binding;
    use crate::virtual_ts::types::VizeMapping;

    let bindings =
        TemplateBindingAccess::from_props(["static", "from"].into_iter().map(Into::into).collect());
    for (newline, extra_byte) in [("\n", 0usize), ("\r\n", 1)] {
        let expression = format!("`雪{newline}${{static}} ${{from}}`");
        let rewritten = rewrite_reserved_template_binding(&expression, &bindings).unwrap();
        assert_eq!(
            rewritten.as_str(),
            format!("`雪{newline}${{props[\"static\"]}} ${{props[\"from\"]}}`")
        );
        let mut emitted = std::string::String::from("/* emit */\n");
        let generated_start = emitted.len();
        emitted.push_str(&rewritten);
        let mut mappings = Vec::new();
        map_rewritten_template_binding(
            &emitted,
            &mut mappings,
            generated_start,
            101,
            &expression,
            &bindings,
        );
        let generated = |start: usize, end: usize| {
            generated_start + start + extra_byte..generated_start + end + extra_byte
        };
        let source = |start: usize, end: usize| 101 + start + extra_byte..101 + end + extra_byte;
        assert_eq!(
            mappings,
            vec![
                VizeMapping::new(
                    generated_start..generated_start + 7 + extra_byte,
                    101..108 + extra_byte
                ),
                VizeMapping::new(generated(14, 20), source(7, 13)),
                VizeMapping::new(generated(22, 26), source(13, 17)),
                VizeMapping::new(generated(33, 37), source(17, 21)),
                VizeMapping::new(generated(39, 41), source(21, 23)),
            ]
        );
        for mapping in &mappings {
            assert_eq!(
                emitted.get(mapping.gen_range.clone()),
                expression.get(mapping.src_range.start - 101..mapping.src_range.end - 101)
            );
        }
        for receiver_start in [7, 26] {
            let mapped_rows = mappings
                .iter()
                .enumerate()
                .filter_map(|(index, mapping)| {
                    mapping
                        .gen_range
                        .contains(&(generated_start + receiver_start + extra_byte))
                        .then_some(index)
                })
                .collect::<Vec<_>>();
            assert_eq!(mapped_rows, Vec::<usize>::new());
        }
        let mut diagnostics = Vec::new();
        bindings.append_instance_diagnostic_prefixes(&mut diagnostics);
        diagnostics.sort_by_key(|row| row.gen_range.start);
        assert_eq!(
            diagnostics,
            vec![
                VizeMapping::new(generated(7, 14), source(7, 13)),
                VizeMapping::new(generated(26, 33), source(17, 21)),
            ]
        );
    }
}

#[test]
fn nested_template_substitutions_keep_quasis_strings_regexes_and_members() {
    let props = TemplateBindingAccess::from_props(["from"].into_iter().map(Into::into).collect());
    let expression = r#"`from \${from} ${`nested ${from} ${({ from: 'from', value: from }).value}`} ${/}/.test('from') ? from : item.from}`"#;
    assert_eq!(
        rewrite_reserved_template_binding(expression, &props).as_deref(),
        Some(
            r#"`from \${from} ${`nested ${props["from"]} ${({ from: 'from', value: props["from"] }).value}`} ${/}/.test('from') ? props["from"] : item.from}`"#
        )
    );
}

#[test]
fn substitutions_keep_comment_braces_out_of_the_template_grammar() {
    let props = TemplateBindingAccess::from_props(["from"].into_iter().map(Into::into).collect());
    assert_eq!(
        rewrite_reserved_template_binding("`x ${/* } from */ from // } from\n} tail from`", &props)
            .as_deref(),
        Some("`x ${/* } from */ props[\"from\"] // } from\n} tail from`")
    );
}

#[test]
fn ternary_value_before_a_colon_is_not_an_object_property_key() {
    assert_eq!(
        rewrite_reserved_template_binding("static ? default : class", &reserved_props()).as_deref(),
        Some("props[\"static\"] ? props[\"default\"] : props[\"class\"]")
    );
}

#[test]
fn contextual_prop_reads_respect_authored_callback_bindings_inside_substitutions() {
    let props = TemplateBindingAccess::from_props(["from"].into_iter().map(Into::into).collect());
    assert_eq!(
        rewrite_reserved_template_binding("`${((from) => from)(1)} ${from}`", &props).as_deref(),
        Some("`${((from) => from)(1)} ${props[\"from\"]}`")
    );
}
