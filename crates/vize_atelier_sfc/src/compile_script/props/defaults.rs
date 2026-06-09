//! Prop default value extraction and normalization.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::FxHashMap;
use vize_carton::{String, ToCompactString};

/// Extract default values from withDefaults second argument
/// Input: "withDefaults(defineProps<{...}>(), { prop1: default1, prop2: default2 })"
/// Returns: HashMap of prop name to default value string
pub fn extract_with_defaults_defaults(with_defaults_args: &str) -> FxHashMap<String, String> {
    let mut defaults = FxHashMap::default();
    let trimmed = with_defaults_args.trim();
    if trimmed.is_empty() {
        return defaults;
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
    if !parse_result.errors.is_empty() {
        return defaults;
    }

    let Some(Statement::VariableDeclaration(var_decl)) = parse_result.program.body.first() else {
        return defaults;
    };
    let Some(declarator) = var_decl.declarations.first() else {
        return defaults;
    };
    let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
        return defaults;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return defaults;
    };
    if callee.name.as_str() != "withDefaults" {
        return defaults;
    }

    let Some(Argument::ObjectExpression(obj)) = call.arguments.get(1) else {
        return defaults;
    };

    for property in obj.properties.iter() {
        let ObjectPropertyKind::ObjectProperty(prop) = property else {
            continue;
        };

        let key = match &prop.key {
            PropertyKey::StaticIdentifier(id) => id.name.to_compact_string(),
            PropertyKey::StringLiteral(lit) => lit.value.to_compact_string(),
            PropertyKey::NumericLiteral(lit) => lit.value.to_compact_string(),
            _ => continue,
        };

        let Some(value_start) = (prop.value.span().start as usize).checked_sub(WRAP_PREFIX.len())
        else {
            continue;
        };
        let Some(value_end) = (prop.value.span().end as usize).checked_sub(WRAP_PREFIX.len())
        else {
            continue;
        };
        if let Some(value_src) = trimmed.get(value_start..value_end) {
            defaults.insert(key, value_src.to_compact_string());
        }
    }

    defaults
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
