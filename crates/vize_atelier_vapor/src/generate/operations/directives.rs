use crate::ir::DirectiveIRNode;
use vize_atelier_core::ExpressionNode;
use vize_carton::{cstr, String};

use super::super::context::GenerateContext;

/// Generate Directive
pub(super) fn generate_directive(ctx: &mut GenerateContext, directive: &DirectiveIRNode<'_>) {
    let element = cstr!("n{}", directive.element);

    // Handle v-show
    if directive.name.as_str() == "vShow" {
        ctx.use_helper("applyVShow");
        let value = if let Some(ref exp) = directive.dir.exp {
            match exp {
                ExpressionNode::Simple(e) => {
                    if e.is_static {
                        cstr!("\"{}\"", e.content)
                    } else {
                        ctx.resolve_expression(&e.content)
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

    // Handle v-model on elements
    if directive.name.as_str() == "model" {
        generate_v_model(ctx, directive);
        return;
    }

    let name = &directive.name;

    let arg = if let Some(ref arg) = directive.dir.arg {
        match arg {
            ExpressionNode::Simple(exp) => {
                if exp.is_static {
                    cstr!("\"{}\"", exp.content)
                } else {
                    vize_carton::CompactString::from(exp.content.as_str())
                }
            }
            _ => vize_carton::CompactString::from("undefined"),
        }
    } else {
        vize_carton::CompactString::from("undefined")
    };

    let value = if let Some(ref exp) = directive.dir.exp {
        match exp {
            ExpressionNode::Simple(e) => {
                if e.is_static {
                    cstr!("\"{}\"", e.content)
                } else {
                    vize_carton::CompactString::from(e.content.as_str())
                }
            }
            _ => vize_carton::CompactString::from("undefined"),
        }
    } else {
        vize_carton::CompactString::from("undefined")
    };

    ctx.push_line_fmt(format_args!(
        "_withDirectives({}, [[_{}, {}, {}]])",
        element, name, value, arg
    ));
}

/// Generate v-model for element
fn generate_v_model(ctx: &mut GenerateContext, directive: &DirectiveIRNode<'_>) {
    let element = cstr!("n{}", directive.element);

    let binding = if let Some(ref exp) = directive.dir.exp {
        match exp {
            ExpressionNode::Simple(e) => e.content.clone(),
            _ => vize_carton::String::from(""),
        }
    } else {
        vize_carton::String::from("")
    };

    let helper = if directive.tag.as_str() == "select" {
        "applySelectModel"
    } else if directive.tag.as_str() == "textarea" {
        "applyTextModel"
    } else if directive.tag.as_str() == "input" {
        match directive.input_type.as_str() {
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
        match m.content.as_str() {
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
