//! script/consistent-props-emits-key-quotes
//!
//! Require consistent quoting inside each `props` or `emits` declaration. When
//! one static key cannot be written as a JavaScript identifier and therefore
//! requires quotes, every other static key in that declaration must also be
//! quoted. Declarations whose keys can all be written without quotes are left
//! unchanged.
//!
//! The rule covers runtime object declarations and TypeScript type literals in
//! `defineProps` / `defineEmits`, plus object-form Options API `props` / `emits`.
//! Computed keys and spreads are ignored because they have no authored static
//! key whose quote style can be made consistent.
//!
//! Formatters should preserve authored property quotes when this rule is in
//! use. Set `quoteProps` to `preserve` in Prettier, Oxfmt, or the Vize
//! formatter so declaration-specific lint policy remains authoritative.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const emit = defineEmits<{
//!   "update:modelValue": [value: string]
//!   close: []
//! }>()
//! ```
//!
//! ### Valid
//! ```ts
//! const emit = defineEmits<{
//!   "update:modelValue": [value: string]
//!   "close": []
//! }>()
//! const props = defineProps({
//!   "aria-label": String,
//!   "disabled": Boolean,
//! })
//! ```

use oxc_ast::ast::{
    CallExpression, Expression, ObjectExpression, ObjectPropertyKind, Program, PropertyKey,
    TSSignature, TSType, TSTypeLiteral,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_identifier_name;

use super::super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use super::emits_source::resolve_emits_object;
use super::props_source::{PropsSource, collect_options_props};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/consistent-props-emits-key-quotes",
    description: "Require consistent quoted keys in props and emits declarations",
    default_severity: Severity::Error,
};

/// Require all static keys to be quoted when one declaration key requires it.
pub struct ConsistentPropsEmitsKeyQuotes;

impl ScriptRule for ConsistentPropsEmitsKeyQuotes {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if let Some(PropsSource::Object(props)) = collect_options_props(program) {
            check_object(props, "props", offset, result);
        }
        if let Some(emits) = resolve_emits_object(program) {
            check_object(emits, "emits", offset, result);
        }

        let mut visitor = MacroVisitor {
            offset,
            result,
            props_shadowed: super::compiler_macro_is_shadowed(program, "defineProps"),
            emits_shadowed: super::compiler_macro_is_shadowed(program, "defineEmits"),
        };
        visitor.visit_program(program);
    }
}

struct MacroVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    props_shadowed: bool,
    emits_shadowed: bool,
}

impl<'a> Visit<'a> for MacroVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let Expression::Identifier(callee) = &call.callee else {
            walk_call_expression(self, call);
            return;
        };
        let declaration = match callee.name.as_str() {
            "defineProps" if !self.props_shadowed => Some("props"),
            "defineEmits" if !self.emits_shadowed => Some("emits"),
            _ => None,
        };
        if let Some(declaration) = declaration {
            self.check_macro(call, declaration);
        }
        walk_call_expression(self, call);
    }
}

impl MacroVisitor<'_> {
    fn check_macro(&mut self, call: &CallExpression<'_>, declaration: &'static str) {
        if let Some(type_arguments) = &call.type_arguments {
            for argument in &type_arguments.params {
                if let TSType::TSTypeLiteral(literal) = argument {
                    check_type_literal(literal, declaration, self.offset, self.result);
                }
            }
        }

        let Some(object) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .map(Expression::get_inner_expression)
            .and_then(|expression| match expression {
                Expression::ObjectExpression(object) => Some(object.as_ref()),
                _ => None,
            })
        else {
            return;
        };
        check_object(object, declaration, self.offset, self.result);
    }
}

#[derive(Clone, Copy)]
struct DeclarationKey {
    span: Span,
    quoted: bool,
    requires_quotes: bool,
}

fn check_object(
    object: &ObjectExpression<'_>,
    declaration: &'static str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let keys = object
        .properties
        .iter()
        .filter_map(|entry| {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                return None;
            };
            (!property.computed)
                .then(|| declaration_key(&property.key))
                .flatten()
        })
        .collect::<Vec<_>>();
    check_keys(&keys, declaration, offset, result);
}

