//! Static property detection and hoisting.

use vize_carton::{Allocator, Box, String, ToCompactString, Vec, camelize};

use crate::codegen::is_constant_simple_expression;
use crate::lane::TransformContext;
use crate::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, JsChildNode, ObjectExpression,
    PropNode, Property, PropsExpression, SimpleExpressionNode, SourceLocation,
};

pub(super) fn create_props_expression<'a>(
    allocator: &'a Allocator,
    props: &[PropNode<'a>],
    scope_id: Option<&'a str>,
    skip_is: bool,
) -> Option<PropsExpression<'a>> {
    let mut obj_props = Vec::new_in(&allocator);
    let mut seen: vize_carton::FxHashSet<String> = vize_carton::FxHashSet::default();

    for prop in props {
        if skip_is && is_is_prop(prop) {
            continue;
        }
        match prop {
            PropNode::Attribute(attr) => {
                if seen.contains(attr.name) {
                    continue;
                }
                seen.insert(attr.name.into());

                let key = ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(attr.name, true, attr.loc.clone()),
                    &allocator,
                ));
                let value_exp = if let Some(value) = &attr.value {
                    SimpleExpressionNode::new(value.content, true, value.loc.clone())
                } else {
                    SimpleExpressionNode::new("", true, attr.loc.clone())
                };
                let value = JsChildNode::SimpleExpression(Box::new_in(value_exp, &allocator));
                obj_props.push(Property {
                    key,
                    value,
                    loc: attr.loc.clone(),
                });
            }
            PropNode::Directive(dir) => {
                let Some((name, exp)) = hoistable_static_bind_parts(dir) else {
                    continue;
                };
                if seen.contains(name.as_str()) {
                    continue;
                }
                seen.insert(name.clone());

                let key = ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(allocator.alloc_str(&name), true, dir.loc.clone()),
                    &allocator,
                ));
                let value = JsChildNode::SimpleExpression(Box::new_in(
                    clone_expression_for_hoist(exp),
                    &allocator,
                ));
                obj_props.push(Property {
                    key,
                    value,
                    loc: dir.loc.clone(),
                });
            }
        }
    }

    if let Some(scope_id) = scope_id {
        let key = ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(scope_id, true, SourceLocation::STUB),
            &allocator,
        ));
        let value = JsChildNode::SimpleExpression(Box::new_in(
            SimpleExpressionNode::new("", true, SourceLocation::STUB),
            &allocator,
        ));
        obj_props.push(Property {
            key,
            value,
            loc: SourceLocation::STUB,
        });
    }

    if obj_props.is_empty() {
        return None;
    }

    Some(PropsExpression::Object(Box::new_in(
        ObjectExpression {
            properties: obj_props,
            loc: SourceLocation::STUB,
        },
        &allocator,
    )))
}

pub(super) fn has_static_props(el: &ElementNode<'_>) -> bool {
    if el.tag_type == ElementType::Slot || el.props.is_empty() {
        return false;
    }
    el.props.iter().all(is_hoistable_static_prop)
}

pub(super) fn is_hoistable_static_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name != "ref",
        PropNode::Directive(dir) => hoistable_static_bind_parts(dir).is_some(),
    }
}

fn hoistable_static_bind_parts<'a, 'b>(
    dir: &'b DirectiveNode<'a>,
) -> Option<(String, &'b SimpleExpressionNode<'a>)> {
    if dir.name != "bind" {
        return None;
    }
    let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
        return None;
    };
    if !arg.is_static {
        return None;
    }

    let has_camel = dir
        .modifiers
        .iter()
        .any(|modifier| modifier.content == "camel");
    let has_prop = dir
        .modifiers
        .iter()
        .any(|modifier| modifier.content == "prop");
    let has_attr = dir
        .modifiers
        .iter()
        .any(|modifier| modifier.content == "attr");
    if dir
        .modifiers
        .iter()
        .any(|modifier| !matches!(modifier.content, "camel" | "prop" | "attr"))
    {
        return None;
    }

    let key = if has_camel {
        camelize(arg.content)
    } else if has_prop {
        prefixed_name('.', arg.content)
    } else if has_attr {
        prefixed_name('^', arg.content)
    } else {
        arg.content.to_compact_string()
    };
    if matches!(key.as_str(), "ref" | "class") {
        return None;
    }

    let Some(ExpressionNode::Simple(exp)) = &dir.exp else {
        return None;
    };
    is_constant_simple_expression(exp, None).then_some((key, exp))
}

fn prefixed_name(prefix: char, name: &str) -> String {
    let mut result = String::with_capacity(1 + name.len());
    result.push(prefix);
    result.push_str(name);
    result
}

pub(super) fn hoist_element_props<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut ElementNode<'a>,
    allocator: &'a Allocator,
) {
    let Some(PropsExpression::Object(obj_expr)) = create_props_expression(
        allocator,
        &el.props,
        ctx.options
            .scope_id
            .as_deref()
            .map(|id| allocator.alloc_str(id)),
        skip_is_on_dynamic_component(el),
    ) else {
        return;
    };

    let js_node = JsChildNode::Object(obj_expr);
    let hoist_index = ctx.hoist(js_node);
    el.hoisted_props_index = Some(hoist_index + 1);
}

/// Vue's `buildProps` drops `is` / `:is` on `<component>` / `<Component>`
/// before `hoistStatic` runs, so the hoisted object must not keep it.
fn skip_is_on_dynamic_component(el: &ElementNode<'_>) -> bool {
    el.tag == "component" || (el.tag == "Component" && el.props.iter().any(is_is_prop))
}

fn is_is_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "is")
        }
    }
}

fn clone_expression_for_hoist<'a>(exp: &SimpleExpressionNode<'a>) -> SimpleExpressionNode<'a> {
    SimpleExpressionNode {
        content: exp.content,
        is_static: false,
        const_type: exp.const_type,
        loc: exp.loc.clone(),
        js_ast: None,
        hoisted: None,
        identifiers: None,
        is_handler_key: false,
        is_ref_transformed: false,
    }
}
