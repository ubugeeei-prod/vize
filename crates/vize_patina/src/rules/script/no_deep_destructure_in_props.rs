//! script/no-deep-destructure-in-props
//!
//! Disallow deeply nested destructuring in defineProps.
//!
//! Checks top-level compiler-owned bindings in a sole inline `<script setup>`.
//! Both object and array patterns contribute nesting depth. This remains a
//! strict-preset depth policy, not a compiler-error check: compiler-disabled reactive
//! destructuring and `withDefaults` can compile while still receiving a warning.
//! Ordinary scripts and unavailable/local/foreign macro ownership are ignored.
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
use oxc_ast::ast::{BindingPattern, CallExpression, Expression, Program, Statement};
use oxc_span::Span;

use super::props_emits::compiler_macro_is_shadowed;
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};

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

    fn as_setup_props_destructure(&self) -> Option<&NoDeepDestructureInProps> {
        Some(self)
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if sfc.is_sfc && sfc.is_script_setup && sfc.sole_script_block {
            self.check_setup(program, source, offset, result);
        }
    }
}

impl NoDeepDestructureInProps {
    /// The builtin caller separately proves a sole inline setup descriptor.
    pub(crate) fn check_setup(
        &self,
        program: &Program<'_>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !may_contain_define_props_object_binding(source) {
            return;
        }

        let mut macros = MacroOwnership::default();
        for statement in &program.body {
            let Statement::VariableDeclaration(declaration) = statement else {
                continue;
            };
            if declaration.declare {
                continue;
            }
            for declarator in &declaration.declarations {
                if let Some(span) = deep_object_pattern_span(&declarator.id, self.max_depth)
                    && let Some(init) = &declarator.init
                    && is_define_props_call(init, program, &mut macros)
                {
                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            MESSAGE,
                            offset as u32 + span.start,
                            offset as u32 + span.end,
                        )
                        .with_help(HELP),
                    );
                }
            }
        }
    }
}

fn deep_object_pattern_span(pattern: &BindingPattern<'_>, max_depth: usize) -> Option<Span> {
    match pattern {
        BindingPattern::ObjectPattern(object) => {
            (binding_pattern_depth(pattern, 0) > max_depth).then_some(object.span)
        }
        _ => None,
    }
}

fn binding_pattern_depth(pattern: &BindingPattern<'_>, current_depth: usize) -> usize {
    match pattern {
        BindingPattern::ObjectPattern(object) => {
            let depth = current_depth + 1;
            let property_depth = object
                .properties
                .iter()
                .map(|property| binding_pattern_depth(&property.value, depth))
                .max()
                .unwrap_or(depth);
            let rest_depth = object
                .rest
                .as_ref()
                .map(|rest| binding_pattern_depth(&rest.argument, depth))
                .unwrap_or(depth);
            property_depth.max(rest_depth)
        }
        BindingPattern::ArrayPattern(array) => {
            let depth = current_depth + 1;
            let element_depth = array
                .elements
                .iter()
                .flatten()
                .map(|element| binding_pattern_depth(element, depth))
                .max()
                .unwrap_or(depth);
            let rest_depth = array
                .rest
                .as_ref()
                .map(|rest| binding_pattern_depth(&rest.argument, depth))
                .unwrap_or(depth);
            element_depth.max(rest_depth)
        }
        BindingPattern::AssignmentPattern(assignment) => {
            binding_pattern_depth(&assignment.left, current_depth)
        }
        BindingPattern::BindingIdentifier(_) => current_depth,
    }
}

#[derive(Default)]
struct MacroOwnership {
    define_props: Option<bool>,
    with_defaults: Option<bool>,
}

fn is_define_props_call(
    expression: &Expression<'_>,
    program: &Program<'_>,
    macros: &mut MacroOwnership,
) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    if call.optional {
        return false;
    }
    if call_is_named(call, "defineProps") {
        return *macros
            .define_props
            .get_or_insert_with(|| !compiler_macro_is_shadowed(program, "defineProps"));
    }
    if call_is_named(call, "withDefaults")
        && *macros
            .with_defaults
            .get_or_insert_with(|| !compiler_macro_is_shadowed(program, "withDefaults"))
        && let Some(first) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
    {
        return is_define_props_call(first, program, macros);
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
        source.get(index..).unwrap_or_default().starts_with(keyword)
            && has_keyword_boundaries(source, index, keyword)
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
mod tests;
