//! Static attrs plus static-name `ui.bind` / `ui.on` props / patch flags.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BinaryOperator, Expression, ObjectPropertyKind,
};
use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::on::{admit_on, event_key_for, needs_hydration};

pub(super) use super::props_bind::{
    BindName, StaticBindKeyCasing, bind_name, emit_dynamic_bind_pair, has_prop_modifier,
    is_dynamic_bind_name, is_emitted_key_bind, js_value, static_bind_key,
};
pub(super) use super::props_object::{Piece, emit_props_object, pieces};
pub(super) use super::props_value::bind_value;

pub(super) struct Patch {
    pub flag: i32,
    pub dynamic_props: StdVec<String>,
}

pub(super) fn admit_element_bindings(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<(), EmitError> {
    admit_bindings_inner(attributes, bindings, true)
}

fn admit_bindings_inner(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    allow_once: bool,
) -> Result<(), EmitError> {
    let mut class = false;
    let mut style = false;
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) if bind.name.is_none() => {
                super::merge::admit_object(bind)?;
            }
            BindingOp::On(on) if on.name.is_none() => {
                super::merge::admit_object_on(on)?;
            }
            BindingOp::Bind(bind) => {
                bind_value(bind)?;
                if let BindName::Static(name) = bind_name(bind)? {
                    match name {
                        "class" if class => {
                            return Err(EmitError::unsupported_at(
                                Reason::DuplicateClassBinding,
                                bind.span,
                            ));
                        }
                        "class" => class = true,
                        "style" if style => {
                            return Err(EmitError::unsupported_at(
                                Reason::DuplicateStyleBinding,
                                bind.span,
                            ));
                        }
                        "style" => style = true,
                        _ => {}
                    }
                }
            }
            BindingOp::On(on) => admit_on(on)?,
            BindingOp::Model(model) => super::model::admit(model)?,
            BindingOp::SlotContent(_) => {}
            BindingOp::VueDirective(_) if super::slots::is_slots_spread(binding) => {}
            BindingOp::VueDirective(directive) => super::directive::admit(directive)?,
            BindingOp::VueShow(show) => super::directive::admit_show(show)?,
            BindingOp::VueHtml(html) => super::html::admit(html)?,
            BindingOp::VueText(text) => super::vtext::admit(text)?,
            BindingOp::VueCloak(_) => {}
            BindingOp::VueOnce(_) if allow_once => {}
            BindingOp::VueMemo(memo) => super::memo::admit(memo)?,
            _ => {
                return Err(EmitError::unsupported_binding(
                    Reason::UnsupportedBindingKind,
                    binding,
                ));
            }
        }
    }
    if style
        && let Some(attr) = attributes
            .iter()
            .find(|attr| attr.name == "style" && attr.value.is_none())
    {
        return Err(EmitError::unsupported_at(
            Reason::BareStyleAttributeWithDynamicStyle,
            attr.span,
        ));
    }
    Ok(())
}

