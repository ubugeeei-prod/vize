//! Element transformation functions.

use vize_carton::{Box, String, Vec, capitalize, is_builtin_directive, is_native_tag};

use crate::ast::*;
use crate::errors::ErrorCode;
use crate::transforms::transform_expression::process_inline_handler;

use super::{ExitFns, TransformContext};

fn is_dynamic_component(el: &ElementNode<'_>) -> bool {
    el.tag == "component" || (el.tag == "Component" && has_is_attribute(el))
}

/// Transform element node
pub fn transform_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut Box<'a, ElementNode<'a>>,
) -> Option<ExitFns<'a>> {
    maybe_promote_element_to_component(ctx, el);

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
                .map(|m| m.bindings.contains_key(el.tag.as_str()))
                .unwrap_or(false);
            if !is_in_bindings {
                ctx.helper(RuntimeHelper::ResolveComponent);
            }
            // Defer add_component to exit phase so inner components resolve before outer ones
            let tag = el.tag.clone();
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
    if el.tag_type != ElementType::Element {
        return;
    }

    let looks_like_component = el.tag == "component"
        || el.tag.chars().next().is_some_and(|c| c.is_uppercase())
        || el.tag.contains('-');

    if looks_like_component {
        el.tag_type = ElementType::Component;
        return;
    }

    let has_is = has_is_attribute(el);
    if has_is {
        el.tag_type = ElementType::Component;
        return;
    }

    if is_native_tag(&el.tag) {
        return;
    }

    if is_registered_component(ctx, &el.tag) {
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

/// Process directive expressions with _ctx prefix
fn process_directive_expressions<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut Box<'a, ElementNode<'a>>,
) {
    use crate::transforms::transform_expression::{process_expression, process_inline_handler};

    for prop in el.props.iter_mut() {
        if let PropNode::Directive(dir) = prop {
            match dir.name.as_str() {
                "bind" | "show" | "if" | "else-if" | "for" | "memo" => {
                    // Process value expression
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, false);
                        dir.exp = Some(processed);
                    }
                }
                "on" => {
                    if let Some(exp) = &dir.exp {
                        if dir.arg.is_none() {
                            // v-on="obj" - process as regular expression (object of handlers),
                            // NOT as an inline handler. toHandlers() expects an object, not a function.
                            let processed = process_expression(ctx, exp, false);
                            dir.exp = Some(processed);
                        } else {
                            // v-on:event="handler" - process as inline handler
                            let processed = process_inline_handler(ctx, exp);
                            dir.exp = Some(processed);
                        }
                    }
                }
                "model" => {
                    // Process v-model expression
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, false);
                        dir.exp = Some(processed);
                    }
                }
                _ => {
                    // Custom directives - process value expression
                    if let Some(exp) = &dir.exp {
                        let processed = process_expression(ctx, exp, false);
                        dir.exp = Some(processed);
                    }
                    // Process dynamic argument
                    if let Some(arg) = &dir.arg
                        && let ExpressionNode::Simple(simple_arg) = arg
                        && !simple_arg.is_static
                    {
                        let processed = process_expression(ctx, arg, false);
                        dir.arg = Some(processed);
                    }
                }
            }
        }
    }
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

    // Process directive expressions with _ctx prefix if needed
    if ctx.options.prefix_identifiers || ctx.options.is_ts {
        process_directive_expressions(ctx, el);
    }

    // Collect indices of v-model directives to process (skip in vapor mode)
    let mut model_indices: std::vec::Vec<usize> = std::vec::Vec::new();
    for (i, prop) in el.props.iter().enumerate() {
        if let PropNode::Directive(dir) = prop {
            match dir.name.as_str() {
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
                _ if !is_builtin_directive(&dir.name) => {
                    ctx.helper(RuntimeHelper::WithDirectives);
                    ctx.helper(RuntimeHelper::ResolveDirective);
                    ctx.add_directive(dir.name.clone());
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
                Some(ExpressionNode::Simple(s)) if !s.content.trim().is_empty() => {
                    (s.content.clone(), s.loc.source.clone())
                }
                Some(ExpressionNode::Compound(c)) if !c.loc.source.trim().is_empty() => {
                    (c.loc.source.clone(), c.loc.source.clone())
                }
                _ => {
                    ctx.on_error(ErrorCode::VModelNoExpression, Some(dir.loc.clone()));
                    invalid_model_indices.push(idx);
                    continue;
                }
            };

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
                    ExpressionNode::Simple(exp) => exp.content.clone(),
                    ExpressionNode::Compound(exp) => exp.loc.source.clone(),
                })
                .unwrap_or_else(|| {
                    if is_component {
                        String::new("modelValue")
                    } else {
                        String::new("value")
                    }
                });

            // Create event name
            let event_name = if is_component {
                let mut name = String::with_capacity(9 + prop_name.len());
                name.push_str("onUpdate:");
                name.push_str(prop_name.as_str());
                name
            } else {
                // For native elements, use input event (or change for lazy)
                let has_lazy = dir.modifiers.iter().any(|m| m.content == "lazy");
                if has_lazy {
                    String::new("onChange")
                } else {
                    String::new("onInput")
                }
            };

            // Build handler expression
            let handler = if is_component {
                let mut out = String::with_capacity(raw_value_exp.len() + 20);
                out.push_str("$event => ((");
                out.push_str(raw_value_exp.as_str());
                out.push_str(") = $event)");
                out
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
                let mut out = String::with_capacity(raw_value_exp.len() + target_value.len() + 16);
                out.push_str("$event => ((");
                out.push_str(raw_value_exp.as_str());
                out.push_str(") = ");
                out.push_str(target_value.as_str());
                out.push(')');
                out
            };

            let dir_loc = dir.loc.clone();

            // Collect modifiers info for components
            let (modifiers_obj, modifiers_key) = if is_component && !dir.modifiers.is_empty() {
                let modifiers_content: std::vec::Vec<String> = dir
                    .modifiers
                    .iter()
                    .map(|m| {
                        let mut item = String::with_capacity(m.content.len() + 6);
                        item.push_str(&m.content);
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

        // For static v-model: remove directives and add generated props
        // First remove all static v-model directives in reverse order (to preserve indices)
        for data in static_vmodel.iter().rev() {
            el.props.remove(data.idx);
        }

        // Then add all generated props in forward order
        for data in static_vmodel.iter() {
            // Add :propName prop
            let value_prop = PropNode::Directive(Box::new_in(
                DirectiveNode {
                    name: String::new("bind"),
                    raw_name: None,
                    arg: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode::new(
                            data.prop_name.clone(),
                            true,
                            data.dir_loc.clone(),
                        ),
                        allocator,
                    ))),
                    exp: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode {
                            content: data.value_exp.clone(),
                            is_static: false,
                            const_type: ConstantType::NotConstant,
                            loc: data.dir_loc.clone(),
                            js_ast: None,
                            hoisted: None,
                            identifiers: None,
                            is_handler_key: false,
                            is_ref_transformed: true, // Already processed for ref .value
                        },
                        allocator,
                    ))),
                    modifiers: Vec::new_in(allocator),
                    for_parse_result: None,
                    shorthand: false,
                    loc: data.dir_loc.clone(),
                },
                allocator,
            ));
            el.props.push(value_prop);

            // Add @update:propName prop
            let raw_handler_expr = ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(data.handler.as_str(), false, data.dir_loc.clone()),
                allocator,
            ));
            let processed_handler = process_inline_handler(ctx, &raw_handler_expr);

            let event_prop = PropNode::Directive(Box::new_in(
                DirectiveNode {
                    name: String::new("on"),
                    raw_name: None,
                    arg: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode::new(
                            &data.event_name[2..],
                            true,
                            data.dir_loc.clone(),
                        ), // Remove "on" prefix
                        allocator,
                    ))),
                    exp: Some(processed_handler),
                    modifiers: Vec::new_in(allocator),
                    for_parse_result: None,
                    shorthand: false,
                    loc: data.dir_loc.clone(),
                },
                allocator,
            ));
            el.props.push(event_prop);

            // Add modelModifiers prop for components with modifiers
            if let (Some(modifiers_obj), Some(modifiers_key)) =
                (&data.modifiers_obj, &data.modifiers_key)
            {
                let modifiers_prop = PropNode::Directive(Box::new_in(
                    DirectiveNode {
                        name: String::new("bind"),
                        raw_name: None,
                        arg: Some(ExpressionNode::Simple(Box::new_in(
                            SimpleExpressionNode::new(
                                modifiers_key.clone(),
                                true,
                                data.dir_loc.clone(),
                            ),
                            allocator,
                        ))),
                        exp: Some(ExpressionNode::Simple(Box::new_in(
                            SimpleExpressionNode::new(
                                modifiers_obj.as_str(),
                                false,
                                data.dir_loc.clone(),
                            ),
                            allocator,
                        ))),
                        modifiers: Vec::new_in(allocator),
                        for_parse_result: None,
                        shorthand: false,
                        loc: SourceLocation::STUB,
                    },
                    allocator,
                ));
                el.props.push(modifiers_prop);
            }
        }

        // For dynamic v-model: keep the directive (will be handled in codegen with normalizeProps)
        // Just mark that we have dynamic v-model by adding helper
        if !dynamic_vmodel.is_empty() {
            ctx.helper(RuntimeHelper::NormalizeProps);
        }
    } else {
        // For native elements: process in reverse order to preserve indices during insertion
        for data in vmodel_data.iter().rev() {
            // Keep v-model directive, insert onUpdate:modelValue handler right after it
            let mut handler = String::with_capacity(data.raw_value_exp.len() + 20);
            handler.push_str("$event => ((");
            handler.push_str(data.raw_value_exp.as_str());
            handler.push_str(") = $event)");
            let raw_handler_expr = ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(handler.as_str(), false, data.dir_loc.clone()),
                allocator,
            ));
            let processed_handler = process_inline_handler(ctx, &raw_handler_expr);
            let event_prop = PropNode::Directive(Box::new_in(
                DirectiveNode {
                    name: String::new("on"),
                    raw_name: None,
                    arg: Some(ExpressionNode::Simple(Box::new_in(
                        SimpleExpressionNode::new("update:modelValue", true, data.dir_loc.clone()),
                        allocator,
                    ))),
                    exp: Some(processed_handler),
                    modifiers: Vec::new_in(allocator),
                    for_parse_result: None,
                    shorthand: false,
                    loc: data.dir_loc.clone(),
                },
                allocator,
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
        use crate::transforms::transform_expression::process_expression;
        let processed = process_expression(ctx, &interp.content, false);
        interp.content = processed;
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use bumpalo::Bump;

    use super::transform_element;
    use crate::{
        ast::{PropNode, TemplateChildNode},
        errors::ErrorCode,
        options::TransformOptions,
        parser::parse,
        transform::TransformContext,
    };

    #[test]
    fn test_transform_v_model_without_expression_reports_error() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(&allocator, r#"<input v-model />"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx =
            TransformContext::new(&allocator, root.source.clone(), TransformOptions::default());
        match &mut root.children[0] {
            TemplateChildNode::Element(el) => {
                transform_element(&mut ctx, el);

                assert!(!el.props.iter().any(|prop| matches!(
                    prop,
                    PropNode::Directive(dir) if dir.name == "model"
                )));
            }
            other => panic!(
                "Expected ElementNode, got {:?}",
                std::mem::discriminant(other)
            ),
        }

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors[0].code, ErrorCode::VModelNoExpression);
    }

    #[test]
    fn test_transform_component_v_model_without_expression_reports_error() {
        let allocator = Bump::new();
        let (mut root, errors) = parse(&allocator, r#"<MyComponent v-model />"#);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);

        let mut ctx =
            TransformContext::new(&allocator, root.source.clone(), TransformOptions::default());
        match &mut root.children[0] {
            TemplateChildNode::Element(el) => {
                transform_element(&mut ctx, el);

                assert!(!el.props.iter().any(|prop| matches!(
                    prop,
                    PropNode::Directive(dir) if dir.name == "model"
                )));
            }
            other => panic!(
                "Expected ElementNode, got {:?}",
                std::mem::discriminant(other)
            ),
        }

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors[0].code, ErrorCode::VModelNoExpression);
    }
}
