//! Collection of `computed`/`methods` sources for the Options API bridge.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ArrowFunctionExpression, CallExpression, Expression,
    Function, ObjectExpression, ObjectPropertyKind,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use super::super::options_api::{
    component_options_from_program, option_object_property, property_key_name, safe_identifier,
    source_slice,
};
use super::{BridgeSpans, MappedType, OptionsApiBridge, OptionsFunction, OptionsFunctionKind};
use vize_carton::{CompactString, FxHashSet, String, append};

/// Generated prefix that wraps a concise arrow body into a statement body.
const ARROW_BODY_PREFIX: &str = "{ return ";

pub(super) fn collect_options_api_bridge(script: &str) -> Option<OptionsApiBridge> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }

    let options = component_options_from_program(&parsed.program)?;
    let mut bridge = OptionsApiBridge::default();
    collect_function_bridge(
        script,
        options,
        "computed",
        OptionsFunctionKind::Computed,
        &mut bridge.computed,
        &mut bridge.mapped_types,
    );
    collect_function_bridge(
        script,
        options,
        "methods",
        OptionsFunctionKind::Method,
        &mut bridge.methods,
        &mut bridge.mapped_types,
    );
    Some(bridge)
}

fn collect_function_bridge(
    script: &str,
    options: &ObjectExpression<'_>,
    option_name: &str,
    kind: OptionsFunctionKind,
    output: &mut Vec<OptionsFunction>,
    mapped_types: &mut Vec<MappedType>,
) {
    let Some(object) = option_object_property(options, option_name) else {
        return;
    };

    let mut used = FxHashSet::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                let Some(mut function) =
                    options_function_from_expression(script, name, &property.value, kind)
                else {
                    continue;
                };
                function.safe_name = unique_name(function.safe_name, &mut used);
                output.push(function);
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Expression::CallExpression(call) = &spread.argument {
                    collect_mapped_type(call, mapped_types);
                }
            }
        }
    }
}

/// `safe_identifier` maps every non-identifier byte to `_`, so distinct
/// authored keys can collapse onto one name (`"a-b"` and `"a.b"` both yield
/// `a_b`). Two bridge functions sharing a name raise TS2393 against an
/// authored body, so suffix repeats before anything is emitted.
fn unique_name(mut name: CompactString, used: &mut FxHashSet<CompactString>) -> CompactString {
    let base = name.clone();
    let mut counter = 1u32;
    while !used.insert(name.clone()) {
        counter = counter.saturating_add(1);
        let mut candidate = String::from(base.as_str());
        append!(candidate, "_{counter}");
        name = CompactString::new(candidate.as_str());
    }
    name
}

fn options_function_from_expression(
    script: &str,
    name: &str,
    expression: &Expression<'_>,
    kind: OptionsFunctionKind,
) -> Option<OptionsFunction> {
    let parts = match expression {
        Expression::FunctionExpression(function) => function_parts(script, function)?,
        Expression::ArrowFunctionExpression(arrow) => arrow_function_parts(script, arrow)?,
        Expression::ParenthesizedExpression(parenthesized) => {
            return options_function_from_expression(script, name, &parenthesized.expression, kind);
        }
        Expression::TSAsExpression(ts_as) => {
            return options_function_from_expression(script, name, &ts_as.expression, kind);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            return options_function_from_expression(script, name, &ts_satisfies.expression, kind);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            return options_function_from_expression(script, name, &ts_non_null.expression, kind);
        }
        _ => return None,
    };

    Some(OptionsFunction {
        kind,
        safe_name: CompactString::new(safe_identifier(name).as_str()),
        params: parts.params,
        body: parts.body,
        is_async: parts.is_async,
        is_generator: parts.is_generator,
        spans: parts.spans,
    })
}

struct FunctionParts {
    params: String,
    body: String,
    spans: BridgeSpans,
    is_async: bool,
    is_generator: bool,
}

