//! `setDynamicProps` over upstream's ordered sources: for an element with a
//! `v-bind` object, every prop authored around the object lands in a literal
//! group so the runtime merges them in authored order (later sources win,
//! `class`/`style` concatenate), exactly as `@vue/compiler-vapor` emits it.

use crate::ir::{MergedPropsSource, SetMergedPropsIRNode};
use vize_carton::{String, ToCompactString, cstr};

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
    let mut sources: std::vec::Vec<String> = std::vec::Vec::with_capacity(node.sources.len());
    for source in node.sources.iter() {
        match source {
            MergedPropsSource::Object(value) => sources.push(ctx.resolve_expression_node(value)),
            MergedPropsSource::Group(props) => {
                let mut group = String::from("{ ");
                for (index, prop) in props.iter().enumerate() {
                    if index > 0 {
                        group.push_str(", ");
                    }
                    let key = prop.key.content;
                    if simple_identifier(key) {
                        group.push_str(key);
                    } else {
                        group.push_str(&cstr!("\"{}\"", escape_js_string_literal(key)));
                    }
                    group.push_str(": ");
                    let values: std::vec::Vec<String> = (prop.values.iter())
                        .map(|value| {
                            if value.is_static {
                                cstr!("\"{}\"", escape_js_string_literal(value.content))
                            } else {
                                ctx.resolve_expression_node(value)
                            }
                        })
                        .collect();
                    match values.as_slice() {
                        [value] => group.push_str(value),
                        values => group.push_str(&cstr!("[{}]", values.join(", "))),
                    }
                }
                group.push_str(" }");
                sources.push(group);
            }
        }
    }
    cstr!(
        "_setDynamicProps(n{}, [{}])",
        node.element,
        sources.join(", ")
    )
    .to_compact_string()
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
