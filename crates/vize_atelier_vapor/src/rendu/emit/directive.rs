use vize_carton::{String, ToCompactString, appendln, cstr};

use super::super::{VaporDirective, VaporName, VaporPlan, VaporProperty};
use super::property::{expression, name, quote_js, use_helper};

pub(super) fn emit_element_directive(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    tag: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    match directive.name.as_ref() {
        "bind" => emit_bind(plan, directive, variable, pad, out, helpers),
        "on" => emit_on(plan, directive, variable, pad, out, helpers),
        "model" => emit_model(plan, directive, variable, tag, pad, out, helpers),
        "show" => emit_show(plan, directive, variable, pad, out, helpers),
        "html" | "text" => emit_content(plan, directive, variable, pad, out, helpers),
        _ => emit_custom(plan, directive, variable, pad, out, helpers),
    }
}

pub(super) fn emit_component_directives(
    plan: &VaporPlan,
    properties: &[VaporProperty],
    variable: &str,
    indent: usize,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let mut pad = String::default();
    for _ in 0..indent {
        pad.push_str("  ");
    }
    for property in properties {
        let VaporProperty::Directive(directive) = property else {
            continue;
        };
        match directive.name.as_ref() {
            "show" => emit_show(plan, directive, variable, &pad, out, helpers),
            "bind" | "on" | "model" | "html" | "text" => {}
            _ => emit_custom(plan, directive, variable, &pad, out, helpers),
        }
    }
}

fn emit_bind(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let Some(value) = directive.expression else {
        return;
    };
    match directive.argument.as_ref() {
        Some(argument) => {
            use_helper(helpers, "setProp");
            appendln!(
                out,
                pad,
                "_setProp(",
                variable,
                ", ",
                name(plan, argument).as_str(),
                ", ",
                expression(plan, value),
                ")"
            );
        }
        None => {
            use_helper(helpers, "setDynamicProps");
            appendln!(
                out,
                pad,
                "_setDynamicProps(",
                variable,
                ", [",
                expression(plan, value),
                "])"
            );
        }
    }
}

fn emit_on(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let (Some(argument), Some(value)) = (directive.argument.as_ref(), directive.expression) else {
        return;
    };
    use_helper(helpers, "setProp");
    let handler = if directive.modifiers.is_empty() {
        expression(plan, value).to_compact_string()
    } else {
        use_helper(helpers, "withModifiers");
        cstr!(
            "_withModifiers({}, [{}])",
            expression(plan, value),
            modifier_values(&directive.modifiers)
        )
    };
    appendln!(
        out,
        pad,
        "_setProp(",
        variable,
        ", ",
        event_key(plan, argument).as_str(),
        ", ",
        handler.as_str(),
        ")"
    );
}

fn emit_model(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    tag: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let Some(value) = directive.expression else {
        return;
    };
    let helper = if tag == "select" {
        "applySelectModel"
    } else {
        "applyTextModel"
    };
    use_helper(helpers, helper);
    appendln!(
        out,
        pad,
        "_",
        helper,
        "(",
        variable,
        ", () => (",
        expression(plan, value),
        "), _value => ((",
        expression(plan, value),
        ") = _value), { ",
        modifier_entries(&directive.modifiers).as_str(),
        " })"
    );
}

fn emit_show(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let Some(value) = directive.expression else {
        return;
    };
    use_helper(helpers, "applyVShow");
    appendln!(
        out,
        pad,
        "_applyVShow(",
        variable,
        ", () => (",
        expression(plan, value),
        "))"
    );
}

fn emit_content(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    let Some(value) = directive.expression else {
        return;
    };
    use_helper(helpers, "renderEffect");
    use_helper(helpers, "setProp");
    let property = if directive.name.as_ref() == "html" {
        "innerHTML"
    } else {
        "textContent"
    };
    appendln!(
        out,
        pad,
        "_renderEffect(() => _setProp(",
        variable,
        ", ",
        quote_js(property).as_str(),
        ", ",
        expression(plan, value),
        "))"
    );
}

fn emit_custom(
    plan: &VaporPlan,
    directive: &VaporDirective,
    variable: &str,
    pad: &str,
    out: &mut String,
    helpers: &mut Vec<&'static str>,
) {
    use_helper(helpers, "resolveDirective");
    use_helper(helpers, "withDirectives");
    let value = directive
        .expression
        .map(|value| expression(plan, value))
        .unwrap_or("undefined");
    let argument = directive
        .argument
        .as_ref()
        .map(|argument| name(plan, argument))
        .unwrap_or_else(|| String::from("undefined"));
    appendln!(
        out,
        pad,
        "_withDirectives(",
        variable,
        ", [[_resolveDirective(",
        quote_js(&directive.name).as_str(),
        "), ",
        value,
        ", ",
        argument.as_str(),
        ", { ",
        modifier_entries(&directive.modifiers).as_str(),
        " }]])"
    );
}

pub(super) fn event_key(plan: &VaporPlan, name: &VaporName) -> String {
    match name {
        VaporName::Static(name) => {
            let mut event = String::from("on");
            let mut chars = name.chars();
            if let Some(first) = chars.next() {
                event.extend(first.to_uppercase());
            }
            event.extend(chars);
            quote_js(&event)
        }
        VaporName::Dynamic(value) => cstr!("\"on\" + ({})", expression(plan, *value)),
    }
}

pub(super) fn modifier_values(modifiers: &[Box<str>]) -> String {
    modifiers
        .iter()
        .map(|modifier| quote_js(modifier))
        .collect::<Vec<_>>()
        .join(", ")
        .into()
}

fn modifier_entries(modifiers: &[Box<str>]) -> String {
    modifiers
        .iter()
        .map(|modifier| cstr!("{}: true", quote_js(modifier)))
        .collect::<Vec<_>>()
        .join(", ")
        .into()
}
