//! Processing of props destructure patterns.
//!
//! Extracts destructured prop bindings from an `ObjectPattern` AST node,
//! handling aliases, defaults, and rest spread patterns.

use oxc_ast::ast::{BindingPattern, Expression, ObjectPattern};
use oxc_span::GetSpan;
use vize_carton::FxHashMap;

use crate::types::BindingType;

use super::{PropsDestructureBinding, PropsDestructuredBindings};
use vize_carton::{String, ToCompactString};

/// Process props destructure from an ObjectPattern
pub fn process_props_destructure(
    pattern: &ObjectPattern<'_>,
    source: &str,
) -> (
    PropsDestructuredBindings,
    FxHashMap<String, BindingType>,
    FxHashMap<String, String>,
) {
    let mut result = PropsDestructuredBindings::default();
    let mut binding_metadata: FxHashMap<String, BindingType> = FxHashMap::default();
    let mut props_aliases: FxHashMap<String, String> = FxHashMap::default();

    for prop in pattern.properties.iter() {
        let key = resolve_object_key(&prop.key, source);

        if let Some(key) = key {
            match &prop.value {
                // Default value: { foo = 123 }
                BindingPattern::AssignmentPattern(assign) => {
                    if let BindingPattern::BindingIdentifier(id) = &assign.left {
                        let local = id.name.to_compact_string();
                        let default_expr = &source
                            [assign.right.span().start as usize..assign.right.span().end as usize];
                        let (needs_factory, skip_factory) = classify_default_value(&assign.right);

                        result.keys.push(key.clone());
                        result.bindings.insert(
                            key.clone(),
                            PropsDestructureBinding {
                                local: local.clone(),
                                default: Some(default_expr.to_compact_string()),
                                default_needs_factory: needs_factory,
                                default_skip_factory: skip_factory,
                            },
                        );

                        // If local name differs from key, it's an alias
                        if local != key {
                            binding_metadata.insert(local.clone(), BindingType::PropsAliased);
                            props_aliases.insert(local, key);
                        } else {
                            // Same name - it's a prop
                            binding_metadata.insert(local.clone(), BindingType::Props);
                        }
                    }
                }
                // Simple destructure: { foo } or { foo: bar }
                BindingPattern::BindingIdentifier(id) => {
                    let local = id.name.to_compact_string();

                    result.keys.push(key.clone());
                    result.bindings.insert(
                        key.clone(),
                        PropsDestructureBinding {
                            local: local.clone(),
                            default: None,
                            default_needs_factory: false,
                            default_skip_factory: false,
                        },
                    );

                    // If local name differs from key, it's an alias
                    if local != key {
                        binding_metadata.insert(local.clone(), BindingType::PropsAliased);
                        props_aliases.insert(local, key);
                    } else {
                        // Same name - it's a prop
                        binding_metadata.insert(local.clone(), BindingType::Props);
                    }
                }
                _ => {
                    // Nested patterns not supported
                }
            }
        }
    }

    // Handle rest spread: { ...rest }
    if let Some(rest) = &pattern.rest
        && let BindingPattern::BindingIdentifier(id) = &rest.argument
    {
        let rest_name = id.name.to_compact_string();
        result.rest_id = Some(rest_name.clone());
        binding_metadata.insert(rest_name, BindingType::SetupReactiveConst);
    }

    (result, binding_metadata, props_aliases)
}

/// Unwrap TS-only wrapper expressions (`x as T`, `x satisfies T`, `<T>x`, `x!`,
/// `(x)`) to reach the underlying value expression.
fn unwrap_ts_node<'a, 'b>(expr: &'b Expression<'a>) -> &'b Expression<'a> {
    match expr {
        Expression::TSAsExpression(e) => unwrap_ts_node(&e.expression),
        Expression::TSSatisfiesExpression(e) => unwrap_ts_node(&e.expression),
        Expression::TSNonNullExpression(e) => unwrap_ts_node(&e.expression),
        Expression::TSTypeAssertion(e) => unwrap_ts_node(&e.expression),
        Expression::ParenthesizedExpression(e) => unwrap_ts_node(&e.expression),
        other => other,
    }
}

/// Classify a runtime destructure default value, mirroring Vue's
/// `genDestructuredDefaultValue` for the runtime (non-typed) declaration path.
///
/// Returns `(needs_factory, skip_factory)`:
/// - `skip_factory`: default is a function or bare identifier -> emit
///   `__skip_<key>: true`, do not factory-wrap.
/// - `needs_factory`: default is a non-literal, non-function, non-identifier
///   expression -> wrap as `() => (...)`.
fn classify_default_value(expr: &Expression<'_>) -> (bool, bool) {
    let unwrapped = unwrap_ts_node(expr);
    let is_function = matches!(
        unwrapped,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    );
    let is_identifier = matches!(unwrapped, Expression::Identifier(_));
    let is_literal = matches!(
        unwrapped,
        Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
    );
    let skip_factory = is_function || is_identifier;
    let needs_factory = !skip_factory && !is_literal;
    (needs_factory, skip_factory)
}

/// Resolve object key to string
fn resolve_object_key(key: &oxc_ast::ast::PropertyKey<'_>, _source: &str) -> Option<String> {
    match key {
        oxc_ast::ast::PropertyKey::StaticIdentifier(id) => Some(id.name.to_compact_string()),
        oxc_ast::ast::PropertyKey::StringLiteral(lit) => Some(lit.value.to_compact_string()),
        oxc_ast::ast::PropertyKey::NumericLiteral(lit) => Some(lit.value.to_compact_string()),
        _ => None, // Computed keys not supported
    }
}
