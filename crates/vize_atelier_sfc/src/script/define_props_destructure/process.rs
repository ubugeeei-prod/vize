//! Processing of props destructure patterns.
//!
//! Extracts destructured prop bindings from an `ObjectPattern` AST node,
//! handling aliases, defaults, and rest spread patterns.

use oxc_ast::ast::{BindingPattern, ObjectPattern};
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

                        result.bindings.insert(
                            key.clone(),
                            PropsDestructureBinding {
                                local: local.clone(),
                                default: Some(default_expr.to_compact_string()),
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

                    result.bindings.insert(
                        key.clone(),
                        PropsDestructureBinding {
                            local: local.clone(),
                            default: None,
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
    if let Some(rest) = &pattern.rest {
        if let BindingPattern::BindingIdentifier(id) = &rest.argument {
            let rest_name = id.name.to_compact_string();
            result.rest_id = Some(rest_name.clone());
            binding_metadata.insert(rest_name, BindingType::SetupReactiveConst);
        }
    }

    (result, binding_metadata, props_aliases)
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
