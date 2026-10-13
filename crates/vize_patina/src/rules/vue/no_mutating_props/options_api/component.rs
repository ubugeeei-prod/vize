//! Only literal options and ordinary functions whose receiver Vue binds.

use oxc_ast::ast::{
    Expression, Function, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKind,
};
use vize_l0::{FxHashSet, String};

/// An unknown later spread/computed option cannot prove an earlier definition.
fn option<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a ObjectProperty<'a>> {
    for property in object.properties.iter().rev() {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        match property.key.static_name() {
            Some(key) if key == name => return Some(property),
            None => return None,
            _ => {}
        }
    }
    None
}

pub(super) fn prop_names<'a>(options: &'a ObjectExpression<'a>) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let Some(property) =
        option(options, "props").filter(|property| property.kind == PropertyKind::Init)
    else {
        return names;
    };
    match property.value.get_inner_expression() {
        Expression::ObjectExpression(props) => {
            for property in &props.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property
                    && let Some(name) = property.key.static_name()
                {
                    names.insert(String::from(name.as_ref()));
                }
            }
        }
        Expression::ArrayExpression(props) => {
            for element in &props.elements {
                if let Some(Expression::StringLiteral(name)) = element
                    .as_expression()
                    .map(Expression::get_inner_expression)
                {
                    names.insert(String::from(name.value.as_str()));
                }
            }
        }
        _ => {}
    }
    names
}

fn ordinary<'a>(property: &'a ObjectProperty<'a>) -> Option<&'a Function<'a>> {
    if property.kind != PropertyKind::Init {
        return None;
    }
    match property.value.get_inner_expression() {
        Expression::FunctionExpression(function) => Some(function),
        _ => None,
    }
}

fn object<'a>(property: &'a ObjectProperty<'a>) -> Option<&'a ObjectExpression<'a>> {
    if property.kind != PropertyKind::Init {
        return None;
    }
    match property.value.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

pub(super) fn visit_instance_functions<'a>(
    options: &'a ObjectExpression<'a>,
    mut visit: impl FnMut(&'a Function<'a>),
) {
    for name in [
        "beforeCreate",
        "created",
        "beforeMount",
        "mounted",
        "beforeUpdate",
        "updated",
        "activated",
        "deactivated",
        "beforeDestroy",
        "destroyed",
        "beforeUnmount",
        "unmounted",
        "errorCaptured",
        "renderTracked",
        "renderTriggered",
        "serverPrefetch",
        "data",
        "provide",
        "render",
        "renderError",
    ] {
        if let Some(function) = option(options, name).and_then(ordinary) {
            visit(function);
        }
    }
    for name in ["methods", "computed", "watch"] {
        let Some(group) = option(options, name).and_then(object) else {
            continue;
        };
        for property in &group.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let Some(key) = property.key.static_name() else {
                continue;
            };
            if !option(group, &key).is_some_and(|last| std::ptr::eq(last, &**property)) {
                continue;
            }
            if let Some(function) = ordinary(property) {
                visit(function);
            } else if name == "computed" {
                if let Some(accessors) = object(property) {
                    for key in ["get", "set"] {
                        if let Some(function) = option(accessors, key).and_then(ordinary) {
                            visit(function);
                        }
                    }
                }
            } else if name == "watch" {
                watch_handlers(&property.value, &mut visit);
            }
        }
    }
}

fn watch_handlers<'a>(expression: &'a Expression<'a>, visit: &mut impl FnMut(&'a Function<'a>)) {
    match expression.get_inner_expression() {
        Expression::FunctionExpression(function) => visit(function),
        Expression::ObjectExpression(handler) => {
            if let Some(function) = option(handler, "handler").and_then(ordinary) {
                visit(function);
            }
        }
        Expression::ArrayExpression(handlers) => {
            for handler in &handlers.elements {
                if let Some(handler) = handler.as_expression() {
                    watch_handlers(handler, visit);
                }
            }
        }
        _ => {}
    }
}
