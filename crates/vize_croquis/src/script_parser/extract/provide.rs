use oxc_ast::ast::{Argument, CallExpression, Expression, TSType};
use oxc_span::GetSpan;

use crate::provide::{InjectPattern, ProvideKey};
use vize_carton::{CompactString, String};

use super::super::ScriptParseResult;
use super::super::result::SetupEffectContext;

pub fn detect_provide_inject_call(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return,
    };

    // Check if this is a direct call or an alias call
    let is_provide = callee_name == "provide" || result.provide_aliases.contains(callee_name);
    let is_inject = callee_name == "inject" || result.inject_aliases.contains(callee_name);

    if is_provide {
        // Detect setup context violation for provide
        super::reactivity::detect_setup_context_violation(result, call);

        // provide(key, value)
        if call.arguments.len() >= 2
            && let Some(key_argument) = call.arguments.first()
        {
            let key = extract_provide_key(key_argument, source);
            let value = call
                .arguments
                .get(1)
                .map(|arg| match arg {
                    Argument::TSTypeAssertion(assertion) => String::from(
                        source
                            .get(assertion.span.start as usize..assertion.span.end as usize)
                            .unwrap_or(""),
                    ),
                    _ => extract_argument_source(arg, source),
                })
                .unwrap_or_default();
            let value_type = call
                .arguments
                .get(1)
                .and_then(|arg| extract_argument_type_annotation(arg, source));

            if let Some(key) = key {
                let from_composable = match result.setup_effect_context() {
                    SetupEffectContext::TopLevel => None,
                    SetupEffectContext::Composable(name) => Some(name),
                    SetupEffectContext::Deferred => return,
                };
                result.provide_inject.add_provide(
                    key,
                    CompactString::new(&value),
                    value_type,
                    from_composable,
                    call.span.start,
                    call.span.end,
                );
            }
        }
    } else if is_inject {
        // The binding-aware top-level declarator pass records richer local and
        // destructuring metadata first. The generic walker owns calls in a
        // function body, including composable modules.
        if result
            .provide_inject
            .has_inject_span(call.span.start, call.span.end)
        {
            return;
        }
        let Some(key) = call
            .arguments
            .first()
            .and_then(|argument| extract_provide_key(argument, source))
        else {
            return;
        };
        let default_value = call
            .arguments
            .get(1)
            .map(|argument| CompactString::new(extract_argument_source(argument, source)));
        let from_composable = match result.setup_effect_context() {
            SetupEffectContext::TopLevel => None,
            SetupEffectContext::Composable(name) => Some(name),
            SetupEffectContext::Deferred => return,
        };
        result.provide_inject.add_inject(
            key,
            CompactString::new("(unbound)"),
            default_value,
            extract_inject_expected_type(call, source),
            InjectPattern::Simple,
            from_composable,
            call.span.start,
            call.span.end,
        );
        if matches!(call.arguments.get(1), Some(Argument::SpreadElement(_))) {
            result.provide_inject.mark_last_inject_default_spread();
        }
    }
}

pub fn extract_inject_expected_type(
    call: &CallExpression<'_>,
    source: &str,
) -> Option<CompactString> {
    call.type_arguments
        .as_ref()
        .and_then(|type_params| type_params.params.first())
        .and_then(|ty| type_source(ty, source))
}

pub fn extract_inject_expected_type_from_init(
    init: Option<&Expression<'_>>,
    call: &CallExpression<'_>,
    source: &str,
) -> Option<CompactString> {
    init.and_then(|init| extract_expression_type_annotation(init, source))
        .or_else(|| extract_inject_expected_type(call, source))
}

pub fn extract_expression_type_annotation(
    expression: &Expression<'_>,
    source: &str,
) -> Option<CompactString> {
    match expression {
        Expression::TSAsExpression(ts_as) => type_source(&ts_as.type_annotation, source),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            type_source(&ts_satisfies.type_annotation, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_expression_type_annotation(&ts_non_null.expression, source)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            extract_expression_type_annotation(&parenthesized.expression, source)
        }
        _ => None,
    }
}

pub fn extract_argument_type_annotation(arg: &Argument<'_>, source: &str) -> Option<CompactString> {
    match arg {
        Argument::TSAsExpression(ts_as) => type_source(&ts_as.type_annotation, source),
        Argument::TSSatisfiesExpression(ts_satisfies) => {
            type_source(&ts_satisfies.type_annotation, source)
        }
        Argument::TSNonNullExpression(ts_non_null) => {
            extract_expression_type_annotation(&ts_non_null.expression, source)
        }
        Argument::ParenthesizedExpression(parenthesized) => {
            extract_expression_type_annotation(&parenthesized.expression, source)
        }
        _ => None,
    }
}

fn type_source(ty: &TSType<'_>, source: &str) -> Option<CompactString> {
    source
        .get(ty.span().start as usize..ty.span().end as usize)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(CompactString::new)
}

