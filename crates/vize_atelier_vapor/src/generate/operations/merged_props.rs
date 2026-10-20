//! `setDynamicProps` over upstream's ordered sources: for an element with a
//! `v-bind` object or computed key, every prop lands in a literal
//! group so the runtime merges them in authored order (later sources win,
//! `class`/`style` concatenate), exactly as `@vue/compiler-vapor` emits it.

use crate::ir::{MergedPropsSource, SetMergedPropsIRNode};
use vize_carton::{String, ToCompactString};

use super::super::{context::GenerateContext, setup::escape_js_string_literal};

pub(super) fn generate_set_merged_props(
    ctx: &mut GenerateContext,
    node: &SetMergedPropsIRNode<'_>,
) {
    let call = merged_props_call(ctx, node);
    ctx.push_line(&call);
}

/// `_setDynamicProps(nX, [sources])`, shared by statement and inline effects.
pub(crate) fn merged_props_call(
    ctx: &mut GenerateContext,
    node: &SetMergedPropsIRNode<'_>,
) -> String {
    ctx.use_helper("setDynamicProps");
    // One buffer for the whole call: no per-source strings to join.
    let mut call = String::from("_setDynamicProps(n");
    call.push_str(&node.element.to_compact_string());
    call.push_str(", [");
    for (index, source) in node.sources.iter().enumerate() {
        if index > 0 {
            call.push_str(", ");
        }
        match source {
            MergedPropsSource::Object(value) => {
                call.push_str(&ctx.resolve_expression_node(value));
            }
            MergedPropsSource::Group(props) => {
                call.push_str("{ ");
                for (index, prop) in props.iter().enumerate() {
                    if index > 0 {
                        call.push_str(", ");
                    }
                    let key = prop.key.content;
                    if !prop.key.is_static {
                        call.push('[');
                        call.push_str(&ctx.resolve_expression_node(&prop.key));
                        call.push(']');
                    } else if simple_identifier(key) {
                        call.push_str(key);
                    } else {
                        call.push('"');
                        call.push_str(&escape_js_string_literal(key));
                        call.push('"');
                    }
                    call.push_str(": ");
                    let list = prop.values.len() != 1;
                    if list {
                        call.push('[');
                    }
                    for (index, value) in prop.values.iter().enumerate() {
                        if index > 0 {
                            call.push_str(", ");
                        }
                        if value.is_static {
                            call.push('"');
                            call.push_str(&escape_js_string_literal(value.content));
                            call.push('"');
                        } else {
                            call.push_str(&ctx.resolve_expression_node(value));
                        }
                    }
                    if list {
                        call.push(']');
                    }
                }
                call.push_str(" }");
            }
        }
    }
    call.push(']');
    let names = dynamic_prop_names(ctx, node);
    let svg = node.namespace == vize_atelier_core::Namespace::Svg;
    if names.is_some() || svg {
        call.push_str(", ");
        call.push_str(names.as_deref().unwrap_or("null"));
    }
    if svg {
        call.push_str(", true");
    }
    call.push(')');
    call
}

/// The `staticKeys` argument (upstream's `genDynamicPropNames`): statically
/// named props with a value that can change, which hydration must write even
/// though the server rendered them. `.prop` keys and `class`/`style` are
/// always patched, and a constant value never differs.
fn dynamic_prop_names(ctx: &GenerateContext, node: &SetMergedPropsIRNode<'_>) -> Option<String> {
    let mut names = String::default();
    for source in node.sources.iter() {
        let MergedPropsSource::Group(props) = source else {
            continue;
        };
        for prop in props.iter() {
            let key = prop.key.content;
            if !prop.key.is_static
                || key.starts_with('.')
                || matches!(key, "class" | "style")
                || !prop.values.iter().any(|value| !constant_value(ctx, value))
            {
                continue;
            }
            names.push_str(if names.is_empty() { "[\"" } else { ",\"" });
            names.push_str(&escape_js_string_literal(key));
            names.push('"');
        }
    }
    (!names.is_empty()).then(|| {
        names.push(']');
        names
    })
}

/// A literal, or a binding that never changes (upstream's `isConstantBinding`).
fn constant_value(
    ctx: &GenerateContext,
    value: &vize_atelier_core::SimpleExpressionNode<'_>,
) -> bool {
    use vize_atelier_core::options::BindingType;
    let content = value.content.trim();
    value.is_static
        || super::super::expression::is_literal_expression(content)
        || ctx
            .binding_metadata
            .and_then(|bindings| bindings.bindings.get(content))
            .is_some_and(|binding| {
                matches!(binding, BindingType::SetupConst | BindingType::LiteralConst)
            })
}

/// Upstream's `isSimpleIdentifier`: such keys stay bare, others are quoted.
fn simple_identifier(key: &str) -> bool {
    let bytes = key.as_bytes();
    bytes
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'$'))
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$'))
}

/// The trailing arguments that tell `setDynamicProps` an element is SVG, as
/// Vue's compiler emits them (`staticKeys`, then `isSVG`; 3.6.0-rc.10+).
/// An earlier runtime reads the null as `isSVG` and behaves as before.
pub(crate) fn svg_argument(namespace: vize_atelier_core::Namespace) -> &'static str {
    if namespace == vize_atelier_core::Namespace::Svg {
        ", null, true"
    } else {
        ""
    }
}
