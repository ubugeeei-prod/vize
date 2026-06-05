//! Main props generation logic.

use crate::ast::{ExpressionNode, PropNode, RuntimeHelper};
use vize_relief::options::BindingType;

use super::{
    super::expression::generate_expression,
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
    // skip_scope_id suppresses duplicate scope attrs for synthetic prop objects.
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
            // Multiple spreads or spread with other props: _mergeProps(...).
            // Vue walks props in source order, accumulating non-spread props into
            // object literals and flushing them around each spread, preserving
            // the original ordering (transforms/transformElement.ts buildProps).
            ctx.use_helper(RuntimeHelper::MergeProps);
            ctx.push(ctx.helper(RuntimeHelper::MergeProps));
            ctx.push("(");

            let mut first_merge_arg = true;
            let mut seg_start = 0usize;

            // scope_id is emitted once as a trailing object, never per segment.
            let prev_skip_scope_id = ctx.skip_scope_id;
            ctx.skip_scope_id = true;

            let flush_object =
                |ctx: &mut CodegenContext, start: usize, end: usize, first: &mut bool| {
                    // Does this range hold any renderable non-spread prop?
                    let segment = &props[start..end];
                    let has_renderable = segment.iter().any(|p| match p {
                        PropNode::Attribute(attr) => !(ctx.skip_is_prop && attr.name == "is"),
                        PropNode::Directive(dir) => {
                            // A `:is`/`v-bind:is` directive on a dynamic component is consumed
                            // as the component tag and skipped during generation (mirrors the
                            // skip_is_prop branch in generate_props_object_inner). It must not
                            // count as renderable, or an empty `{}` is flushed into mergeProps.
                            let is_skipped_is = ctx.skip_is_prop
                                && dir.name == "bind"
                                && matches!(
                                    &dir.arg,
                                    Some(ExpressionNode::Simple(exp)) if exp.content == "is"
                                );
                            !is_skipped_is
                                && !(dir.arg.is_none() && (dir.name == "bind" || dir.name == "on"))
                                && is_supported_directive(dir)
                                && dir.name != "slot"
                        }
                    });
                    if !has_renderable {
                        return;
                    }
                    if !*first {
                        ctx.push(", ");
                    }
                    *first = false;
                    let seg_scan = PropsScan::new(ctx, segment, ctx.skip_is_prop);
                    generate_props_object_inner(ctx, segment, true, true, &seg_scan);
                };

            for (index, prop) in props.iter().enumerate() {
                let PropNode::Directive(dir) = prop else {
                    continue;
                };
                let is_vbind_spread = dir.name == "bind" && dir.arg.is_none();
                let is_von_spread = dir.name == "on" && dir.arg.is_none();
                if !is_vbind_spread && !is_von_spread {
                    continue;
                }

                // Flush accumulated non-spread props before this spread.
                flush_object(ctx, seg_start, index, &mut first_merge_arg);
                seg_start = index + 1;

                if !first_merge_arg {
                    ctx.push(", ");
                }
                first_merge_arg = false;
                if is_vbind_spread {
                    if let Some(exp) = &dir.exp {
                        generate_expression(ctx, exp);
                    }
                } else {
                    // v-on spread wrapped with _toHandlers(..., true)
                    ctx.use_helper(RuntimeHelper::ToHandlers);
                    ctx.push(ctx.helper(RuntimeHelper::ToHandlers));
                    ctx.push("(");
                    if let Some(exp) = &dir.exp {
                        generate_expression(ctx, exp);
                    }
                    ctx.push(", true)");
                }
            }

            // Flush any trailing non-spread props.
            flush_object(ctx, seg_start, props.len(), &mut first_merge_arg);

            ctx.skip_scope_id = prev_skip_scope_id;

            // scope_id (if present) is appended as a trailing object.
            if let Some(ref sid) = scope_id {
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
    if ctx.in_v_for
        && props
            .iter()
            .any(|prop| matches!(prop, PropNode::Attribute(attr) if attr.name == "ref"))
    {
        return false;
    }
    if !props
        .iter()
        .all(|prop| matches!(prop, PropNode::Attribute(_)))
    {
        return false;
    }

    // Vue's behavior on duplicate attributes is to keep the first
    // occurrence and ignore later repeats — the parser already records
    // a recoverable diagnostic for the duplicate but leaves both nodes
    // in the AST so linters can flag them. Dedupe here so the rendered
    // props object doesn't emit `{ id: "a", id: "b" }`. (#958)
    let mut seen: FxHashSet<String> = FxHashSet::default();
    let mut unique_props: Vec<&PropNode<'_>> = Vec::with_capacity(props.len());
    for prop in props {
        if let PropNode::Attribute(attr) = prop {
            if seen.contains(attr.name.as_str()) {
                continue;
            }
            seen.insert(attr.name.clone());
        }
        unique_props.push(prop);
    }

    let multiline = unique_props.len() + usize::from(scope_id.is_some()) > 1;
    if multiline {
        ctx.push("{");
        ctx.indent();
    } else {
        ctx.push("{ ");
    }

    let mut first = true;
    for prop in unique_props {
        let PropNode::Attribute(attr) = prop else {
            // Panic path by invariant: the preflight `all(Attribute)` check above
            // has already rejected directive props. Reaching this arm would mean
            // `props` was mutated while iterating, which is impossible through the
            // shared slice used by codegen.
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
        // Skip v-slot directive (handled separately in slots codegen)
        if let PropNode::Directive(dir) = prop
            && dir.name == "slot"
        {
            continue;
        }

        // Skip `is` prop when generating for dynamic components
        if ctx.skip_is_prop {
            match prop {
                PropNode::Attribute(attr) if attr.name == "is" => continue,
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(&dir.arg, Some(ExpressionNode::Simple(exp)) if exp.content == "is") =>
                {
                    continue;
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
                        // In inline mode, ref="refName" should reference a mutable/setup-ref
                        // binding. Other bindings (notably props) are still string refs.
                        if should_ref_runtime_binding {
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
                    if dir.name == "on"
                        && let Some(event_key) = get_von_event_key(dir)
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