pub fn extract_provide_key(arg: &Argument<'_>, source: &str) -> Option<ProvideKey> {
    match arg {
        Argument::StringLiteral(s) => {
            Some(ProvideKey::String(CompactString::new(s.value.as_str())))
        }
        Argument::Identifier(id) => {
            // Could be a Symbol or a variable reference - treat as Symbol for now
            Some(ProvideKey::Symbol(CompactString::new(id.name.as_str())))
        }
        _ => {
            // For complex expressions, extract source as string key
            let expr_source = extract_argument_source(arg, source);
            if !expr_source.is_empty() {
                Some(ProvideKey::String(CompactString::new(&expr_source)))
            } else {
                None
            }
        }
    }
}

/// Extract source code of an argument
pub fn extract_argument_source(arg: &Argument<'_>, source: &str) -> String {
    let span = match arg {
        Argument::SpreadElement(s) => s.span,
        Argument::Identifier(id) => id.span,
        Argument::StringLiteral(s) => s.span,
        Argument::NumericLiteral(n) => n.span,
        Argument::BooleanLiteral(b) => b.span,
        Argument::NullLiteral(n) => n.span,
        Argument::ArrayExpression(a) => a.span,
        Argument::ObjectExpression(o) => o.span,
        Argument::FunctionExpression(f) => f.span,
        Argument::ArrowFunctionExpression(a) => a.span,
        Argument::CallExpression(c) => c.span,
        Argument::TSAsExpression(ts_as) => ts_as.span,
        Argument::TSSatisfiesExpression(ts_satisfies) => ts_satisfies.span,
        Argument::TSNonNullExpression(ts_non_null) => ts_non_null.span,
        Argument::ParenthesizedExpression(parenthesized) => parenthesized.span,
        _ => return String::default(),
    };
    String::from(
        source
            .get(span.start as usize..span.end as usize)
            .unwrap_or(""),
    )
}

#[cfg(test)]
mod source_tests {
    use crate::provide::{ProvideEntry, ProvideKey, ProviderId};
    use vize_carton::CompactString;

    #[test]
    fn typed_assertion_value_keeps_exact_provider_source_and_metadata() {
        const SOURCE: &str = r#"import { provide, ref } from 'vue';
import type { Ref } from 'vue';
const state = ref('a');
provide('plain', state);
provide('as', state as Ref<string>);
provide('angle', <Ref<string>>state);
"#;
        let mut analyzer = crate::Analyzer::with_options(crate::AnalyzerOptions::full());
        analyzer.analyze_script_setup(SOURCE);
        let croquis = analyzer.finish();
        let expected = [
            ProvideEntry {
                id: ProviderId::new(0),
                key: ProvideKey::String(CompactString::new("plain")),
                value: CompactString::new("state"),
                value_type: None,
                from_composable: None,
                start: 92,
                end: 115,
            },
            ProvideEntry {
                id: ProviderId::new(1),
                key: ProvideKey::String(CompactString::new("as")),
                value: CompactString::new("state as Ref<string>"),
                value_type: Some(CompactString::new("Ref<string>")),
                from_composable: None,
                start: 117,
                end: 152,
            },
            ProvideEntry {
                id: ProviderId::new(2),
                key: ProvideKey::String(CompactString::new("angle")),
                value: CompactString::new("<Ref<string>>state"),
                value_type: None,
                from_composable: None,
                start: 154,
                end: 190,
            },
        ];
        assert_eq!(
            crate::facts::provide_entries(&croquis).as_slice(),
            expected.as_slice()
        );
    }

    #[test]
    fn composables_own_injects_without_counting_deferred_helpers() {
        const SOURCE: &str = r#"import { inject } from 'vue'
export function useFormField() {
  const form = inject(FormKey, null)
}
export const useArrowField = () => {
  const form = inject(ArrowKey)
}
export const ordinaryArrow = () => {
  const form = inject(ArrowHelperKey)
}
export function ordinaryHelper() {
  const form = inject(HelperKey)
}
export function useDeferredField() {
  return () => inject(DeferredKey)
}
"#;
        let mut analyzer = crate::Analyzer::with_options(crate::AnalyzerOptions::full());
        analyzer.analyze_script_plain(SOURCE);
        let croquis = analyzer.finish();
        let injects = crate::facts::inject_entries(&croquis);

        assert_eq!(injects.len(), 2);
        assert_eq!(injects[0].from_composable.as_deref(), Some("useFormField"));
        assert_eq!(injects[1].from_composable.as_deref(), Some("useArrowField"));
    }

    #[test]
    fn setup_calls_keep_export_identity_and_ignore_deferred_calls() {
        const SOURCE: &str = r#"import { useFormField as bindField } from '../composables/form'
bindField(() => true)
function deferred() {
  bindField(() => false)
}
"#;
        let mut analyzer = crate::Analyzer::with_options(crate::AnalyzerOptions::full());
        analyzer.analyze_script_setup(SOURCE);
        let croquis = analyzer.finish();
        let calls = crate::facts::composable_calls(&croquis);
        let start = SOURCE.find("bindField(() => true)").unwrap() as u32;

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "useFormField");
        assert_eq!(calls[0].source, "../composables/form");
        assert_eq!(calls[0].start, start);
        assert_eq!(calls[0].end, start + "bindField(() => true)".len() as u32);
    }
}
