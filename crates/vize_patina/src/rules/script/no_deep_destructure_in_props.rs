//! script/no-deep-destructure-in-props
//!
//! Disallow deeply nested destructuring in defineProps.
//!
//! Deep destructuring patterns like `const { a: { b = 1 }} = defineProps()`
//! are hard to read, prone to runtime errors, and make it difficult to
//! understand the prop structure at a glance.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // Deep nested destructuring
//! const { user: { name, age } } = defineProps<{ user: User }>()
//!
//! // Very deep nesting
//! const { config: { settings: { theme } } } = defineProps()
//! ```
//!
//! ### Valid
//! ```ts
//! // Simple destructuring (one level)
//! const { name, count = 0 } = defineProps<{ name: string; count?: number }>()
//!
//! // Access nested properties in the component instead
//! const props = defineProps<{ user: User }>()
//! const userName = computed(() => props.user.name)
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, Program, VariableDeclarator};
use oxc_ast_visit::{Visit, walk::walk_variable_declarator};
use oxc_span::Span;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deep-destructure-in-props",
    description: "Disallow deeply nested destructuring in defineProps",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "Avoid deeply nested destructuring in defineProps";
const HELP: &str =
    "Use simple destructuring and access nested properties via computed or direct prop access";

/// Disallow deep destructuring in defineProps
pub struct NoDeepDestructureInProps {
    /// Maximum allowed nesting depth (default: 1)
    pub max_depth: usize,
}

impl Default for NoDeepDestructureInProps {
    fn default() -> Self {
        Self { max_depth: 1 }
    }
}

impl ScriptRule for NoDeepDestructureInProps {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !may_contain_define_props_object_binding(source) {
            return;
        }

        let mut visitor = NoDeepDestructureInPropsVisitor {
            max_depth: self.max_depth,
            offset,
            result,
        };
        visitor.visit_program(program);
    }
}

struct NoDeepDestructureInPropsVisitor<'result> {
    max_depth: usize,
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoDeepDestructureInPropsVisitor<'_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init
            && is_define_props_call(init)
            && let Some(span) = deep_object_pattern_span(&it.id, self.max_depth)
        {
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(META.name, MESSAGE, start, end).with_help(HELP),
            );
        }
        walk_variable_declarator(self, it);
    }
}

fn deep_object_pattern_span(pattern: &BindingPattern<'_>, max_depth: usize) -> Option<Span> {
    match pattern {
        BindingPattern::ObjectPattern(object) => {
            (object_pattern_depth(pattern, 0) > max_depth).then_some(object.span)
        }
        _ => None,
    }
}

fn object_pattern_depth(pattern: &BindingPattern<'_>, current_depth: usize) -> usize {
    match pattern {
        BindingPattern::ObjectPattern(object) => {
            let depth = current_depth + 1;
            let property_depth = object
                .properties
                .iter()
                .map(|property| object_pattern_depth(&property.value, depth))
                .max()
                .unwrap_or(depth);
            let rest_depth = object
                .rest
                .as_ref()
                .map(|rest| object_pattern_depth(&rest.argument, depth))
                .unwrap_or(depth);
            property_depth.max(rest_depth)
        }
        BindingPattern::ArrayPattern(array) => {
            let element_depth = array
                .elements
                .iter()
                .flatten()
                .map(|element| object_pattern_depth(element, current_depth))
                .max()
                .unwrap_or(current_depth);
            let rest_depth = array
                .rest
                .as_ref()
                .map(|rest| object_pattern_depth(&rest.argument, current_depth))
                .unwrap_or(current_depth);
            element_depth.max(rest_depth)
        }
        BindingPattern::AssignmentPattern(assignment) => {
            object_pattern_depth(&assignment.left, current_depth)
        }
        BindingPattern::BindingIdentifier(_) => current_depth,
    }
}

fn is_define_props_call(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression else {
        return false;
    };
    if call_is_named(call, "defineProps") {
        return true;
    }
    if call_is_named(call, "withDefaults")
        && let Some(first) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
    {
        return is_define_props_call(first);
    }
    false
}

fn call_is_named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == name
    )
}

fn may_contain_define_props_object_binding(source: &str) -> bool {
    source.contains("defineProps") && contains_object_binding_declaration(source)
}

fn contains_object_binding_declaration(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let Some(keyword) = declaration_keyword_at(source, index) else {
            index += 1;
            continue;
        };

        let cursor = skip_javascript_trivia(bytes, index + keyword.len());

        if bytes.get(cursor) == Some(&b'{') {
            return true;
        }

        index = cursor;
    }

    false
}

fn declaration_keyword_at(source: &str, index: usize) -> Option<&'static str> {
    ["const", "let", "var"].into_iter().find(|keyword| {
        source[index..].starts_with(keyword) && has_keyword_boundaries(source, index, keyword)
    })
}

fn skip_javascript_trivia(bytes: &[u8], mut cursor: usize) -> usize {
    loop {
        while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            cursor += 1;
        }

        match (bytes.get(cursor), bytes.get(cursor + 1)) {
            (Some(b'/'), Some(b'/')) => {
                cursor += 2;
                while !matches!(bytes.get(cursor), None | Some(b'\n' | b'\r')) {
                    cursor += 1;
                }
            }
            (Some(b'/'), Some(b'*')) => {
                cursor += 2;
                while !matches!(
                    (bytes.get(cursor), bytes.get(cursor + 1)),
                    (None, _) | (Some(b'*'), Some(b'/'))
                ) {
                    cursor += 1;
                }
                if bytes.get(cursor).is_some() {
                    cursor += 2;
                }
            }
            _ => return cursor,
        }
    }
}

fn has_keyword_boundaries(source: &str, index: usize, keyword: &str) -> bool {
    let bytes = source.as_bytes();
    let before = index.checked_sub(1).and_then(|before| bytes.get(before));
    let after = bytes.get(index + keyword.len());

    before.is_none_or(|byte| !is_identifier_byte(*byte))
        && after.is_none_or(|byte| !is_identifier_byte(*byte))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::NoDeepDestructureInProps;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeepDestructureInProps::default()));
        linter
    }

    #[test]
    fn test_valid_simple_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            "const { name, count = 0 } = defineProps<{ name: string }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_destructure() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps<{ name: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_deep_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            "const { user: { name } } = defineProps<{ user: User }>()",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_very_deep_destructure() {
        let linter = create_linter();
        let result = linter.lint(
            "const /* props */ { config: { settings: { theme } } } = defineProps()",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_deep_destructure_with_defaults() {
        let linter = create_linter();
        let result = linter.lint(
            "const { user: { name } } = withDefaults(defineProps<{ user: User }>(), {})",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_with_higher_max_depth() {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeepDestructureInProps { max_depth: 2 }));
        let result = linter.lint(
            "const { user: { name } } = defineProps<{ user: User }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_define_props_string_not_matched() {
        let linter = create_linter();
        let result = linter.lint(
            r#"const text = "const { user: { name } } = defineProps()""#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_unrelated_deep_destructure_not_matched() {
        let linter = create_linter();
        let result = linter.lint(
            "const { user: { name } } = createProps<{ user: User }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }
}
