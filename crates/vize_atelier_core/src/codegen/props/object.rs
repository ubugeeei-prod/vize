//! Props object emission (`{ key: value, ... }` and mergeProps segments).
//!
//! Split out of `generate.rs` to keep each props-codegen file under the
//! source-length guard so render-semantic (Rendu) reroutes have room to land.

use crate::rendu::RenduOp;
use crate::{ExpressionNode, PropNode};
use vize_relief::options::BindingType;

use super::{
    super::{
        context::CodegenContext,
        helpers::{escape_js_string, is_valid_js_identifier},
    },
    directives::{generate_directive_prop_with_static, is_supported_directive},
    events::{generate_merged_event_handlers, get_von_event_key},
    scan::PropsScan,
};
use vize_carton::{FxHashSet, String};

/// Generate props as a regular object { key: value, ... }
pub(super) fn generate_props_object(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    skip_object_spreads: bool,
    scan: &PropsScan<'_>,
) {
    generate_props_object_inner(ctx, props, skip_object_spreads, false, scan);
}

/// Generate the props object with optional class/style normalization skipping.
/// `inside_merge_props`: when true, skip normalizeClass/normalizeStyle wrappers
/// because mergeProps handles normalization internally.
pub(super) fn generate_props_object_inner(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    skip_object_spreads: bool,
    inside_merge_props: bool,
    scan: &PropsScan<'_>,
) {
    // When inside mergeProps, skip normalizeClass/normalizeStyle wrappers
    let prev_skip = ctx.skip_normalize;
    if inside_merge_props {
        ctx.skip_normalize = true;
    }

    // Clone scope_id to avoid borrow checker issues.
    // skip_scope_id suppresses duplicate scope attrs for synthetic prop objects.
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };

    // Skip static class/style if we have dynamic version (will merge them)
    let skip_static_class = scan.skip_static_class();
    let skip_static_style = scan.skip_static_style();
    let multiline = scan.multiline(scope_id.is_some());

    if multiline {
        ctx.push("{");
        ctx.indent();
    } else {
        ctx.push("{ ");
    }

    let mut first = true;
    // Track which event names have already been output (for array merging)
    let mut emitted_events: Option<FxHashSet<String>> = None;

    for prop in props {
        let op = RenduOp::from_prop(prop);
        // Skip v-slot directive (handled separately in slots codegen)
        if matches!(op, RenduOp::Directive { name: "slot", .. }) {
            continue;
        }

        // Skip `is` prop when generating for dynamic components
        if ctx.skip_is_prop
            && match op {
                RenduOp::Attribute { name: "is", .. } => true,
                RenduOp::Directive {
                    name: "bind",
                    arg: Some(arg),
                    ..
                } => matches!(arg.node(), Some(ExpressionNode::Simple(exp)) if exp.content == "is"),
                _ => false,
            }
        {
            continue;
        }

        match op {
            RenduOp::Attribute {
                name,
                name_span,
                value: attr_value,
                value_span,
                ..
            } => {
                let PropNode::Attribute(attr) = prop else {
                    unreachable!("Rendu attribute must borrow an attribute prop");
                };
                // Skip static class/style if merging with dynamic
                if skip_static_class && name == "class" {
                    continue;
                }
                if skip_static_style && name == "style" {
                    continue;
                }
                if !first {
                    ctx.push(",");
                }
                if multiline {
                    ctx.newline();
                } else if !first {
                    ctx.push(" ");
                }
                first = false;

                // Check if this is a ref attribute that needs ref_key generation
                let ref_value = if attr.name == "ref" && ctx.options.inline {
                    attr.value.as_ref()
                } else {
                    None
                };
                let ref_binding_type = if let Some(value) = ref_value {
                    ctx.options
                        .binding_metadata
                        .as_ref()
                        .and_then(|m| m.bindings.get(value.content.as_str()).copied())
                } else {
                    None
                };
                let should_ref_runtime_binding = matches!(
                    ref_binding_type,
                    Some(
                        BindingType::SetupLet | BindingType::SetupRef | BindingType::SetupMaybeRef
                    )
                );
                let needs_ref_for = attr.name == "ref" && ctx.in_v_for;

                if let (true, Some(ref_value)) = (should_ref_runtime_binding, ref_value) {
                    // Emit ref_key + ref pair for setup-let/ref/maybe-ref bindings.
                    // Vue's runtime setRef() needs ref_key to write to instance.refs,
                    // which is essential for useTemplateRef to receive the element.
                    let ref_name = &ref_value.content;
                    if needs_ref_for {
                        ctx.push("ref_for: true, ");
                    }
                    ctx.push("ref_key: \"");
                    ctx.push(ref_name);
                    ctx.push("\", ref: ");
                    ctx.push(ref_name);
                } else {
                    if needs_ref_for {
                        ctx.push("ref_for: true, ");
                    }
                    // Normal attribute output, lowered through the Rendu op
                    // (#1756): name, value, and their source spans are read from
                    // `RenduOp::Attribute`, not the Relief node.
                    let needs_quotes = !is_valid_js_identifier(name);
                    if needs_quotes {
                        ctx.push("\"");
                    }
                    // Anchor the prop key back to the attribute name (no-op
                    // without `source_map`); records the symbol for the v3
                    // `names` array.
                    ctx.record_mapping_named(&name_span.start, name);
                    ctx.push(name);
                    if needs_quotes {
                        ctx.push("\"");
                    }
                    ctx.push(": ");
                    if let Some(attr_value) = attr_value {
                        // In inline mode, ref="refName" references a mutable/setup-ref
                        // binding. Other bindings (notably props) are string refs.
                        if should_ref_runtime_binding {
                            ctx.push(attr_value);
                        } else {
                            ctx.push("\"");
                            if let Some(span) = value_span {
                                ctx.record_mapping(&span.start);
                            }
                            ctx.push(&escape_js_string(attr_value));
                            ctx.push("\"");
                        }
                    } else {
                        ctx.push("\"\"");
                    }
                }
            }
            RenduOp::Directive { name, arg, .. } => {
                let PropNode::Directive(dir) = prop else {
                    unreachable!("Rendu directive must borrow a directive prop");
                };
                // Skip v-bind/v-on object spreads (handled separately by generate_props)
                if skip_object_spreads && arg.is_none() && matches!(name, "bind" | "on") {
                    continue;
                }
                // Only add comma if directive produces valid output
                if is_supported_directive(dir) {
                    // Check for duplicate v-on events that should be merged into arrays
                    if name == "on"
                        && let Some(event_key) = get_von_event_key(dir, ctx.props_is_plain_element)
                    {
                        let count = scan.event_counts.count(&event_key);
                        if count > 1 {
                            let emitted_events =
                                emitted_events.get_or_insert_with(FxHashSet::default);
                            if emitted_events.contains(&event_key) {
                                // Skip: already emitted as part of array
                                continue;
                            }
                            // First occurrence: emit as array with all handlers for this event
                            emitted_events.insert(event_key.clone());
                            if !first {
                                ctx.push(",");
                            }
                            if multiline {
                                ctx.newline();
                            } else if !first {
                                ctx.push(" ");
                            }
                            first = false;
                            generate_merged_event_handlers(
                                ctx,
                                props,
                                &event_key,
                                scan.static_class,
                                scan.static_style,
                            );
                            continue;
                        }
                    }

                    if !first {
                        ctx.push(",");
                    }
                    if multiline {
                        ctx.newline();
                    } else if !first {
                        ctx.push(" ");
                    }
                    first = false;
                    generate_directive_prop_with_static(
                        ctx,
                        dir,
                        super::directives::StaticMerge {
                            class: scan.static_class,
                            class_before: scan.static_class_before_dynamic,
                            style: scan.static_style,
                            style_before: scan.static_style_before_dynamic,
                        },
                    );
                }
            }
            _ => unreachable!("element props lower to attribute or directive Rendu ops"),
        }
    }

    // Add scope_id attribute for scoped CSS
    if let Some(ref sid) = scope_id {
        if !first {
            ctx.push(",");
        }
        if multiline {
            ctx.newline();
        } else if !first {
            ctx.push(" ");
        }
        ctx.push("\"");
        ctx.push(sid);
        ctx.push("\": \"\"");
    }

    if multiline {
        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    } else {
        ctx.push(" }");
    }

    // Restore skip_normalize flag
    ctx.skip_normalize = prev_skip;
}
