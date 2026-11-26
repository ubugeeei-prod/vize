//! One v-for item attribute or directive, preserving its existing emission order.

use crate::PropNode;
use crate::codegen::{context::CodegenContext, helpers::escape_js_string};

/// Generate a single prop (attribute or directive)
pub(in crate::codegen::v_for) fn generate_single_prop(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_merge: crate::codegen::props::StaticMerge<'_>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let ref_value = if attr.name == "ref" && ctx.options.inline {
                attr.value.as_ref()
            } else {
                None
            };
            let ref_binding_type = ref_value.and_then(|v| {
                ctx.options
                    .binding_metadata
                    .as_ref()
                    .and_then(|m| m.bindings.get(v.content).copied())
            });
            let should_ref_runtime_binding = matches!(
                ref_binding_type,
                Some(
                    crate::options::BindingType::SetupLet
                        | crate::options::BindingType::SetupRef
                        | crate::options::BindingType::SetupMaybeRef
                )
            );
            let needs_ref_for = attr.name == "ref" && ctx.in_v_for;

            if let (true, Some(ref_value)) = (should_ref_runtime_binding, ref_value) {
                let ref_name = &ref_value.content;
                if needs_ref_for {
                    ctx.push("ref_for: true, ");
                }
                ctx.push("ref_key: \"");
                ctx.push(ref_name);
                ctx.push("\", ref: ");
                ctx.push(ref_name);
                return;
            }

            if needs_ref_for {
                ctx.push("ref_for: true, ");
            }
            let needs_quotes = !crate::codegen::helpers::is_valid_js_identifier(attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = &attr.value {
                if should_ref_runtime_binding {
                    ctx.push(value.content);
                } else {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(value.content));
                    ctx.push("\"");
                }
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            crate::codegen::props::generate_directive_prop_with_static(ctx, dir, static_merge);
        }
    }
}
