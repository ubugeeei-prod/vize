use super::reserved_props::rewrite_reserved_template_binding;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;

fn reserved_props() -> TemplateBindingAccess {
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
