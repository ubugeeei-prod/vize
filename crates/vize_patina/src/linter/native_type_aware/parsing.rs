use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{
    CallExpression, ChainElement, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    Statement,
};
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::operator::UnaryOperator;
use vize_carton::{profile, String, ToCompactString};

pub(super) fn is_runtime_array_macro(runtime_args: Option<&str>) -> bool {
    let Some(runtime_args) = runtime_args.map(str::trim_start) else {
        return false;
    };
    if runtime_args.starts_with('[') {
        return true;
    }
    runtime_args
        .find('(')
        .and_then(|open| runtime_args.get(open + 1..))
        .map(str::trim_start)
        .is_some_and(|after_paren| after_paren.starts_with('['))
}

pub(super) fn extract_runtime_object_property_values(source: &str) -> Vec<String> {
    let mut wrapped = String::with_capacity(source.len() + 32);
    wrapped.push_str("const __vize_runtime = (");
    let expression_offset = wrapped.len();
    wrapped.push_str(source);
    wrapped.push_str(");");

    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("runtime.ts").unwrap_or_default();
    let parsed = profile!(
        "patina.type_aware.runtime_object.parse",
        OxcParser::new(&allocator, wrapped.as_str(), source_type).parse()
    );
    if parsed.panicked {
        return Vec::new();
    }

    let Some(Statement::VariableDeclaration(declaration)) = parsed.program.body.first() else {
        return Vec::new();
    };
    let Some(declarator) = declaration.declarations.first() else {
        return Vec::new();
    };
    let Some(init) = declarator.init.as_ref() else {
        return Vec::new();
    };
    let Some(object) = unwrap_object_expression(init) else {
        return Vec::new();
    };

    let mut values = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property_name(&property.key).is_none() {
            continue;
        }
        let value_start = property.value.span().start as usize;
        let value_end = property.value.span().end as usize;
        let local_start = value_start.saturating_sub(expression_offset);
        let local_end = value_end.saturating_sub(expression_offset);
        if let Some(value_source) = source.get(local_start..local_end) {
            values.push(value_source.to_compact_string());
        }
    }

    values
}

#[derive(Clone, Copy)]
pub(super) struct FloatingCandidate {
    pub start: u32,
    pub end: u32,
}

pub(super) fn collect_floating_candidates(source: &str) -> Vec<FloatingCandidate> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let parsed = profile!(
        "patina.type_aware.floating_candidates.parse",
        OxcParser::new(&allocator, source, source_type).parse()
    );
    if parsed.panicked {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for statement in parsed.program.body.iter() {
        if let Statement::ExpressionStatement(expression_statement) = statement {
            if is_explicitly_handled(&expression_statement.expression) {
                continue;
            }
            let span = expression_statement.expression.span();
            candidates.push(FloatingCandidate {
                start: span.start,
                end: span.end,
            });
        }
    }
    candidates
}

fn unwrap_object_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(paren) => unwrap_object_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => unwrap_object_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_object_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_object_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn is_explicitly_handled(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::AwaitExpression(_) => true,
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        Expression::ParenthesizedExpression(paren) => is_explicitly_handled(&paren.expression),
        Expression::TSAsExpression(ts_as) => is_explicitly_handled(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_explicitly_handled(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_explicitly_handled(&ts_non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => {
                is_macro_call_expression(&call.callee) || is_handled_call(call)
            }
            ChainElement::TSNonNullExpression(non_null) => {
                is_explicitly_handled(&non_null.expression)
            }
            _ => false,
        },
        Expression::CallExpression(call) => {
            is_macro_call_expression(&call.callee) || is_handled_call(call)
        }
        _ => false,
    }
}

fn is_handled_call(call: &CallExpression<'_>) -> bool {
    call.callee
        .as_member_expression()
        .and_then(|member| member.static_property_name())
        .is_some_and(|name| matches!(name, "then" | "catch" | "finally"))
}

fn is_macro_call_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(identifier) => matches!(
            identifier.name.as_str(),
            "defineProps"
                | "defineEmits"
                | "defineExpose"
                | "defineOptions"
                | "defineSlots"
                | "defineModel"
                | "withDefaults"
        ),
        _ => false,
    }
}
