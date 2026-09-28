//! Prop default value extraction and normalization.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, Expression, ObjectPropertyKind, PropertyKey, PropertyKind, Statement,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::FxHashMap;
use vize_carton::{String, ToCompactString};

/// Default values supplied to `withDefaults`.
///
/// Object literals with statically named properties can be embedded directly
/// in the generated prop options. Other expressions must be preserved and
/// passed to Vue's `mergeDefaults` helper at runtime.
#[derive(Debug, Default)]
pub(crate) struct WithDefaultsValues {
    pub(crate) static_values: FxHashMap<String, String>,
    pub(crate) runtime_expression: Option<String>,
}

/// Extract default values from withDefaults second argument
/// Input: "withDefaults(defineProps<{...}>(), { prop1: default1, prop2: default2 })"
/// Returns: HashMap of prop name to default value string
pub fn extract_with_defaults_defaults(with_defaults_args: &str) -> FxHashMap<String, String> {
    extract_with_defaults_values(with_defaults_args).static_values
}

pub(crate) fn extract_with_defaults_values(with_defaults_args: &str) -> WithDefaultsValues {
    let mut values = WithDefaultsValues::default();
    let trimmed = with_defaults_args.trim();
    if trimmed.is_empty() {
        return values;
    }

    const WRAP_PREFIX: &str = "const __vize_defaults__ = ";
    let mut wrapped = String::with_capacity(WRAP_PREFIX.len() + trimmed.len() + 1);
    wrapped.push_str(WRAP_PREFIX);
    wrapped.push_str(trimmed);
    wrapped.push(';');

    let allocator = Allocator::default();
    let parse_result = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if !parse_result.diagnostics.is_empty() {
        return values;
    }

    let Some(Statement::VariableDeclaration(var_decl)) = parse_result.program.body.first() else {
        return values;
    };
    let Some(declarator) = var_decl.declarations.first() else {
        return values;
    };
    let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
        return values;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return values;
    };
    if callee.name.as_str() != "withDefaults" {
        return values;
    }

    let Some(defaults_argument) = call.arguments.get(1) else {
        return values;
    };
    let Argument::ObjectExpression(obj) = defaults_argument else {
        values.runtime_expression = source_for_span(trimmed, defaults_argument.span());
        return values;
    };

    for property in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            return runtime_defaults(trimmed, defaults_argument);
        };
        if prop.computed || prop.method || prop.kind != PropertyKind::Init {
            return runtime_defaults(trimmed, defaults_argument);
        }

        let key = match &prop.key {
            PropertyKey::StaticIdentifier(id) => id.name.to_compact_string(),
            PropertyKey::StringLiteral(lit) => lit.value.to_compact_string(),
            PropertyKey::NumericLiteral(lit) => lit.value.to_compact_string(),
            _ => return runtime_defaults(trimmed, defaults_argument),
        };

        let Some(value_src) = source_for_span(trimmed, prop.value.span()) else {
            return runtime_defaults(trimmed, defaults_argument);
        };
        values
            .static_values
            .insert(key, value_src.to_compact_string());
    }

    values
}

fn runtime_defaults(trimmed: &str, argument: &Argument<'_>) -> WithDefaultsValues {
    WithDefaultsValues {
        static_values: FxHashMap::default(),
        runtime_expression: source_for_span(trimmed, argument.span()),
    }
}

fn source_for_span(trimmed: &str, span: oxc_span::Span) -> Option<String> {
    const WRAP_PREFIX: &str = "const __vize_defaults__ = ";
    let start = (span.start as usize).checked_sub(WRAP_PREFIX.len())?;
    let end = (span.end as usize).checked_sub(WRAP_PREFIX.len())?;
    trimmed
        .get(start..end)
        .map(|source| source.to_compact_string())
}

/// Normalize default values from reactive props destructure for runtime props.
///
/// Vue treats array/object destructure defaults as per-instance factories, while
/// function defaults are already factories/values and must not be wrapped.
pub(crate) fn normalize_destructure_default_value(default_value: &str) -> String {
    let trimmed = default_value.trim();
    if trimmed.starts_with('[') {
        let mut wrapped = String::with_capacity(trimmed.len() + 6);
        wrapped.push_str("() => ");
        wrapped.push_str(trimmed);
        return wrapped;
    }

    if trimmed.starts_with('{') {
        let mut wrapped = String::with_capacity(trimmed.len() + 8);
        wrapped.push_str("() => (");
        wrapped.push_str(trimmed);
        wrapped.push(')');
        return wrapped;
    }

    default_value.to_compact_string()
}

pub(crate) fn normalize_destructure_default_value_with_factory(
    default_value: &str,
    needs_factory: bool,
) -> String {
    let trimmed = default_value.trim();
    if needs_factory && !trimmed.starts_with(['[', '{']) {
        let mut wrapped = String::with_capacity(trimmed.len() + 8);
        wrapped.push_str("() => (");
        wrapped.push_str(trimmed);
        wrapped.push(')');
        wrapped
    } else {
        normalize_destructure_default_value(default_value)
    }
}

#[cfg(test)]
mod tests {
    use super::extract_with_defaults_values;

    #[test]
    fn extracts_static_object_defaults() {
        let values = extract_with_defaults_values(
            "withDefaults(defineProps<Props>(), { label: undefined, count: 1 })",
        );

        assert_eq!(
            values
                .static_values
                .get("label")
                .map(|value| value.as_str()),
            Some("undefined")
        );
        assert_eq!(
            values
                .static_values
                .get("count")
                .map(|value| value.as_str()),
            Some("1")
        );
        assert!(values.runtime_expression.is_none());
    }

    #[test]
    fn preserves_imported_defaults_for_runtime_merge() {
        let values = extract_with_defaults_values(
            "withDefaults(defineProps<Props>(), checkboxPropsDefaults)",
        );

        assert!(values.static_values.is_empty());
        assert_eq!(
            values.runtime_expression.as_deref(),
            Some("checkboxPropsDefaults")
        );
    }

    #[test]
    fn preserves_entire_object_when_a_spread_requires_runtime_evaluation() {
        let values = extract_with_defaults_values(
            "withDefaults(defineProps<Props>(), { label: undefined, ...sharedDefaults })",
        );

        assert!(values.static_values.is_empty());
        assert_eq!(
            values.runtime_expression.as_deref(),
            Some("{ label: undefined, ...sharedDefaults }")
        );
    }

    #[test]
    fn preserves_entire_object_when_a_computed_key_requires_runtime_evaluation() {
        let values = extract_with_defaults_values(
            "withDefaults(defineProps<Props>(), { [defaultKey]: undefined })",
        );

        assert!(values.static_values.is_empty());
        assert_eq!(
            values.runtime_expression.as_deref(),
            Some("{ [defaultKey]: undefined }")
        );
    }
}
