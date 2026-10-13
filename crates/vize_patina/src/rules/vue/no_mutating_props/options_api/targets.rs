//! A proved first static property of direct instance `this`, then access depth.

use oxc_ast::ast::{Expression, MemberExpression, SimpleAssignmentTarget};
use vize_l0::{FxHashSet, String};

pub(super) fn expression<'a>(
    expression: &'a Expression<'a>,
    props: &FxHashSet<String>,
) -> Option<(&'a str, bool)> {
    member(expression.get_member_expr()?, props)
}

pub(super) fn assignment<'a>(
    target: &'a SimpleAssignmentTarget<'a>,
    props: &FxHashSet<String>,
) -> Option<(&'a str, bool)> {
    if let Some(target) = target.as_member_expression() {
        member(target, props)
    } else {
        target
            .get_expression()
            .and_then(|target| expression(target, props))
    }
}

fn member<'a>(
    mut member: &'a MemberExpression<'a>,
    props: &FxHashSet<String>,
) -> Option<(&'a str, bool)> {
    let mut deep = false;
    loop {
        let object = member.object().get_inner_expression();
        if matches!(object, Expression::ThisExpression(_)) {
            let name = member.static_property_name()?;
            return props.contains(name).then_some((name, deep));
        }
        member = object.get_member_expr()?;
        deep = true;
    }
}

pub(super) fn object_assign(expression: &Expression<'_>) -> bool {
    expression.get_member_expr().is_some_and(|member| {
        member.static_property_name() == Some("assign")
            && matches!(member.object().get_inner_expression(), Expression::Identifier(identifier) if identifier.name.as_str() == "Object")
    })
}
