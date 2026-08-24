//! Element transformation functions.

use vize_carton::{Box, String, Vec, capitalize, is_builtin_directive, is_native_tag};

use crate::errors::ErrorCode;
use crate::steps::expression::process_inline_handler;
use crate::steps::v_model::{
    generate_model_assignment_handler, model_update_listener_name,
    supports_plain_element_model_argument,
};
use crate::steps::v_slot::validate_v_slot_usage;
use crate::{
    ConstantType, DirectiveNode, ElementNode, ElementType, ExpressionNode, InterpolationNode,
    PropNode, RuntimeHelper, SimpleExpressionNode, SourceLocation,
};

use super::{ExitFns, TransformContext};

mod directive_expressions;

use directive_expressions::process_directive_expressions;

fn is_dynamic_component(el: &ElementNode<'_>) -> bool {
    el.tag == "component" || (el.tag == "Component" && has_is_attribute(el))
}

/// Transform element node
pub fn transform_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut Box<'a, ElementNode<'a>>,
) -> Option<ExitFns<'a>> {
    maybe_promote_element_to_component(ctx, el);
    validate_v_slot_usage(ctx, el);

    // Process props and directives
    process_element_props(ctx, el);

    // Determine helpers based on element type
    match el.tag_type {
        ElementType::Element => {
            ctx.helper(RuntimeHelper::CreateElementVNode);
        }
        ElementType::Component => {
            if is_dynamic_component(el) {
                return None;
            }

            // Only add ResolveComponent if component is not in binding metadata
            let is_in_bindings = ctx
                .options
                .binding_metadata
                .as_ref()
                .map(|m| m.bindings.contains_key(el.tag))
                .unwrap_or(false);
            if !is_in_bindings {
                ctx.helper(RuntimeHelper::ResolveComponent);
            }
            // Defer add_component to exit phase so inner components resolve before outer ones
            let tag = el.tag;
            let mut exits = ExitFns::new();
            exits.push(std::boxed::Box::new(move |ctx| {
                ctx.add_component(tag);
            }));
            return Some(exits);
        }
        ElementType::Slot => {
            ctx.helper(RuntimeHelper::RenderSlot);
        }
        ElementType::Template => {
            ctx.helper(RuntimeHelper::Fragment);
        }
    }

    None
}

fn maybe_promote_element_to_component(
    ctx: &TransformContext<'_>,
    el: &mut Box<'_, ElementNode<'_>>,
) {
    if el.tag_type != ElementType::Element || ctx.is_jsx_custom_element(el) {
        return;
    }

    if is_native_tag(el.tag) {
        return;
    }

    if is_registered_component(ctx, el.tag) {
        el.tag_type = ElementType::Component;
        return;
    }

    if ctx.custom_elements.matches(el.tag) {
        return;
    }

    if el.tag == "component"
        || el.tag.chars().next().is_some_and(|c| c.is_uppercase())
        || el.tag.contains('-')
        || has_is_attribute(el)
    {
        el.tag_type = ElementType::Component;
    }
}

fn has_is_attribute(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| match prop {
        PropNode::Directive(dir) => {
            dir.name == "is"
                || (dir.name == "bind"
                    && matches!(
                        &dir.arg,
                        Some(ExpressionNode::Simple(arg)) if arg.content == "is"
                    ))
        }
        PropNode::Attribute(attr) => attr.name == "is",
    })
}

fn is_registered_component(ctx: &TransformContext<'_>, tag: &str) -> bool {
    if ctx.is_component_registered(tag) {
        return true;
    }

    if tag.contains('-') || tag.contains('_') {
        let camel = vize_carton::camelize(tag);
        if ctx.is_component_registered(camel.as_str()) {
            return true;
        }

        let pascal = capitalize(&camel);
        return ctx.is_component_registered(pascal.as_str());
    }

    let pascal = capitalize(tag);
    ctx.is_component_registered(pascal.as_str())
}

