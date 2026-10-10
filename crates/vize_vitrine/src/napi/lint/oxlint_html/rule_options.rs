//! Strict projection of the existing seven Oxlint rule-option schemas.

use serde_json::Value;
use vize_l0::{String, cstr};

use super::super::lint_options::{
    HtmlSelfClosingHtmlOptionsNapi, HtmlSelfClosingOptionsNapi, NoMutatingPropsOptionsNapi,
    PatinaLintOptionsNapi, SfcElementOrderOptionsNapi, configure_patina_rule_options,
};

/// The caller owns catalog validation and tuple arity; omission keeps defaults.
/// Validate the complete payload before replacing any existing option field.
pub(super) fn parse_rule_option(
    name: &str,
    payload: Option<&Value>,
    options: &mut PatinaLintOptionsNapi,
) -> Result<(), String> {
    let Some(payload) = payload else {
        return Ok(());
    };
    match name {
        "vue/component-name-in-template-casing" => {
            options.component_name_in_template_casing =
                Some(string_enum(payload, &["PascalCase", "kebab-case"], name)?.to_owned());
        }
        "script/custom-event-name-casing" => {
            options.custom_event_name_casing =
                Some(string_enum(payload, &["camelCase", "kebab-case"], name)?.to_owned());
        }
        "vue/no-mutating-props" => {
            let object = strict_object(payload, &["shallowOnly"])?;
            let shallow_only = object
                .get("shallowOnly")
                .map(|value| {
                    value
                        .as_bool()
                        .ok_or_else(|| cstr!("shallowOnly must be a boolean"))
                })
                .transpose()?;
            options.no_mutating_props = Some(NoMutatingPropsOptionsNapi { shallow_only });
        }
        "vue/sfc-element-order" => {
            let object = strict_object(payload, &["order"])?;
            let order = object
                .get("order")
                .map(|value| {
                    let order = value
                        .as_array()
                        .ok_or_else(|| cstr!("order must be an array"))?;
                    for group in order {
                        if !group.is_string()
                            && !group
                                .as_array()
                                .is_some_and(|items| items.iter().all(Value::is_string))
                        {
                            return Err(cstr!("order entries must be strings or string arrays"));
                        }
                    }
                    // Preserve authored empty groups; the existing converter
                    // owns their effective normalization, including whitespace.
                    Ok(order.clone())
                })
                .transpose()?;
            options.sfc_element_order = Some(SfcElementOrderOptionsNapi { order });
        }
        "vue/html-self-closing" => {
            options.html_self_closing = Some(parse_html_self_closing(payload)?);
        }
        "vue/v-on-event-hyphenation" => {
            options.v_on_event_hyphenation =
                Some(string_enum(payload, &["always", "never"], name)?.to_owned());
        }
        "vue/attribute-hyphenation" => {
            options.attribute_hyphenation =
                Some(string_enum(payload, &["always", "never"], name)?.to_owned());
        }
        _ => return Err(cstr!("rule {name} accepts no option payload")),
    }
    Ok(())
}

/// Apply only validated rule fields through the unchanged native converters.
/// The caller must establish its explicit enabled set before this conversion.
pub(super) fn apply_rule_options(
    linter: vize_patina::Linter,
    options: PatinaLintOptionsNapi,
) -> vize_patina::Linter {
    configure_patina_rule_options(
        linter,
        options.component_name_in_template_casing.as_deref(),
        options.custom_event_name_casing.as_deref(),
        options.no_mutating_props,
        options.sfc_element_order,
        options.html_self_closing,
        options.v_on_event_hyphenation.as_deref(),
        options.attribute_hyphenation.as_deref(),
    )
}

fn strict_object<'a>(value: &'a Value, keys: &[&str]) -> Result<&'a Value, String> {
    let object = value
        .as_object()
        .ok_or_else(|| cstr!("option payload must be an object"))?;
    if let Some(key) = object.keys().find(|key| !keys.contains(&key.as_str())) {
        return Err(cstr!("unsupported option key: {key}"));
    }
    Ok(value)
}

fn string_enum<'a>(value: &'a Value, allowed: &[&str], field: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|value| allowed.contains(value))
        .ok_or_else(|| cstr!("invalid option value for {field}"))
}

fn parse_html_self_closing(value: &Value) -> Result<HtmlSelfClosingOptionsNapi, String> {
    let object = strict_object(value, &["html", "svg", "math"])?;
    let html = object
        .get("html")
        .map(|value| {
            let html = strict_object(value, &["void", "normal", "component"])?;
            Ok::<_, String>(HtmlSelfClosingHtmlOptionsNapi {
                r#void: self_closing_field(html, "void")?.map(str::to_owned),
                normal: self_closing_field(html, "normal")?.map(str::to_owned),
                component: self_closing_field(html, "component")?.map(str::to_owned),
            })
        })
        .transpose()?;
    Ok(HtmlSelfClosingOptionsNapi {
        html,
        svg: self_closing_field(object, "svg")?.map(str::to_owned),
        math: self_closing_field(object, "math")?.map(str::to_owned),
    })
}

