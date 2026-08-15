use crate::ir::DirectiveIRNode;
use vize_atelier_core::ExpressionNode;
use vize_carton::{String, cstr};

use super::super::context::GenerateContext;

fn directive_resolution_var(name: &str) -> String {
    let mut ident = String::with_capacity(name.len() + 11);
    ident.push_str("_directive_");
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident.push(ch);
        } else {
            ident.push('_');
        }
    }
    ident
}

fn directive_arg(ctx: &GenerateContext, directive: &DirectiveIRNode<'_>) -> String {
    if let Some(ref arg) = directive.dir.arg {
        match arg {
            ExpressionNode::Simple(exp) => {
                if exp.is_static {
                    cstr!("\"{}\"", exp.content)
                } else {
                    ctx.resolve_expression_node(exp)
                }
            }
            ExpressionNode::Compound(compound) => {
                ctx.resolve_expression(compound.loc.span.slice(ctx.source))
            }
        }
    } else {
        vize_carton::CompactString::from("undefined")
    }
}

fn directive_value(ctx: &GenerateContext, directive: &DirectiveIRNode<'_>) -> String {
    if let Some(ref exp) = directive.dir.exp {
        match exp {
            ExpressionNode::Simple(e) => {
                if e.is_static {
                    cstr!("\"{}\"", e.content)
                } else {
                    ctx.resolve_expression_node(e)
                }
            }
            ExpressionNode::Compound(compound) => {
                ctx.resolve_expression(compound.loc.span.slice(ctx.source))
            }
        }
    } else {
        vize_carton::CompactString::from("undefined")
    }
}

fn directive_modifiers(directive: &DirectiveIRNode<'_>) -> Option<String> {
    if directive.dir.modifiers.is_empty() {
        return None;
    }

    let modifiers = directive
        .dir
        .modifiers
        .iter()
        .map(|modifier| cstr!("{}: true", modifier.content))
        .collect::<std::vec::Vec<_>>()
        .join(", ");

    Some(cstr!("{{ {} }}", modifiers))
}

/// Generate Directive
pub(super) fn generate_directive(ctx: &mut GenerateContext, directive: &DirectiveIRNode<'_>) {
    let element = cstr!("n{}", directive.element);

    // Handle v-show
    if directive.name == "vShow" {
        ctx.use_helper("applyVShow");
        let value = if let Some(ref exp) = directive.dir.exp {
            match exp {
                ExpressionNode::Simple(e) => {
                    if e.is_static {
                        cstr!("\"{}\"", e.content)
                    } else {
                        ctx.resolve_expression_node(e)
                    }
                }
                _ => vize_carton::CompactString::from("undefined"),
            }
        } else {
            vize_carton::CompactString::from("undefined")
        };
        ctx.push_line_fmt(format_args!("_applyVShow({}, () => ({}))", element, value));
        return;
    }

    if directive.name == "vCloak" {
        ctx.push_line_fmt(format_args!("{element}.removeAttribute(\"v-cloak\")"));
        return;
    }

    // Handle v-model on elements
    if directive.name == "model" {
        generate_v_model(ctx, directive);
        return;
    }

    let value = directive_value(ctx, directive);
    let arg = directive_arg(ctx, directive);
    let modifiers = directive_modifiers(directive);

    if directive.builtin {
        let name = &directive.name;
        match modifiers {
            Some(modifiers) => ctx.push_line_fmt(format_args!(
                "_withDirectives({}, [[_{}, {}, {}, {}]])",
                element, name, value, arg, modifiers
            )),
            None => ctx.push_line_fmt(format_args!(
                "_withDirectives({}, [[_{}, {}, {}]])",
                element, name, value, arg
            )),
        }
        return;
    }

    ctx.use_helper("withDirectives");
    let resolved = directive_resolution_var(directive.name);
    match modifiers {
        Some(modifiers) => ctx.push_line_fmt(format_args!(
            "_withDirectives({}, [[{}, {}, {}, {}]])",
            element, resolved, value, arg, modifiers
        )),
        None => ctx.push_line_fmt(format_args!(
            "_withDirectives({}, [[{}, {}, {}]])",
            element, resolved, value, arg
        )),
    }
}

/// Generate v-model for element
fn generate_v_model(ctx: &mut GenerateContext, directive: &DirectiveIRNode<'_>) {
    let element = cstr!("n{}", directive.element);

    let binding = if let Some(ref exp) = directive.dir.exp {
        match exp {
            ExpressionNode::Simple(e) => String::new(e.content),
            _ => vize_carton::String::from(""),
        }
    } else {
        vize_carton::String::from("")
    };

    let helper = if directive.tag == "select" {
        "applySelectModel"
    } else if directive.tag == "textarea" {
        "applyTextModel"
    } else if directive.tag == "input" {
        match directive.input_type {
            "checkbox" => "applyCheckboxModel",
            "radio" => "applyRadioModel",
            _ => "applyTextModel",
        }
    } else {
        "applyTextModel"
    };

    ctx.use_helper(helper);

    // Build modifiers options
    let modifiers = &directive.dir.modifiers;
    let mut mod_parts: std::vec::Vec<String> = std::vec::Vec::new();
    for m in modifiers.iter() {
        match m.content {
            "lazy" => mod_parts.push("lazy: true".into()),
            "number" => mod_parts.push("number: true".into()),
            "trim" => mod_parts.push("trim: true".into()),
            _ => {}
        }
    }

    if mod_parts.is_empty() {
        ctx.push_line_fmt(format_args!(
            "_{}({}, () => (_ctx.{}), _value => (_ctx.{} = _value))",
            helper, element, binding, binding
        ));
    } else {
        ctx.push_line_fmt(format_args!(
            "_{}({}, () => (_ctx.{}), _value => (_ctx.{} = _value), {{ {} }})",
            helper,
            element,
            binding,
            binding,
            mod_parts.join(",")
        ));
    }
}
