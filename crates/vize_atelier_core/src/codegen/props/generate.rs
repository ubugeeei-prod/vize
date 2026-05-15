//! Main props generation logic.

use crate::ast::{ExpressionNode, PropNode, RuntimeHelper};
use vize_relief::options::BindingType;

use super::{
    super::{
        context::CodegenContext,
        helpers::{escape_js_string, is_valid_js_identifier},
    },
    directives::{generate_directive_prop_with_static, is_supported_directive},
    events::{generate_merged_event_handlers, get_von_event_key},
    generate_vbind_object_exp, generate_von_object_exp,
    scan::PropsScan,
};
use vize_carton::{FxHashSet, String};

/// Generate props object
pub fn generate_props(ctx: &mut CodegenContext, props: &[PropNode<'_>]) {
    // Clone scope_id to avoid borrow checker issues.
    // For component/slot elements, skip_scope_id suppresses the attribute.
    let scope_id = if ctx.skip_scope_id {
        None
    } else {
        ctx.options.scope_id.clone()
    };

    // If no props but we have scope_id, generate object with just scope_id
    if props.is_empty() {
        if let Some(ref sid) = scope_id {
            ctx.push("{ \"");
            ctx.push(sid);
            ctx.push("\": \"\" }");
        } else {
            ctx.push("null");
        }
        return;
    }

    if try_generate_static_attrs(ctx, props, scope_id.as_deref()) {
        return;
    }

    let scan = PropsScan::new(ctx, props, ctx.skip_is_prop);

    // Handle cases with object spreads (v-bind="obj" or v-on="obj")
    if scan.has_vbind_obj || scan.has_von_obj {
        if scan.has_other || (scan.has_vbind_obj && scan.has_von_obj) {
            // Multiple spreads or spread with other props: _mergeProps(...)
            ctx.use_helper(RuntimeHelper::MergeProps);
            ctx.push(ctx.helper(RuntimeHelper::MergeProps));
            ctx.push("(");

            let mut first_merge_arg = true;

            // Add v-bind object spread
            if scan.has_vbind_obj {
                generate_vbind_object_exp(ctx, props);
                first_merge_arg = false;
            }

            // Add v-on object spread (wrapped with toHandlers)
            if scan.has_von_obj {
                if !first_merge_arg {
                    ctx.push(", ");
                }
                generate_von_object_exp(ctx, props);
                first_merge_arg = false;
            }

            // Add other props as object (includes scope_id)
            // Inside mergeProps, skip normalizeClass/normalizeStyle - mergeProps handles it
            if scan.has_other {
                if !first_merge_arg {
                    ctx.push(", ");
                }
                generate_props_object_inner(ctx, props, true, true, &scan);
            } else if let Some(ref sid) = scope_id {
                // No other props but we have scope_id, add it as separate object
                if !first_merge_arg {
                    ctx.push(", ");
                }
                ctx.push("{ \"");
                ctx.push(sid);
                ctx.push("\": \"\" }");
            }

            ctx.push(")");
        } else if scan.has_vbind_obj {
            // v-bind="attrs" alone
            // If we have scope_id, we need to merge it with the bound object
            if let Some(ref sid) = scope_id {
                // _mergeProps(_normalizeProps(_guardReactiveProps(obj)), { "data-v-xxx": "" })
                ctx.use_helper(RuntimeHelper::MergeProps);
                ctx.use_helper(RuntimeHelper::NormalizeProps);
                ctx.use_helper(RuntimeHelper::GuardReactiveProps);
                ctx.push(ctx.helper(RuntimeHelper::MergeProps));
                ctx.push("(");
                ctx.push(ctx.helper(RuntimeHelper::NormalizeProps));
                ctx.push("(");
                ctx.push(ctx.helper(RuntimeHelper::GuardReactiveProps));
                ctx.push("(");
                generate_vbind_object_exp(ctx, props);
                ctx.push(")), { \"");
                ctx.push(sid);
                ctx.push("\": \"\" })");
            } else {
                // _normalizeProps(_guardReactiveProps(_ctx.attrs))
                ctx.use_helper(RuntimeHelper::NormalizeProps);
                ctx.use_helper(RuntimeHelper::GuardReactiveProps);
                ctx.push(ctx.helper(RuntimeHelper::NormalizeProps));
                ctx.push("(");
                ctx.push(ctx.helper(RuntimeHelper::GuardReactiveProps));
                ctx.push("(");
                generate_vbind_object_exp(ctx, props);
                ctx.push("))");
            }
        } else {
            // v-on="handlers" alone
            // If we have scope_id, we need to merge it with the handlers
            if let Some(ref sid) = scope_id {
                // _mergeProps(_toHandlers(handlers, true), { "data-v-xxx": "" })
                ctx.use_helper(RuntimeHelper::MergeProps);
                ctx.push(ctx.helper(RuntimeHelper::MergeProps));
                ctx.push("(");
                generate_von_object_exp(ctx, props);
                ctx.push(", { \"");
                ctx.push(sid);
                ctx.push("\": \"\" })");
            } else {
                // _toHandlers(_ctx.handlers)
                generate_von_object_exp(ctx, props);
            }
        }
        return;
    }

    // Check if we need normalizeProps wrapper
    // - dynamic v-model argument
    // - dynamic v-bind key (:[attr])
    // - dynamic v-on key (@[event])
    let needs_normalize = scan.needs_normalize();
    if needs_normalize {
        ctx.use_helper(RuntimeHelper::NormalizeProps);
        ctx.push(ctx.helper(RuntimeHelper::NormalizeProps));
        ctx.push("(");
    }

    generate_props_object(ctx, props, false, &scan);

    // Close normalizeProps wrapper if needed
    if needs_normalize {
        ctx.push(")");
    }
}

fn try_generate_static_attrs(
    ctx: &mut CodegenContext,
    props: &[PropNode<'_>],
    scope_id: Option<&str>,
) -> bool {
    if ctx.skip_is_prop || ctx.options.inline {
        return false;
    }
    if !props
        .iter()
        .all(|prop| matches!(prop, PropNode::Attribute(_)))
    {
        return false;
    }

    let multiline = props.len() + usize::from(scope_id.is_some()) > 1;
    if multiline {
        ctx.push("{");
        ctx.indent();
    } else {
        ctx.push("{ ");
    }

    let mut first = true;
    for prop in props {
        let PropNode::Attribute(attr) = prop else {
            unreachable!("checked above");
        };

        if !first {
            ctx.push(",");
        }
        if multiline {
            ctx.newline();
        } else if !first {
            ctx.push(" ");
        }
        first = false;

        let needs_quotes = !is_valid_js_identifier(&attr.name);
        if needs_quotes {
            ctx.push("\"");
        }
        ctx.push(&attr.name);
        if needs_quotes {
            ctx.push("\"");
        }
        ctx.push(": ");
        if let Some(value) = &attr.value {
            ctx.push("\"");
            ctx.push(&escape_js_string(&value.content));
            ctx.push("\"");
        } else {
            ctx.push("\"\"");
        }
    }

    if let Some(sid) = scope_id {
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

    true
}

/// Generate props as a regular object { key: value, ... }
fn generate_props_object(
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
fn generate_props_object_inner(
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
    // For component/slot elements, skip_scope_id suppresses the attribute.
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
        // Skip v-slot directive (handled separately in slots codegen)
        if let PropNode::Directive(dir) = prop {
            if dir.name == "slot" {
                continue;
            }
        }

        // Skip `is` prop when generating for dynamic components
        if ctx.skip_is_prop {
            match prop {
                PropNode::Attribute(attr) if attr.name == "is" => continue,
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "is") =>
                {
                    continue
                }
                _ => {}
            }
        }

        match prop {
            PropNode::Attribute(attr) => {
                // Skip static class/style if merging with dynamic
                if skip_static_class && attr.name == "class" {
                    continue;
                }
                if skip_static_style && attr.name == "style" {
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
                let ref_binding_type = if attr.name == "ref" && ctx.options.inline {
                    attr.value.as_ref().and_then(|v| {
                        ctx.options
                            .binding_metadata
                            .as_ref()
                            .and_then(|m| m.bindings.get(v.content.as_str()).copied())
                    })
                } else {
                    None
                };
                let needs_ref_key = matches!(
                    ref_binding_type,
                    Some(
                        BindingType::SetupLet | BindingType::SetupRef | BindingType::SetupMaybeRef
                    )
                );

                if needs_ref_key {
                    // Emit ref_key + ref pair for setup-let/ref/maybe-ref bindings.
                    // Vue's runtime setRef() needs ref_key to write to instance.refs,
                    // which is essential for useTemplateRef to receive the element.
                    let ref_name = &attr.value.as_ref().unwrap().content;
                    ctx.push("ref_key: \"");
                    ctx.push(ref_name);
                    ctx.push("\", ref: ");
                    ctx.push(ref_name);
                } else {
                    // Normal attribute output
                    let needs_quotes = !is_valid_js_identifier(&attr.name);
                    if needs_quotes {
                        ctx.push("\"");
                    }
                    ctx.push(&attr.name);
                    if needs_quotes {
                        ctx.push("\"");
                    }
                    ctx.push(": ");
                    if let Some(value) = &attr.value {
                        // In inline mode, ref="refName" should reference the setup variable
                        // instead of being a string literal, if refName is a known binding
                        if ref_binding_type.is_some() {
                            ctx.push(&value.content);
                        } else {
                            ctx.push("\"");
                            ctx.push(&escape_js_string(&value.content));
                            ctx.push("\"");
                        }
                    } else {
                        ctx.push("\"\"");
                    }
                }
            }
            PropNode::Directive(dir) => {
                // Skip v-bind/v-on object spreads (handled separately by generate_props)
                if skip_object_spreads
                    && dir.arg.is_none()
                    && (dir.name == "bind" || dir.name == "on")
                {
                    continue;
                }
                // Only add comma if directive produces valid output
                if is_supported_directive(dir) {
                    // Check for duplicate v-on events that should be merged into arrays
                    if dir.name == "on" {
                        if let Some(event_key) = get_von_event_key(dir) {
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
                        scan.static_class,
                        scan.static_style,
                    );
                }
            }
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