/// Process element properties and directives
fn process_element_props<'a>(ctx: &mut TransformContext<'a>, el: &mut Box<'a, ElementNode<'a>>) {
    if el.props.is_empty()
        || !el
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(_)))
    {
        return;
    }

    let allocator = ctx.allocator;
    let is_component = el.tag_type == ElementType::Component;

    // Vue 2 v-on event-modifier sugar (`@click.native`, numeric keycodes such
    // as `@keyup.13`). Legacy-only and dialect-gated: a no-op for the default
    // Vue 3 dialect (and every other legacy line), so the directive modifiers
    // stay byte-identical there. Runs before any modifier classification so
    // `.native` is stripped and numeric keycodes are rewritten to key names.
    #[cfg(feature = "legacy")]
    if ctx.supports_v2_event_sugar() {
        for prop in el.props.iter_mut() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "on"
            {
                crate::steps::legacy::desugar_v2_v_on_modifiers(dir);
            }
        }
    }

    // Process directive expressions with _ctx prefix if needed
    if ctx.options.prefix_identifiers || ctx.options.is_ts {
        process_directive_expressions(ctx, el);
    }

    // Collect indices of v-model directives to process (skip in vapor mode)
    let mut model_indices: std::vec::Vec<usize> = std::vec::Vec::new();
    for (i, prop) in el.props.iter().enumerate() {
        if let PropNode::Directive(dir) = prop {
            match dir.name {
                "model" if !ctx.options.vapor => {
                    model_indices.push(i);
                }
                "slot" => {
                    ctx.helper(RuntimeHelper::RenderSlot);
                }
                // v-show is a built-in directive - it uses vShow helper directly
                // No need to add to ctx.directives (which would use resolveDirective)
                "show" => {}
                // Handle custom directives - register them for resolveDirective
                _ if !is_builtin_directive(dir.name) => {
                    ctx.helper(RuntimeHelper::WithDirectives);
                    ctx.helper(RuntimeHelper::ResolveDirective);
                    ctx.add_directive(dir.name);
                }
                _ => {}
            }
        }
    }

    // Collect all v-model data first, then modify props
    struct VModelData {
        idx: usize,
        value_exp: String,
        raw_value_exp: String,
        prop_name: String,
        event_name: String,
        handler: String,
        dir_loc: SourceLocation,
        modifiers_obj: Option<String>,
        modifiers_key: Option<String>,
        is_dynamic: bool,
    }

    let mut vmodel_data: std::vec::Vec<VModelData> = std::vec::Vec::new();
    let mut invalid_model_indices: std::vec::Vec<usize> = std::vec::Vec::new();

    for &idx in model_indices.iter() {
        if let Some(PropNode::Directive(dir)) = el.props.get(idx) {
            // Get value expression
            let (value_exp, raw_value_exp) = match &dir.exp {
                Some(ExpressionNode::Simple(s)) if !s.content.trim().is_empty() => (
                    String::new(s.content),
                    String::new(s.loc.span.slice(ctx.source)),
                ),
                Some(ExpressionNode::Compound(c))
                    if !c.loc.span.slice(ctx.source).trim().is_empty() =>
                {
                    let text = String::new(c.loc.span.slice(ctx.source));
                    (text.clone(), text)
                }
                _ => {
                    ctx.on_error(ErrorCode::VModelNoExpression, Some(dir.loc.clone()));
                    invalid_model_indices.push(idx);
                    continue;
                }
            };

            let value_exp = value_exp.trim();
            if ctx.is_in_scope(value_exp) {
                ctx.on_error(ErrorCode::VModelOnScope, Some(dir.loc.clone()));
                invalid_model_indices.push(idx);
                continue;
            }

            if !is_component
                && !supports_plain_element_model_argument(
                    ctx.jsx_compat.allow_static_v_model_arg_on_element,
                    dir.arg.as_ref(),
                )
            {
                ctx.on_error(ErrorCode::VModelArgOnElement, Some(dir.loc.clone()));
                invalid_model_indices.push(idx);
                continue;
            }

            let value_exp = String::new(value_exp);

            // Check if arg is dynamic
            let is_dynamic = dir.arg.as_ref().is_some_and(|arg| match arg {
                ExpressionNode::Simple(exp) => !exp.is_static,
                ExpressionNode::Compound(_) => true,
            });

            // Get prop name (default: modelValue for components, value for inputs)
            let prop_name = dir
                .arg
                .as_ref()
                .map(|arg| match arg {
                    ExpressionNode::Simple(exp) => String::new(exp.content),
                    ExpressionNode::Compound(exp) => String::new(exp.loc.span.slice(ctx.source)),
                })
                .unwrap_or_else(|| {
                    if is_component {
                        String::new("modelValue")
                    } else {
                        String::new("value")
                    }
                });

            let named_model = is_component || dir.arg.is_some();
            let event_name = model_update_listener_name(named_model.then_some(prop_name.as_str()));

            // Build handler expression
            let handler = if is_component {
                generate_model_assignment_handler(ctx, raw_value_exp.as_str(), "$event")
            } else {
                // For native elements, check modifiers
                let has_number = dir.modifiers.iter().any(|m| m.content == "number");
                let has_trim = dir.modifiers.iter().any(|m| m.content == "trim");

                let mut target_value = String::from("$event.target.value");
                if has_trim {
                    target_value.push_str(".trim()");
                }
                if has_number {
                    let mut wrapped = String::with_capacity(target_value.len() + 11);
                    wrapped.push_str("_toNumber(");
                    wrapped.push_str(&target_value);
                    wrapped.push(')');
                    target_value = wrapped;
                }
                generate_model_assignment_handler(
                    ctx,
                    raw_value_exp.as_str(),
                    target_value.as_str(),
                )
            };

            let dir_loc = dir.loc.clone();

            // Collect modifiers info for components
            let (modifiers_obj, modifiers_key) = if is_component && !dir.modifiers.is_empty() {
                let modifiers_content: std::vec::Vec<String> = dir
                    .modifiers
                    .iter()
                    .map(|m| {
                        let mut item = String::with_capacity(m.content.len() + 6);
                        item.push_str(m.content);
                        item.push_str(": true");
                        item
                    })
                    .collect();
                let mut obj = String::from("{ ");
                for (i, item) in modifiers_content.iter().enumerate() {
                    if i > 0 {
                        obj.push_str(", ");
                    }
                    obj.push_str(item);
                }
                obj.push_str(" }");
                let key = if prop_name == "modelValue" {
                    String::new("modelModifiers")
                } else {
                    let mut key = String::with_capacity(prop_name.len() + 9);
                    key.push_str(prop_name.as_str());
                    key.push_str("Modifiers");
                    key
                };
                (Some(obj), Some(key))
            } else {
                (None, None)
            };

            vmodel_data.push(VModelData {
                idx,
                value_exp,
                raw_value_exp,
                prop_name,
                event_name,
                handler,
                dir_loc,
                modifiers_obj,
                modifiers_key,
                is_dynamic,
            });
        }
    }

    if !invalid_model_indices.is_empty() {
        for data in vmodel_data.iter_mut() {
            let removed_before = invalid_model_indices
                .iter()
                .filter(|&&invalid_idx| invalid_idx < data.idx)
                .count();
            data.idx -= removed_before;
        }
        for &idx in invalid_model_indices.iter().rev() {
            el.props.remove(idx);
        }
    }

    if is_component {
        // Separate static and dynamic v-model data
        let static_vmodel: std::vec::Vec<_> =
            vmodel_data.iter().filter(|d| !d.is_dynamic).collect();
        let dynamic_vmodel: std::vec::Vec<_> =
            vmodel_data.iter().filter(|d| d.is_dynamic).collect();

        // Build in source order for stable cache indexes, then splice in
        // reverse so each v-model remains beside its explicit listeners.
        let mut replacements = std::vec::Vec::with_capacity(static_vmodel.len());
        for data in static_vmodel.iter() {
            let prop_atom = ctx.interner.intern(&data.prop_name);
            let event_atom = ctx.interner.intern(&data.event_name[2..]);
            let mut generated = std::vec::Vec::with_capacity(3);
            let value_prop = PropNode::Directive(Box::new_in(
                DirectiveNode {
                    name: "bind",
                    raw_name: None,
                    arg: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode::new(prop_atom, true, data.dir_loc.clone()),
                        &allocator,
                    ))),
                    exp: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode {
                            content: allocator.alloc_str(&data.value_exp),
                            is_static: false,
                            const_type: ConstantType::NotConstant,
                            loc: data.dir_loc.clone(),
                            js_ast: None,
                            hoisted: None,
                            identifiers: None,
                            is_handler_key: false,
                            is_ref_transformed: true, // Already processed for ref .value
                        },
                        &allocator,
                    ))),
                    modifiers: Vec::new_in(&allocator),
                    for_parse_result: None,
                    shorthand: false,
                    loc: data.dir_loc.clone(),
                },
                &allocator,
            ));
            generated.push(value_prop);

            let handler = allocator.alloc_str(&data.handler);
            let raw_handler_expr = ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(handler, false, data.dir_loc.clone()),
                &allocator,
            ));
            let processed_handler = process_inline_handler(ctx, &raw_handler_expr);

            let event_prop = PropNode::Directive(Box::new_in(
                DirectiveNode {
                    name: "on",
                    raw_name: None,
                    arg: Some(ExpressionNode::Simple(Box::new_in(
                        // Remove "on" prefix
                        SimpleExpressionNode::new(event_atom, true, data.dir_loc.clone()),
                        &allocator,
                    ))),
                    exp: Some(processed_handler),
                    modifiers: Vec::new_in(&allocator),
                    for_parse_result: None,
                    shorthand: false,
                    loc: data.dir_loc.clone(),
                },
                &allocator,
            ));
            generated.push(event_prop);

            if let (Some(modifiers_obj), Some(modifiers_key)) =
                (&data.modifiers_obj, &data.modifiers_key)
            {
                let modifiers_prop = PropNode::Directive(Box::new_in(
                    DirectiveNode {
                        name: "bind",
                        raw_name: None,
                        arg: Some(ExpressionNode::Simple(Box::new_in(
                            SimpleExpressionNode::new(
                                ctx.interner.intern(modifiers_key),
                                true,
                                data.dir_loc.clone(),
                            ),
                            &allocator,
                        ))),
                        exp: Some(ExpressionNode::Simple(Box::new_in(
                            SimpleExpressionNode::new(
                                allocator.alloc_str(modifiers_obj),
                                false,
                                data.dir_loc.clone(),
                            ),
                            &allocator,
                        ))),
                        modifiers: Vec::new_in(&allocator),
                        for_parse_result: None,
                        shorthand: false,
                        loc: SourceLocation::STUB,
                    },
                    &allocator,
                ));
                generated.push(modifiers_prop);
            }

            replacements.push((data.idx, generated));
        }

        for (index, generated) in replacements.into_iter().rev() {
            drop(el.props.splice(index..=index, generated));
        }

        // For dynamic v-model: keep the directive (will be handled in codegen with normalizeProps)
        // Just mark that we have dynamic v-model by adding helper
        if !dynamic_vmodel.is_empty() {
            ctx.helper(RuntimeHelper::NormalizeProps);
        }
    } else {
        // For native elements: process in reverse order to preserve indices during insertion
        for data in vmodel_data.iter().rev() {
            // Keep v-model directive, insert its update handler right after it.
            let handler =
                generate_model_assignment_handler(ctx, data.raw_value_exp.as_str(), "$event");
            let handler = allocator.alloc_str(&handler);
            let raw_handler_expr = ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(handler, false, data.dir_loc.clone()),
                &allocator,
            ));
            let processed_handler = process_inline_handler(ctx, &raw_handler_expr);
            let event_prop = PropNode::Directive(Box::new_in(
                DirectiveNode {
                    name: "on",
                    raw_name: None,
                    arg: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode::new(
                            ctx.interner.intern(&data.event_name[2..]),
                            true,
                            data.dir_loc.clone(),
                        ),
                        &allocator,
                    ))),
                    exp: Some(processed_handler),
                    modifiers: Vec::new_in(&allocator),
                    for_parse_result: None,
                    shorthand: false,
                    loc: data.dir_loc.clone(),
                },
                &allocator,
            ));
            // Insert right after the v-model directive to ensure proper ordering
            el.props.insert(data.idx + 1, event_prop);
        }
    }
}

/// Transform interpolation node
pub fn transform_interpolation<'a>(
    ctx: &mut TransformContext<'a>,
    interp: &mut Box<'a, InterpolationNode<'a>>,
) {
    ctx.helper(RuntimeHelper::ToDisplayString);

    // Process the expression to add _ctx. prefix and/or strip TypeScript if needed
    if ctx.options.prefix_identifiers || ctx.options.is_ts {
        use crate::steps::expression::process_expression;
        let processed = process_expression(ctx, &interp.content, false);
        interp.content = processed;
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests;