pub(super) fn bind_patch(
    bindings: &[BindingOp<'_>],
    is_component: bool,
    if_key: Option<&str>,
    for_item: bool,
) -> Patch {
    if super::merge::has_object_spread(bindings) {
        return super::merge::object_patch(bindings, is_component, if_key, for_item);
    }
    let mut flag = 0i32;
    let mut dynamic_props = StdVec::new();
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) => match bind_name(bind) {
                _ if is_emitted_key_bind(bind, if_key) => {
                    if for_item && is_dynamic_bind_name(bind) {
                        flag |= 16;
                    }
                }
                Ok(BindName::Static(raw_name)) => match raw_name {
                    "ref" => flag |= 512,
                    "class"
                        if bind_value_is_static_patchless(bind)
                            || bind_value_uses_legacy_patchless_runtime_expr(bind) => {}
                    "class" if !is_component => flag |= 2,
                    "style"
                        if bind_value_is_static_patchless(bind)
                            || bind_value_uses_legacy_patchless_runtime_expr(bind) => {}
                    "style" if !is_component => flag |= 4,
                    "key" => {}
                    key if key.ends_with("Modifiers")
                        || bind_value_is_static_patchless(bind)
                        || (!is_component
                            && bind_value_uses_legacy_patchless_bounded_string_concat(bind)) => {}
                    _ => {
                        flag |= 8;
                        let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
                            continue;
                        };
                        let owned = String::from(key.as_str());
                        if !dynamic_props.contains(&owned) {
                            dynamic_props.push(owned);
                        }
                        if has_prop_modifier(bind) {
                            flag |= 32;
                        }
                    }
                },
                Ok(BindName::Dynamic(_)) => {
                    flag |= 16;
                    if has_prop_modifier(bind) {
                        flag |= 32;
                    }
                }
                Ok(BindName::Spread) | Err(_) => {}
            },
            BindingOp::On(on) => {
                if super::on_dynamic::is_dynamic_on_name(on) {
                    flag |= 16;
                    continue;
                }
                let Ok(key) = event_key_for(on, !is_component) else {
                    continue;
                };
                flag |= 8;
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key.clone());
                }
                if !is_component && needs_hydration(key.as_str(), on) {
                    flag |= 32;
                }
            }
            BindingOp::Model(model) => {
                super::model::patch(model, is_component, &mut flag, &mut dynamic_props);
            }
            BindingOp::VueHtml(_) => {
                flag |= 8;
                let key = String::from("innerHTML");
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key);
                }
            }
            BindingOp::VueText(_) => {
                flag |= 8;
                let key = String::from("textContent");
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key);
                }
            }
            _ => {}
        }
    }
    if super::directive::has_runtime(bindings) && flag & (2 | 4 | 8 | 16) == 0 {
        flag |= 512;
    }
    if flag & 16 != 0 {
        flag &= !(2 | 4 | 8);
    }
    Patch {
        flag,
        dynamic_props,
    }
}

pub(super) fn prune_legacy_patchless_dynamic_props(
    bindings: &[BindingOp<'_>],
    dynamic_props: &mut StdVec<String>,
) {
    for binding in bindings.iter() {
        let BindingOp::Bind(bind) = binding else {
            continue;
        };
        if has_prop_modifier(bind) || !bind_value_uses_legacy_patchless_runtime_expr(bind) {
            continue;
        }
        let Ok(BindName::Static(_)) = bind_name(bind) else {
            continue;
        };
        let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
            continue;
        };
        dynamic_props.retain(|name| name.as_str() != key.as_str());
    }
}

pub(super) fn bind_value_is_static_patchless(bind: &vize_s2::op::BindOp<'_>) -> bool {
    match bind_value(bind) {
        Ok(value) => value.js().is_some_and(|js| is_static_bound_expr(js.ast)),
        Err(_) => false,
    }
}

fn bind_value_uses_legacy_patchless_runtime_expr(bind: &vize_s2::op::BindOp<'_>) -> bool {
    match bind_value(bind) {
        Ok(value) => value
            .js()
            .is_some_and(|js| is_legacy_patchless_runtime_expr(js.ast)),
        Err(_) => false,
    }
}

fn bind_value_uses_legacy_patchless_bounded_string_concat(bind: &vize_s2::op::BindOp<'_>) -> bool {
    match bind_value(bind) {
        Ok(value) => value
            .js()
            .is_some_and(|js| is_legacy_bounded_string_concat(js.ast)),
        Err(_) => false,
    }
}

fn is_legacy_patchless_runtime_expr(expr: &Expression<'_>) -> bool {
    is_legacy_bounded_string_concat(expr) || is_legacy_in_conditional(expr)
}

fn is_legacy_bounded_string_concat(expr: &Expression<'_>) -> bool {
    let Expression::BinaryExpression(binary) = expr else {
        return false;
    };
    binary.operator == BinaryOperator::Addition
        && concat_left_edge_is_string(&binary.left)
        && concat_right_edge_is_string(&binary.right)
}