fn function_parts(script: &str, function: &Function<'_>) -> Option<FunctionParts> {
    let params = params_source(script, &function.params)?;
    let body = function.body.as_ref()?;
    let (body_source, body_span) = trimmed_span(script, body.span())?;
    let spans = BridgeSpans {
        function: function.params.span().start as usize..body.span().end as usize,
        body: Some((0, body_span)),
    };
    Some(FunctionParts {
        params,
        body: String::from(body_source),
        spans,
        is_async: function.r#async,
        is_generator: function.generator,
    })
}

fn arrow_function_parts(
    script: &str,
    arrow: &ArrowFunctionExpression<'_>,
) -> Option<FunctionParts> {
    let params = params_source(script, &arrow.params)?;
    let (body_source, body_span) = trimmed_span(script, arrow.body.span())?;
    let function = arrow.params.span().start as usize..arrow.body.span().end as usize;
    if arrow.expression {
        // A concise body is wrapped, so only the expression itself is verbatim.
        let expression = body_source.trim_end_matches(';');
        let mut body = String::from(ARROW_BODY_PREFIX);
        body.push_str(expression);
        body.push_str("; }");
        let spans = BridgeSpans {
            function,
            body: Some((
                ARROW_BODY_PREFIX.len(),
                body_span.start..body_span.start.saturating_add(expression.len()),
            )),
        };
        Some(FunctionParts {
            params,
            body,
            spans,
            is_async: arrow.r#async,
            is_generator: false,
        })
    } else {
        let spans = BridgeSpans {
            function,
            body: Some((0, body_span)),
        };
        Some(FunctionParts {
            params,
            body: String::from(body_source),
            spans,
            is_async: arrow.r#async,
            is_generator: false,
        })
    }
}

/// Slice `span` out of `script` and return it trimmed, paired with the
/// script-relative range the trimmed text actually occupies.
fn trimmed_span(script: &str, span: oxc_span::Span) -> Option<(&str, Range<usize>)> {
    let raw = source_slice(script, span)?;
    let trimmed = raw.trim();
    let leading = raw.len().saturating_sub(raw.trim_start().len());
    let start = (span.start as usize).saturating_add(leading);
    Some((trimmed, start..start.saturating_add(trimmed.len())))
}

fn params_source(script: &str, params: &oxc_ast::ast::FormalParameters<'_>) -> Option<String> {
    let mut result = String::default();
    let mut first = true;
    for param in params.items.iter() {
        if !first {
            result.push_str(", ");
        }
        first = false;
        result.push_str(source_slice(script, param.span())?.trim());
    }
    if let Some(rest) = params.rest.as_ref() {
        if !first {
            result.push_str(", ");
        }
        result.push_str(source_slice(script, rest.span())?.trim());
    }
    Some(result)
}

fn collect_mapped_type(call: &CallExpression<'_>, mapped_types: &mut Vec<MappedType>) {
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    if !matches!(
        callee.name.as_str(),
        "mapState" | "mapGetters" | "mapWritableState" | "mapActions"
    ) {
        return;
    }

    let Some(Argument::Identifier(store)) = call.arguments.first() else {
        return;
    };
    let Some(Argument::ArrayExpression(keys)) = call.arguments.get(1) else {
        return;
    };
    let keys: Vec<&str> = keys
        .elements
        .iter()
        .filter_map(|element| {
            let ArrayExpressionElement::StringLiteral(literal) = element else {
                return None;
            };
            Some(literal.value.as_str())
        })
        .collect();
    if keys.is_empty() {
        return;
    }

    let mut key_union = String::default();
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            key_union.push_str(" | ");
        }
        append!(key_union, "'{key}'");
    }

    let mut text = String::default();
    append!(
        text,
        "[K in {key_union}]: ReturnType<typeof {}>[K]",
        store.name.as_str()
    );
    mapped_types.push(MappedType {
        text,
        src: call.span().start as usize..call.span().end as usize,
    });
}