fn check_type_literal(
    literal: &TSTypeLiteral<'_>,
    declaration: &'static str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let keys = literal
        .members
        .iter()
        .filter_map(|member| match member {
            TSSignature::TSPropertySignature(property) if !property.computed => {
                declaration_key(&property.key)
            }
            TSSignature::TSMethodSignature(method) if !method.computed => {
                declaration_key(&method.key)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    check_keys(&keys, declaration, offset, result);
}

fn declaration_key(key: &PropertyKey<'_>) -> Option<DeclarationKey> {
    match key {
        PropertyKey::StaticIdentifier(_)
        | PropertyKey::Identifier(_)
        | PropertyKey::NumericLiteral(_)
        | PropertyKey::BooleanLiteral(_)
        | PropertyKey::BigIntLiteral(_)
        | PropertyKey::NullLiteral(_) => Some(DeclarationKey {
            span: key.span(),
            quoted: false,
            requires_quotes: false,
        }),
        PropertyKey::StringLiteral(literal) => Some(DeclarationKey {
            span: literal.span,
            quoted: true,
            requires_quotes: !is_identifier_name(literal.value.as_str()),
        }),
        _ => None,
    }
}

fn check_keys(
    keys: &[DeclarationKey],
    declaration: &'static str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    if !keys.iter().any(|key| key.requires_quotes) {
        return;
    }

    for key in keys.iter().filter(|key| !key.quoted) {
        let start = offset as u32 + key.span.start;
        let end = offset as u32 + key.span.end;
        result.add_diagnostic(
            LintDiagnostic::error(
                META.name,
                vize_l0::cstr!(
                    "Quote every {declaration} key because this declaration contains a key that requires quotes."
                ),
                start,
                end,
            )
            .with_label("quote this key for consistency", start, end)
            .with_help(vize_l0::cstr!(
                "Wrap this key in quotes so all {declaration} keys use the same quoted form."
            )),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ConsistentPropsEmitsKeyQuotes;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ConsistentPropsEmitsKeyQuotes));
        linter.lint(source, 0)
    }

    #[test]
    fn reports_unquoted_typed_emit_key() {
        let source = r#"defineEmits<{
  "update:modelValue": [value: string]
  close: []
}>()"#;
        let result = lint(source);
        assert_eq!(result.error_count, 1);
        assert_eq!(
            Some(result.diagnostics[0].start as usize),
            source.find("close")
        );
    }

    #[test]
    fn accepts_all_quoted_typed_emit_keys() {
        let result = lint(
            r#"defineEmits<{
  "update:modelValue": [value: string]
  "close": []
}>()"#,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn reports_unquoted_typed_prop_key() {
        let result = lint(r#"defineProps<{ "aria-label": string; disabled: boolean }>()"#);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_runtime_macro_keys() {
        let props = lint(r#"defineProps({ "aria-label": String, disabled: Boolean })"#);
        assert_eq!(props.error_count, 1);

        let emits = lint(r#"defineEmits({ "update:modelValue": null, close: null })"#);
        assert_eq!(emits.error_count, 1);
    }

    #[test]
    fn reports_unquoted_numeric_static_keys() {
        let result = lint(r#"defineProps({ "aria-label": String, 1: String })"#);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_options_api_props_and_emits_independently() {
        let result = lint(
            r#"export default {
  props: { "aria-label": String, disabled: Boolean },
  emits: { "update:modelValue": null, close: null },
}"#,
        );
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn does_not_force_quotes_without_a_key_that_requires_them() {
        let result = lint(
            r#"defineProps<{ title: string; "disabled": boolean }>()
defineEmits({ close: null, submit: null })"#,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn keeps_declarations_scoped_separately() {
        let result = lint(
            r#"defineProps<{ "aria-label": string; "disabled": boolean }>()
defineEmits<{ close: []; submit: [] }>()"#,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_computed_keys_and_spreads() {
        let result = lint(
            r#"const base = { disabled: Boolean }
defineProps({ ...base, [dynamic]: String, title: String })"#,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_shadowed_compiler_macros() {
        let result = lint(
            r#"function defineProps(value: unknown) { return value }
defineProps({ "aria-label": String, disabled: Boolean })"#,
        );
        assert_eq!(result.error_count, 0);
    }
}
