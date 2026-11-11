//! `<script setup>` compiler macros as eslint-plugin-vue's
//! `defineScriptSetupVisitor` recognizes them.
//!
//! A macro is a call with a plain identifier callee that is the whole
//! expression of a top-level statement or the whole initializer of a top-level
//! declarator. `defineProps` may also sit as the first argument of such a
//! `withDefaults(...)`. A member call (`form.defineModel()`), a call nested in
//! another expression and a cast (`defineModel() as Ref`) are ordinary calls.

use oxc_ast::ast::{Argument, BindingPattern, CallExpression, Expression, Program, Statement};
use vize_l0::{CompactString, ToCompactString};

/// One top-level macro call.
pub(crate) struct SetupMacro<'b, 'a> {
    pub(crate) name: &'b str,
    pub(crate) call: &'b CallExpression<'a>,
    /// The declarator binding, for `const x = macro()`.
    pub(crate) binding: Option<&'b BindingPattern<'a>>,
}

/// Every top-level macro call of `program`, in source order.
pub(crate) fn setup_macros<'b, 'a>(program: &'b Program<'a>) -> Vec<SetupMacro<'b, 'a>> {
    let mut macros = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExpressionStatement(statement) => {
                push_macro(&statement.expression, None, &mut macros);
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if let Some(init) = &declarator.init {
                        push_macro(init, Some(&declarator.id), &mut macros);
                    }
                }
            }
            _ => {}
        }
    }
    macros
}

fn push_macro<'b, 'a>(
    candidate: &'b Expression<'a>,
    binding: Option<&'b BindingPattern<'a>>,
    macros: &mut Vec<SetupMacro<'b, 'a>>,
) {
    let Expression::CallExpression(call) = candidate.without_parentheses() else {
        return;
    };
    let Some(name) = identifier_callee(call) else {
        return;
    };
    if name == "withDefaults" {
        if let Some(Argument::CallExpression(inner)) = call.arguments.first()
            && identifier_callee(inner) == Some("defineProps")
        {
            macros.push(SetupMacro {
                name: "defineProps",
                call: inner,
                binding,
            });
        }
        return;
    }
    macros.push(SetupMacro {
        name,
        call,
        binding,
    });
}

fn identifier_callee<'b>(call: &'b CallExpression<'_>) -> Option<&'b str> {
    match &call.callee {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

/// A `defineModel(...)` call read as upstream's `getComponentModelFromDefineModel`.
pub(crate) struct DefineModel<'b, 'a> {
    /// `modelValue` unless the first argument is a literal.
    pub(crate) name: CompactString,
    /// The options argument (after the name, when there is one).
    pub(crate) options: Option<&'b Expression<'a>>,
}

impl<'b, 'a> DefineModel<'b, 'a> {
    pub(crate) fn of(call: &'b CallExpression<'a>) -> Self {
        let argument = |index: usize| call.arguments.get(index).and_then(Argument::as_expression);
        let literal = argument(0).and_then(|first| literal_name(skip_ts_as(first)));
        let (name, options) = match literal {
            Some(name) => (name, argument(1)),
            None => (CompactString::new("modelValue"), argument(0)),
        };
        Self { name, options }
    }
}

/// Upstream's `skipTSAsExpression`; ESTree has no parenthesized node.
pub(crate) fn skip_ts_as<'b, 'a>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression.without_parentheses() {
        Expression::TSAsExpression(cast) => skip_ts_as(&cast.expression),
        expression => expression,
    }
}

/// `String(value)` of an ESTree `Literal` (not a template literal).
fn literal_name(expression: &Expression<'_>) -> Option<CompactString> {
    let name = match expression {
        Expression::StringLiteral(literal) => literal.value.to_compact_string(),
        Expression::NumericLiteral(literal) => literal.value.to_compact_string(),
        Expression::BooleanLiteral(literal) => literal.value.to_compact_string(),
        Expression::NullLiteral(_) => CompactString::new("null"),
        Expression::BigIntLiteral(literal) => literal.value.to_compact_string(),
        _ => return None,
    };
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;

    fn names(source: &str) -> Vec<String> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, super::super::script_source_type()).parse();
        setup_macros(&parsed.program)
            .iter()
            .map(|found| match found.name {
                "defineModel" => format!("model:{}", DefineModel::of(found.call).name),
                name => name.to_owned(),
            })
            .collect()
    }

    #[test]
    fn recognizes_only_top_level_identifier_calls() {
        assert_eq!(
            names(
                "const a = defineModel('a'); defineModel(); const p = withDefaults(defineProps<{ x?: 1 }>(), {});\n\
                 form.defineModel(); const c = defineModel() as any; function f() { defineModel('n') }\n\
                 const t = defineModel(`t`); const s = defineModel(('s' as string))"
            ),
            [
                "model:a",
                "model:modelValue",
                "defineProps",
                "model:modelValue",
                "model:s"
            ]
        );
    }
}
