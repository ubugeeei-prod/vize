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
    call.push_str(svg_argument(node.namespace));
    call.push(')');
    call
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
