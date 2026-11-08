//! The names declared in the Options API `computed` option.
//!
//! Component options resolution mirrors `no_side_effects_in_computed`: a plain
//! object, an identifier bound to one, or a `defineComponent(...)` wrapper,
//! optionally through TS expression wrappers.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use vize_l0::{CompactString, FxHashMap, FxHashSet};

/// Collect the names declared in the `computed` option. Only plain
/// (non-computed-key) properties contribute; spreads like `...mapGetters([..])`
/// are skipped since their member names are not statically known.
pub(super) fn collect_computed_names<'a>(
    options: &'a ObjectExpression<'a>,
) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    let Some(computed) = find_computed_object(options) else {
        return names;
    };
    for property in &computed.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if let Some(name) = property_key_name(&property.key)
            && !computed_directly_returns_function(&property.value)
        {
            names.insert(CompactString::from(name));
        }
    }
    names
}

fn computed_directly_returns_function(value: &Expression<'_>) -> bool {
    let value = match value.get_inner_expression() {
        Expression::ObjectExpression(descriptor) => descriptor
            .properties
            .iter()
            .find_map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                (!property.computed && property_key_name(&property.key) == Some("get"))
                    .then_some(&property.value)
            })
            .unwrap_or(value),
        value => value,
    };
    match value.get_inner_expression() {
        Expression::ArrowFunctionExpression(arrow) if arrow.expression => arrow
            .body
            .statements
            .first()
            .and_then(|statement| match statement {
                Statement::ExpressionStatement(statement) => Some(&statement.expression),
                _ => None,
            })
            .is_some_and(expression_may_be_function),
        Expression::ArrowFunctionExpression(arrow) => arrow
            .body
            .statements
            .iter()
            .any(statement_may_return_function),
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()
            .is_some_and(|body| body.statements.iter().any(statement_may_return_function)),
        _ => false,
    }
}

fn statement_may_return_function(statement: &Statement<'_>) -> bool {
    let Statement::ReturnStatement(statement) = statement else {
        return false;
    };
    statement
        .argument
        .as_ref()
        .is_some_and(expression_may_be_function)
}

fn expression_may_be_function(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::BinaryExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_) => false,
        Expression::ConditionalExpression(conditional) => {
            expression_may_be_function(&conditional.consequent)
                || expression_may_be_function(&conditional.alternate)
        }
        Expression::LogicalExpression(logical) => {
            expression_may_be_function(&logical.left) || expression_may_be_function(&logical.right)
        }
        // Known callable namespaces can intentionally select a function
        // (`Math[method] || Math.round`). Do not generalize this to arbitrary
        // member reads: `return this.value` is the ordinary invalid case.
        Expression::StaticMemberExpression(member) => callable_namespace(&member.object),
        Expression::ComputedMemberExpression(member) => callable_namespace(&member.object),
        _ => false,
    }
}

fn callable_namespace(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::Identifier(identifier) if identifier.name == "Math"
    )
}

fn find_computed_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("computed"))
            && let Expression::ObjectExpression(object) = &property.value
        {
            return Some(object);
        }
    }
    None
}

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

pub(super) fn find_component_options<'a>(
    program: &'a Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let mut bindings: FxHashMap<&'a str, &'a ObjectExpression<'a>> = FxHashMap::default();

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(object) = options_from_expression(init, &bindings)
            {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, &bindings) {
            return Some(object);
        }
    }

    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    // Every export-default kind we care about is also an `Expression` variant,
    // so route through the shared expression resolver.
    options_from_expression(declaration.as_expression()?, bindings)
}

/// Resolve an expression to the component options object, peeling
/// parenthesized/TS-cast wrappers, following identifier bindings, and entering
/// `defineComponent(...)` calls.
fn options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => options_from_expression(&ts_as.expression, bindings),
        Expression::TSSatisfiesExpression(ts) => options_from_expression(&ts.expression, bindings),
        Expression::TSNonNullExpression(ts) => options_from_expression(&ts.expression, bindings),
        _ => None,
    }
}

fn options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| options_from_expression(expression, bindings)),
    }
}
