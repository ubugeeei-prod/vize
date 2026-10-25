//! Project static defaults without evaluating component or factory code.

use oxc_ast::ast::{
    ArrayExpressionElement, Expression, ObjectPropertyKind, PropertyKey, PropertyKind, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::{Map, Number, Value};
use vize_l0::Allocator;

pub(super) struct DefaultInput<'a> {
    pub source: Option<&'a str>,
    pub allow_factory: bool,
}

pub(super) struct ProjectedDefault {
    pub value: Option<Value>,
    pub unsupported: bool,
}

/// One expression parse for all defaults, including single-quoted JavaScript
/// strings. Parentheses retain comma-expression ownership inside the array.
pub(super) fn project_defaults(inputs: &[DefaultInput<'_>]) -> Vec<ProjectedDefault> {
    if inputs.iter().all(|input| input.source.is_none()) {
        return inputs
            .iter()
            .map(|_| ProjectedDefault {
                value: None,
                unsupported: false,
            })
            .collect();
    }
    let mut source = String::from("[");
    for (index, input) in inputs.iter().enumerate() {
        if index != 0 {
            source.push(',');
        }
        source.push('(');
        source.push_str(input.source.unwrap_or("undefined"));
        source.push(')');
    }
    source.push(']');
    let allocator = Allocator::new();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse_expression();
    let Ok(Expression::ArrayExpression(array)) = parsed else {
        return inputs
            .iter()
            .map(|input| ProjectedDefault {
                value: None,
                unsupported: input.source.is_some(),
            })
            .collect();
    };
    inputs
        .iter()
        .zip(array.elements.iter())
        .map(|(input, element)| {
            if input.source.is_none() {
                return ProjectedDefault {
                    value: None,
                    unsupported: false,
                };
            }
            let mut budget = 4096;
            let value = element.as_expression().and_then(|expression| {
                static_value(expression, input.allow_factory, 0, &mut budget)
            });
            ProjectedDefault {
                unsupported: value.is_none(),
                value,
            }
        })
        .collect()
}

fn static_value(
    expression: &Expression<'_>,
    factory: bool,
    depth: usize,
    budget: &mut usize,
) -> Option<Value> {
    if depth > 32 || *budget == 0 {
        return None;
    }
    *budget -= 1;
    let expression = expression.get_inner_expression();
    match expression {
        Expression::StringLiteral(value) => Some(Value::String(value.value.as_str().to_owned())),
        Expression::TemplateLiteral(value) if value.expressions.is_empty() => {
            let [quasi] = value.quasis.as_slice() else {
                return None;
            };
            Some(Value::String(
                quasi.value.cooked.as_ref()?.as_str().to_owned(),
            ))
        }
        Expression::NumericLiteral(value) => Number::from_f64(value.value).map(Value::Number),
        Expression::BooleanLiteral(value) => Some(Value::Bool(value.value)),
        Expression::NullLiteral(_) => Some(Value::Null),
        Expression::UnaryExpression(unary) => {
            let Expression::NumericLiteral(value) = unary.argument.get_inner_expression() else {
                return None;
            };
            let value = match unary.operator.as_str() {
                "-" => -value.value,
                "+" => value.value,
                _ => return None,
            };
            Number::from_f64(value).map(Value::Number)
        }
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .map(|element| match element {
                ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => {
                    None
                }
                _ => element
                    .as_expression()
                    .and_then(|value| static_value(value, false, depth + 1, budget)),
            })
            .collect::<Option<Vec<_>>>()
            .map(Value::Array),
        Expression::ObjectExpression(object) => {
            let mut values = Map::new();
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.method || property.kind != PropertyKind::Init {
                    return None;
                }
                let name = match &property.key {
                    PropertyKey::StaticIdentifier(name) => name.name.as_str().to_owned(),
                    PropertyKey::StringLiteral(name) => name.value.as_str().to_owned(),
                    PropertyKey::NumericLiteral(name) => name.value.to_string(),
                    _ => return None,
                };
                // Do not turn JavaScript prototype operations into JSON own data.
                if name == "__proto__" {
                    return None;
                }
                values.insert(
                    name,
                    static_value(&property.value, false, depth + 1, budget)?,
                );
            }
            Some(Value::Object(values))
        }
        Expression::ArrowFunctionExpression(function)
            if factory
                && !function.r#async
                && function.params.items.is_empty()
                && function.params.rest.is_none() =>
        {
            let [statement] = function.body.statements.as_slice() else {
                return None;
            };
            match statement {
                Statement::ExpressionStatement(statement) if function.expression => {
                    static_value(&statement.expression, false, depth + 1, budget)
                }
                Statement::ReturnStatement(statement) if !function.expression => {
                    static_value(statement.argument.as_ref()?, false, depth + 1, budget)
                }
                _ => None,
            }
        }
        Expression::FunctionExpression(function)
            if factory
                && !function.generator
                && !function.r#async
                && function.params.items.is_empty()
                && function.params.rest.is_none() =>
        {
            let body = function.body.as_ref()?;
            let [Statement::ReturnStatement(statement)] = body.statements.as_slice() else {
                return None;
            };
            static_value(statement.argument.as_ref()?, false, depth + 1, budget)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultInput, project_defaults};
    use serde_json::Value;
    use serde_json::json;

    #[test]
    fn preserves_complete_static_values_and_factory_shapes() {
        let sources = [
            "'true'",
            "'007'",
            "'日本語 \\n 🎨'",
            "false",
            "-2.5",
            "null",
            "() => ({ flag: true, count: 3, items: [1, 'x'] })",
            "function () { return ['a', false] }",
            "({ 'data-test': 1 } as const)",
        ];
        let inputs: Vec<_> = sources
            .iter()
            .map(|source| DefaultInput {
                source: Some(source),
                allow_factory: true,
            })
            .collect();
        let values = project_defaults(&inputs);
        assert_eq!(
            values
                .into_iter()
                .map(|value| value.value)
                .collect::<Vec<_>>(),
            vec![
                Some(json!("true")),
                Some(json!("007")),
                Some(json!("日本語 \n 🎨")),
                Some(json!(false)),
                Some(json!(-2.5)),
                Some(Value::Null),
                Some(json!({"flag":true,"count":3.0,"items":[1.0,"x"]})),
                Some(json!(["a", false])),
                Some(json!({"data-test":1.0}))
            ]
        );
    }

    #[test]
    fn unsupported_defaults_are_explicit_and_never_execute() {
        let sources = [
            "undefined",
            "new Date()",
            "sideEffect()",
            "() => { sideEffect(); return 1 }",
            "() => readState()",
            "async () => 42",
            "function* () { return 42 }",
            "({ get value() { return 1 } })",
            "({ ...local })",
            "[...local]",
            "({ __proto__: { polluted: true } })",
        ];
        let inputs: Vec<_> = sources
            .iter()
            .map(|source| DefaultInput {
                source: Some(source),
                allow_factory: true,
            })
            .collect();
        for value in project_defaults(&inputs) {
            assert!(value.unsupported);
            assert_eq!(value.value, None);
        }
        let values = project_defaults(&[
            DefaultInput {
                source: Some("() => 42"),
                allow_factory: false,
            },
            DefaultInput {
                source: None,
                allow_factory: false,
            },
        ]);
        assert!(values[0].unsupported);
        assert!(!values[1].unsupported);
    }
}