fn self_closing_field<'a>(object: &'a Value, field: &str) -> Result<Option<&'a str>, String> {
    object
        .get(field)
        .map(|value| string_enum(value, &["always", "never", "any"], field))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::{PatinaLintOptionsNapi, parse_rule_option};
    use serde_json::json;

    #[test]
    fn seven_schema_families_accept_authored_payloads() {
        let mut options = PatinaLintOptionsNapi::default();
        for (name, payload) in [
            ("vue/component-name-in-template-casing", json!("kebab-case")),
            ("script/custom-event-name-casing", json!("camelCase")),
            ("vue/no-mutating-props", json!({ "shallowOnly": true })),
            (
                "vue/sfc-element-order",
                json!({ "order": [[], ["template"], "script"] }),
            ),
            (
                "vue/html-self-closing",
                json!({ "html": { "void": "never", "normal": "always", "component": "any" }, "svg": "never", "math": "always" }),
            ),
            ("vue/v-on-event-hyphenation", json!("never")),
            ("vue/attribute-hyphenation", json!("always")),
        ] {
            parse_rule_option(name, Some(&payload), &mut options).unwrap();
        }
        assert_eq!(
            options.component_name_in_template_casing.as_deref(),
            Some("kebab-case")
        );
        assert_eq!(
            options.custom_event_name_casing.as_deref(),
            Some("camelCase")
        );
        assert_eq!(options.no_mutating_props.unwrap().shallow_only, Some(true));
        assert_eq!(
            options.sfc_element_order.unwrap().order,
            Some(vec![json!([]), json!(["template"]), json!("script")])
        );
        let self_closing = options.html_self_closing.unwrap();
        let html = self_closing.html.unwrap();
        assert_eq!(html.r#void.as_deref(), Some("never"));
        assert_eq!(html.normal.as_deref(), Some("always"));
        assert_eq!(html.component.as_deref(), Some("any"));
        assert_eq!(self_closing.svg.as_deref(), Some("never"));
        assert_eq!(self_closing.math.as_deref(), Some("always"));
        assert_eq!(options.v_on_event_hyphenation.as_deref(), Some("never"));
        assert_eq!(options.attribute_hyphenation.as_deref(), Some("always"));
    }

    #[test]
    fn omission_and_empty_objects_keep_existing_defaults() {
        let mut options = PatinaLintOptionsNapi::default();
        parse_rule_option("vue/html-self-closing", None, &mut options).unwrap();
        assert!(options.html_self_closing.is_none());
        for name in [
            "vue/no-mutating-props",
            "vue/sfc-element-order",
            "vue/html-self-closing",
        ] {
            parse_rule_option(name, Some(&json!({})), &mut options).unwrap();
        }
        assert_eq!(options.no_mutating_props.unwrap().shallow_only, None);
        assert_eq!(options.sfc_element_order.unwrap().order, None);
        let self_closing = options.html_self_closing.unwrap();
        assert!(self_closing.html.is_none());
        assert!(self_closing.svg.is_none());
        assert!(self_closing.math.is_none());
    }

    #[test]
    fn all_enum_values_and_empty_order_shapes_are_admitted() {
        let mut options = PatinaLintOptionsNapi::default();
        for (name, values) in [
            (
                "vue/component-name-in-template-casing",
                ["PascalCase", "kebab-case"],
            ),
            (
                "script/custom-event-name-casing",
                ["camelCase", "kebab-case"],
            ),
            ("vue/v-on-event-hyphenation", ["always", "never"]),
            ("vue/attribute-hyphenation", ["always", "never"]),
        ] {
            for value in values {
                parse_rule_option(name, Some(&json!(value)), &mut options).unwrap();
            }
        }
        for style in ["always", "never", "any"] {
            let payload = json!({ "html": { "void": style, "normal": style, "component": style }, "svg": style, "math": style });
            parse_rule_option("vue/html-self-closing", Some(&payload), &mut options).unwrap();
        }
        for order in [json!([]), json!([[]]), json!(["", [" ", "template"]])] {
            let payload = json!({ "order": order });
            parse_rule_option("vue/sfc-element-order", Some(&payload), &mut options).unwrap();
            assert_eq!(
                options.sfc_element_order.take().unwrap().order,
                Some(order.as_array().unwrap().clone())
            );
        }
        parse_rule_option(
            "vue/no-mutating-props",
            Some(&json!({ "shallowOnly": false })),
            &mut options,
        )
        .unwrap();
        assert_eq!(options.no_mutating_props.unwrap().shallow_only, Some(false));
    }

    #[test]
    fn strict_shapes_reject_invalid_payloads_without_applying_them() {
        let mut options = PatinaLintOptionsNapi::default();
        for (name, payload) in [
            ("vue/component-name-in-template-casing", json!("pascalcase")),
            ("script/custom-event-name-casing", json!("PascalCase")),
            ("vue/no-mutating-props", json!({ "shallowOnly": "true" })),
            ("vue/no-mutating-props", json!(null)),
            ("vue/no-mutating-props", json!([])),
            (
                "vue/no-mutating-props",
                json!({ "shallowOnly": true, "extra": false }),
            ),
            (
                "vue/sfc-element-order",
                json!({ "order": [["template", 1]] }),
            ),
            ("vue/sfc-element-order", json!({ "order": null })),
            ("vue/sfc-element-order", json!({ "order": [], "extra": [] })),
            ("vue/html-self-closing", json!({ "html": null })),
            (
                "vue/html-self-closing",
                json!({ "html": { "normal": "always", "extra": "any" } }),
            ),
            (
                "vue/html-self-closing",
                json!({ "html": { "normal": "always" }, "math": "invalid" }),
            ),
            ("vue/html-self-closing", json!({ "svg": true })),
            ("vue/html-self-closing", json!({ "extra": "always" })),
            ("vue/v-on-event-hyphenation", json!("any")),
            ("vue/attribute-hyphenation", json!(true)),
            ("vue/no-v-html", json!({})),
        ] {
            assert!(parse_rule_option(name, Some(&payload), &mut options).is_err());
        }
        assert!(options.component_name_in_template_casing.is_none());
        assert!(options.custom_event_name_casing.is_none());
        assert!(options.no_mutating_props.is_none());
        assert!(options.sfc_element_order.is_none());
        assert!(options.html_self_closing.is_none());
        assert!(options.v_on_event_hyphenation.is_none());
        assert!(options.attribute_hyphenation.is_none());
    }
}
