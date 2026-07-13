use vize_carton::{String, ToCompactString, appendln, cstr};

use super::super::{
    VaporAttributeValue, VaporDirective, VaporExpressionId, VaporName, VaporPlan, VaporProperty,
};
use super::directive::{emit_element_directive, event_key, modifier_values};

pub(super) fn expression(plan: &VaporPlan, id: VaporExpressionId) -> &str {
    plan.expression(id)
        .expect("validated Vapor expression")
        .code
        .as_ref()
}

pub(super) fn name(plan: &VaporPlan, name: &VaporName) -> String {
    match name {
        VaporName::Static(name) => quote_js(name),
        VaporName::Dynamic(value) => expression(plan, *value).to_compact_string(),
    }
}

pub(super) fn object_key(plan: &VaporPlan, name: &VaporName) -> String {
    match name {
        VaporName::Static(name) => quote_js(name),
        VaporName::Dynamic(value) => cstr!("[{}]", expression(plan, *value)),
    }
}

pub(super) fn props_object(
    plan: &VaporPlan,
    properties: &[VaporProperty],
    getters: bool,
) -> String {
    let mut entries = Vec::new();
    for property in properties {
        match property {
            VaporProperty::Attribute {
                name: property_name,
                value,
                ..
            } => entries.push(wrap_prop(
                object_key(plan, property_name),
                attribute_value(plan, value.as_ref()),
                getters,
            )),
            VaporProperty::Spread {
                expression: value, ..
            } => entries.push(cstr!("...({})", expression(plan, *value))),
            VaporProperty::Directive(directive) => {
                emit_component_directive_props(plan, directive, &mut entries, getters);
            }
        }
    }
    cstr!("{{ {} }}", entries.join(", "))
}

fn emit_component_directive_props(
    plan: &VaporPlan,
    directive: &VaporDirective,
    entries: &mut Vec<String>,
    getters: bool,
) {
    match directive.name.as_ref() {
        "bind" => {
            let Some(value) = directive.expression else {
                return;
            };
            if let Some(argument) = directive.argument.as_ref() {
                entries.push(wrap_prop(
                    object_key(plan, argument),
                    expression(plan, value).to_compact_string(),
                    getters,
                ));
            } else {
                entries.push(cstr!("...({})", expression(plan, value)));
            }
        }
        "on" => {
            let (Some(argument), Some(value)) = (directive.argument.as_ref(), directive.expression)
            else {
                return;
            };
            let value = if directive.modifiers.is_empty() {
                expression(plan, value).to_compact_string()
            } else {
                cstr!(
                    "_withModifiers({}, [{}])",
                    expression(plan, value),
                    modifier_values(&directive.modifiers)
                )
            };
            entries.push(wrap_prop(event_key(plan, argument), value, getters));
        }
        "model" => {
            let Some(value) = directive.expression else {
                return;
            };
            entries.push(wrap_prop(
                quote_js("modelValue"),
                expression(plan, value).to_compact_string(),
                getters,
            ));
            entries.push(wrap_prop(
                quote_js("onUpdate:modelValue"),
                cstr!("_value => (({}) = _value)", expression(plan, value)),
                getters,
            ));
        }
        "html" | "text" => {
            let Some(value) = directive.expression else {
                return;
            };
            entries.push(wrap_prop(
                quote_js(if directive.name.as_ref() == "html" {
                    "innerHTML"
                } else {
                    "textContent"
                }),
                expression(plan, value).to_compact_string(),
                getters,
            ));
        }
        _ => {}
    }
}

fn wrap_prop(key: String, value: String, getters: bool) -> String {
    if getters {
        cstr!("{key}: () => ({value})")
    } else {
        cstr!("{key}: {value}")
    }
}

pub(super) fn emit_element_properties(
    plan: &VaporPlan,
    properties: &[VaporProperty],
    variable: &str,
    tag: &str,
    indent: usize,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let mut pad = String::default();
    for _ in 0..indent {
        pad.push_str("  ");
    }
    for property in properties {
        match property {
            VaporProperty::Attribute {
                name: property_name,
                value,
                ..
            } => {
                use_helper(helpers, "setProp");
                appendln!(
                    out,
                    pad.as_str(),
                    "_setProp(",
                    variable,
                    ", ",
                    name(plan, property_name).as_str(),
                    ", ",
                    attribute_value(plan, value.as_ref()).as_str(),
                    ")"
                );
            }
            VaporProperty::Spread {
                expression: value, ..
            } => {
                use_helper(helpers, "setDynamicProps");
                appendln!(
                    out,
                    pad.as_str(),
                    "_setDynamicProps(",
                    variable,
                    ", [",
                    expression(plan, *value),
                    "])"
                );
            }
            VaporProperty::Directive(directive) => {
                emit_element_directive(plan, directive, variable, tag, pad.as_str(), out, helpers)
            }
        }
    }
}

fn attribute_value(plan: &VaporPlan, value: Option<&VaporAttributeValue>) -> String {
    match value {
        None => String::from("true"),
        Some(VaporAttributeValue::Static(value)) => quote_js(value),
        Some(VaporAttributeValue::Expression(value)) => {
            expression(plan, *value).to_compact_string()
        }
    }
}

pub(super) fn quote_js(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            '\u{2028}' => quoted.push_str("\\u2028"),
            '\u{2029}' => quoted.push_str("\\u2029"),
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

pub(super) fn use_helper(helpers: &mut Vec<&'static str>, helper: &'static str) {
    if !helpers.contains(&helper) {
        helpers.push(helper);
    }
}