fn concat_left_edge_is_string(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_) => true,
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            concat_left_edge_is_string(&binary.left)
        }
        _ => false,
    }
}

fn concat_right_edge_is_string(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_) => true,
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            concat_right_edge_is_string(&binary.right)
        }
        _ => false,
    }
}

fn is_legacy_in_conditional(expr: &Expression<'_>) -> bool {
    let Expression::ConditionalExpression(conditional) = expr else {
        return false;
    };
    legacy_test_starts_with_in(&conditional.test)
}

fn legacy_test_starts_with_in(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BinaryExpression(binary) => binary.operator == BinaryOperator::In,
        Expression::LogicalExpression(logical) => {
            matches!(
                &logical.left,
                Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::In
            )
        }
        _ => false,
    }
}

fn is_static_bound_expr(expr: &Expression<'_>) -> bool {
    match unwrap_expr(expr) {
        Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::UnaryExpression(unary) => is_static_bound_expr(&unary.argument),
        Expression::ArrayExpression(array) => array.elements.iter().all(static_array_element),
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            is_static_bound_expr(&property.value)
        }),
        Expression::CallExpression(call)
            if matches!(
                &call.callee,
                Expression::Identifier(ident)
                    if matches!(ident.name.as_str(), "_normalizeClass" | "_normalizeStyle")
            ) =>
        {
            call.arguments.iter().all(static_argument)
        }
        _ => false,
    }
}

fn static_argument(argument: &Argument<'_>) -> bool {
    match argument {
        Argument::SpreadElement(_) => false,
        _ => argument.as_expression().is_some_and(is_static_bound_expr),
    }
}

fn static_array_element(element: &ArrayExpressionElement<'_>) -> bool {
    match element {
        ArrayExpressionElement::SpreadElement(_) => false,
        ArrayExpressionElement::Elision(_) => true,
        _ => element.as_expression().is_some_and(is_static_bound_expr),
    }
}

fn unwrap_expr<'a>(mut expr: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        match expr {
            Expression::ParenthesizedExpression(paren) => expr = &paren.expression,
            _ => return expr,
        }
    }
}

pub(super) fn emit_bind_props(
    cx: &mut EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    if_key: Option<&str>,
    skip_is: bool,
    for_item: bool,
    is_plain_element: bool,
    once_layout: bool,
    once_cache_initializer: bool,
    force_multiline: bool,
) -> Result<(), EmitError> {
    if super::merge::has_object_spread(bindings) {
        return super::merge::emit_spread_props(
            cx,
            attributes,
            bindings,
            if_key,
            skip_is,
            for_item,
            is_plain_element,
        );
    }
    let pieces = pieces(attributes, bindings, skip_is)?;
    let normalize = !for_item && has_dynamic_bind_name(bindings, if_key);
    if normalize {
        cx.buf.use_normalize_props();
        cx.buf.push(Buf::normalize_props_alias());
        cx.buf.push("(");
    }
    emit_props_object(
        cx,
        &pieces,
        if_key,
        false,
        for_item && (super::directive::has_custom(bindings) || once_layout || force_multiline),
        is_plain_element,
        for_item,
        once_cache_initializer,
        once_layout || force_multiline,
    )?;
    if normalize {
        cx.buf.push(")");
    }
    Ok(())
}

pub(super) fn apply_static_ref_patch(attributes: &[Attribute<'_>], flag: &mut i32) {
    let has_static_ref = attributes.iter().any(|attr| attr.name == "ref");
    if has_static_ref && *flag & (2 | 4 | 8 | 16 | 32) == 0 {
        *flag |= 512;
    }
}

fn has_dynamic_bind_name(bindings: &[BindingOp<'_>], if_key: Option<&str>) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) => is_dynamic_bind_name(bind) && !is_emitted_key_bind(bind, if_key),
        BindingOp::Model(model) => super::model::has_dynamic_argument(model),
        _ => false,
    })
}
