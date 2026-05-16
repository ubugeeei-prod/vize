//! Analyze bindings in normal `<script>` blocks.
//!
//! Based on Vue.js official implementation:
//! https://github.com/vuejs/core/blob/main/packages/compiler-sfc/src/script/analyzeScriptBindings.ts
//!
//! Note that `compileScriptSetup` already analyzes bindings as part of its
//! compilation process so this should only be used on single `<script>` SFCs.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

use vize_carton::{CompactString, ToCompactString};

use crate::types::{BindingMetadata, BindingType};

/// Analyze bindings in normal `<script>` block
///
/// This analyzes the default export object to extract binding information
/// from props, inject, computed, methods, setup, and data options.
pub fn analyze_script_bindings(source: &str) -> BindingMetadata {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();

    let ret = Parser::new(&allocator, source, source_type).parse();

    if ret.panicked {
        return BindingMetadata::default();
    }

    for stmt in ret.program.body.iter() {
        if let Statement::ExportDefaultDeclaration(export) = stmt {
            if let ExportDefaultDeclarationKind::ObjectExpression(obj) = &export.declaration {
                return analyze_bindings_from_options(obj, source);
            }
        }
    }

    BindingMetadata::default()
}

/// Analyze bindings from options object (e.g., export default { ... })
fn analyze_bindings_from_options(node: &ObjectExpression<'_>, source: &str) -> BindingMetadata {
    let mut bindings = BindingMetadata::default();

    // Mark as non-script-setup so we don't resolve components/directives from these
    bindings.is_script_setup = false;

    for property in node.properties.iter() {
        match property {
            ObjectPropertyKind::ObjectProperty(prop) => {
                if prop.computed {
                    continue;
                }

                let key_name = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.to_compact_string(),
                    _ => continue,
                };

                match key_name.as_str() {
                    // props: ['foo'] or props: { foo: ... }
                    "props" => {
                        for key in get_object_or_array_expression_keys(&prop.value, source) {
                            bindings.bindings.insert(key, BindingType::Props);
                        }
                    }
                    // inject: ['foo'] or inject: { foo: {} }
                    "inject" => {
                        for key in get_object_or_array_expression_keys(&prop.value, source) {
                            bindings.bindings.insert(key, BindingType::Options);
                        }
                    }
                    // computed: { foo() {} } or methods: { foo() {} }
                    "computed" | "methods" => {
                        if let Expression::ObjectExpression(obj) = &prop.value {
                            for key in get_object_expression_keys(obj, source) {
                                bindings.bindings.insert(key, BindingType::Options);
                            }
                        }
                    }
                    _ => {}
                }
            }
            ObjectPropertyKind::SpreadProperty(_) => {
                // Skip spread properties
            }
        }
    }

    // Handle setup() and data() methods
    for property in node.properties.iter() {
        if let ObjectPropertyKind::ObjectProperty(prop) = property {
            if let Expression::FunctionExpression(func) = &prop.value {
                let key_name = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.to_compact_string(),
                    _ => continue,
                };

                if key_name == "setup" || key_name == "data" {
                    // Look for return statements in the function body
                    for stmt in func
                        .body
                        .as_ref()
                        .map(|b| b.statements.iter())
                        .into_iter()
                        .flatten()
                    {
                        if let Statement::ReturnStatement(ret) = stmt {
                            if let Some(Expression::ObjectExpression(obj)) = &ret.argument {
                                for key in get_object_expression_keys(obj, source) {
                                    let binding_type = if key_name == "setup" {
                                        BindingType::SetupMaybeRef
                                    } else {
                                        BindingType::Data
                                    };
                                    bindings.bindings.insert(key, binding_type);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    bindings
}

/// Get keys from an object expression
fn get_object_expression_keys(node: &ObjectExpression<'_>, source: &str) -> Vec<CompactString> {
    let mut keys = Vec::new();

    for prop in node.properties.iter() {
        match prop {
            ObjectPropertyKind::ObjectProperty(p) => {
                if let Some(key) = resolve_object_key(&p.key, p.computed, source) {
                    keys.push(key);
                }
            }
            ObjectPropertyKind::SpreadProperty(_) => {
                // Skip spread properties
            }
        }
    }

    keys
}

/// Get keys from an array expression (string literals only)
fn get_array_expression_keys(node: &ArrayExpression<'_>) -> Vec<CompactString> {
    let mut keys = Vec::new();

    for element in node.elements.iter() {
        if let ArrayExpressionElement::StringLiteral(s) = element {
            keys.push(s.value.to_compact_string());
        }
    }

    keys
}

/// Get keys from either an object or array expression
pub fn get_object_or_array_expression_keys(
    value: &Expression<'_>,
    source: &str,
) -> Vec<CompactString> {
    match value {
        Expression::ArrayExpression(arr) => get_array_expression_keys(arr),
        Expression::ObjectExpression(obj) => get_object_expression_keys(obj, source),
        _ => Vec::new(),
    }
}

/// Resolve object key to a string
fn resolve_object_key(
    key: &PropertyKey<'_>,
    computed: bool,
    _source: &str,
) -> Option<CompactString> {
    if computed {
        // For computed keys, we'd need to evaluate the expression
        // For now, just return None for computed keys
        return None;
    }

    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.to_compact_string()),
        PropertyKey::StringLiteral(s) => Some(s.value.to_compact_string()),
        PropertyKey::NumericLiteral(n) => Some(n.value.to_compact_string()),
        PropertyKey::PrivateIdentifier(_) => None,
        _ => {
            // For other expression types, try to extract from source
            // This is a simplified implementation
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_script_bindings, BindingType};

    #[test]
    fn test_analyze_props_array() {
        let source = r#"
            export default {
                props: ['foo', 'bar']
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(bindings.bindings.get("foo"), Some(&BindingType::Props));
        assert_eq!(bindings.bindings.get("bar"), Some(&BindingType::Props));
    }

    #[test]
    fn test_analyze_props_object() {
        let source = r#"
            export default {
                props: {
                    foo: String,
                    bar: { type: Number, default: 0 }
                }
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(bindings.bindings.get("foo"), Some(&BindingType::Props));
        assert_eq!(bindings.bindings.get("bar"), Some(&BindingType::Props));
    }

    #[test]
    fn test_analyze_inject() {
        let source = r#"
            export default {
                inject: ['service', 'store']
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(
            bindings.bindings.get("service"),
            Some(&BindingType::Options)
        );
        assert_eq!(bindings.bindings.get("store"), Some(&BindingType::Options));
    }

    #[test]
    fn test_analyze_computed() {
        let source = r#"
            export default {
                computed: {
                    doubled() { return this.count * 2 },
                    triple: function() { return this.count * 3 }
                }
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(
            bindings.bindings.get("doubled"),
            Some(&BindingType::Options)
        );
        assert_eq!(bindings.bindings.get("triple"), Some(&BindingType::Options));
    }

    #[test]
    fn test_analyze_methods() {
        let source = r#"
            export default {
                methods: {
                    handleClick() {},
                    handleSubmit: function() {}
                }
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(
            bindings.bindings.get("handleClick"),
            Some(&BindingType::Options)
        );
        assert_eq!(
            bindings.bindings.get("handleSubmit"),
            Some(&BindingType::Options)
        );
    }

    #[test]
    fn test_analyze_data() {
        let source = r#"
            export default {
                data: function() {
                    return {
                        count: 0,
                        name: 'hello'
                    }
                }
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(bindings.bindings.get("count"), Some(&BindingType::Data));
        assert_eq!(bindings.bindings.get("name"), Some(&BindingType::Data));
    }

    #[test]
    fn test_analyze_setup() {
        let source = r#"
            export default {
                setup: function() {
                    return {
                        count: ref(0),
                        doubled: computed(() => count.value * 2)
                    }
                }
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert_eq!(
            bindings.bindings.get("count"),
            Some(&BindingType::SetupMaybeRef)
        );
        assert_eq!(
            bindings.bindings.get("doubled"),
            Some(&BindingType::SetupMaybeRef)
        );
    }

    #[test]
    fn test_is_not_script_setup() {
        let source = r#"
            export default {
                props: ['foo']
            }
        "#;
        let bindings = analyze_script_bindings(source);
        assert!(!bindings.is_script_setup);
    }

    #[test]
    fn test_empty_export() {
        let source = r#"
            export default {}
        "#;
        let bindings = analyze_script_bindings(source);
        assert!(bindings.bindings.is_empty());
    }

    #[test]
    fn test_no_default_export() {
        let source = r#"
            const foo = 'bar'
        "#;
        let bindings = analyze_script_bindings(source);
        assert!(bindings.bindings.is_empty());
    }
}
