//! Event ownership comes from the complete expression AST, including generic
//! callbacks, destructuring and comments. Inline statements alone bind `$event`.

use super::slot_props::extract_slot_binding_names;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ChainElement, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, SmallVec, cstr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventHandlerExpression {
    Inline,
    Reference,
    Callback {
        params: SmallVec<[CompactString; 4]>,
        accepts_event: bool,
    },
}

pub fn classify_event_handler(source: &str) -> EventHandlerExpression {
    let allocator = Allocator::default();
    // Parsing the entire wrapped program rejects valid expression prefixes
    // followed by statements, which must retain their implicit event scope.
    let wrapped = cstr!("({source}\n)");
    let parsed = Parser::new(&allocator, &wrapped, SourceType::ts()).parse();
    let parsed = if parsed.panicked || !parsed.diagnostics.is_empty() {
        Parser::new(&allocator, &wrapped, SourceType::tsx()).parse()
    } else {
        parsed
    };
    if parsed.panicked || !parsed.diagnostics.is_empty() || parsed.program.body.len() != 1 {
        return EventHandlerExpression::Inline;
    }
    let Statement::ExpressionStatement(statement) = &parsed.program.body[0] else {
        return EventHandlerExpression::Inline;
    };
    let parameters = match statement.expression.get_inner_expression() {
        Expression::ArrowFunctionExpression(function) => &function.params,
        Expression::FunctionExpression(function) => &function.params,
        Expression::Identifier(identifier) if identifier.name != "undefined" => {
            return EventHandlerExpression::Reference;
        }
        Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_) => {
            return EventHandlerExpression::Reference;
        }
        Expression::ChainExpression(chain)
            if matches!(
                chain.expression,
                ChainElement::StaticMemberExpression(_) | ChainElement::ComputedMemberExpression(_)
            ) =>
        {
            return EventHandlerExpression::Reference;
        }
        _ => return EventHandlerExpression::Inline,
    };
    let mut bindings = SmallVec::new();
    for parameter in &parameters.items {
        extract_slot_binding_names(&parameter.pattern, 0, &mut bindings);
    }
    if let Some(rest) = &parameters.rest {
        extract_slot_binding_names(&rest.rest.argument, 0, &mut bindings);
    }
    EventHandlerExpression::Callback {
        params: bindings.into_iter().map(|(name, _)| name).collect(),
        accepts_event: !parameters.items.is_empty() || parameters.rest.is_some(),
    }
}

pub fn extract_inline_callback_params(source: &str) -> Option<SmallVec<[CompactString; 4]>> {
    if !source.contains("=>") && !source.contains("function") {
        return None;
    }
    match classify_event_handler(source) {
        EventHandlerExpression::Callback { params, .. } => Some(params),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::extract_inline_callback_params;

    #[test]
    fn parameters_belong_only_to_the_outer_callback() {
        for (source, expected) in [
            (
                "({ key: local = (() => 1)(), ...rest }, [first, , last]) => local",
                vec!["local", "rest", "first", "last"],
            ),
            (
                "async <T>(value: T, next = () => value) => next()",
                vec!["value", "next"],
            ),
            (
                "(function (value /* ) => */) { return value; })",
                vec!["value"],
            ),
            ("() => () => nested", vec![]),
        ] {
            let actual = extract_inline_callback_params(source).expect(source);
            assert_eq!(
                actual.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
                expected,
                "{source}"
            );
        }
        for source in [
            "function f<T>(value: T) { return value; } f(1)",
            "const callback = () => 1; callback()",
            "items.map(item => item)",
            "run('function () => punctuation')",
            "if (ready) run(() => value)",
        ] {
            assert!(extract_inline_callback_params(source).is_none(), "{source}");
        }
    }
}
