use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, cstr};
use vize_croquis::BindingType;
use vize_croquis::ScopeData;
use vize_croquis::provide::ProvideKey;
use vize_croquis::reactivity::{ReactiveKind, ReactivityLossKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ProvidedValueClassification {
    pub(super) reactive_kind: Option<ReactiveKind>,
    pub(super) is_reactive_service: bool,
}

fn named_reactive_kind(analysis: &vize_croquis::Croquis, value: &str) -> Option<ReactiveKind> {
    vize_croquis::facts::reactivity_sources(analysis)
        .into_iter()
        .find(|source| source.name.as_str() == value)
        .map(|source| source.kind)
}

pub(crate) fn provided_value_reactive_kind(
    analysis: &vize_croquis::Croquis,
    value: &str,
) -> Option<ReactiveKind> {
    let value = value.trim();
    let callee = value.split_once('(').map(|(callee, _)| callee.trim());
    let plain_name = |name: &str| {
        !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_alphanumeric() || matches!(character, '_' | '$'))
    };
    // The index only needs the kind. Ordinary unresolved names/calls and
    // literal values keep their fact-only path; service collection owns its
    // existing AST parse. Wrapped/generic expressions need one fallback AST.
    let literal = value
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_digit() || matches!(byte, b'\'' | b'"' | b'`' | b'{'));
    if value.is_empty() || plain_name(value) || callee.is_some_and(plain_name) || literal {
        return named_reactive_kind(analysis, value)
            .or_else(|| callee.and_then(ReactiveKind::from_name));
    }
    classify_provided_value(analysis, value).reactive_kind
}

pub(super) fn classify_provided_value(
    analysis: &vize_croquis::Croquis,
    value: &str,
) -> ProvidedValueClassification {
    let value = value.trim();
    // Preserve the existing direct binding and direct call fast paths.
    let direct_kind = named_reactive_kind(analysis, value).or_else(|| {
        value
            .split_once('(')
            .and_then(|(callee, _)| ReactiveKind::from_name(callee.trim()))
    });
    if let Some(kind) = direct_kind {
        return ProvidedValueClassification {
            reactive_kind: Some(kind),
            is_reactive_service: false,
        };
    }

    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, value, SourceType::ts()).parse_expression() else {
        return ProvidedValueClassification::default();
    };
    let expression = expression.get_inner_expression();
    let reactive_kind = match expression {
        Expression::Identifier(identifier) => {
            named_reactive_kind(analysis, identifier.name.as_str())
        }
        Expression::CallExpression(call) => match call.callee.get_inner_expression() {
            Expression::Identifier(callee) => ReactiveKind::from_name(callee.name.as_str()),
            _ => None,
        },
        _ => None,
    };
    ProvidedValueClassification {
        reactive_kind,
        is_reactive_service: is_reactive_service_expression(analysis, expression),
    }
}

#[cfg(test)]
fn is_reactive_service(analysis: &vize_croquis::Croquis, value: &str) -> bool {
    classify_provided_value(analysis, value).is_reactive_service
}

fn is_reactive_service_expression(
    analysis: &vize_croquis::Croquis,
    expression: &Expression<'_>,
) -> bool {
    match expression.get_inner_expression() {
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(callee) => {
                vize_croquis::call_graph::is_composable_name(callee.name.as_str())
            }
            _ => false,
        },
        Expression::ObjectExpression(object) => {
            !object.properties.is_empty()
                && object.properties.iter().all(|property| {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return false;
                    };
                    is_live_service_member(analysis, &property.value)
                })
        }
        _ => false,
    }
}

fn is_live_service_member(analysis: &vize_croquis::Croquis, value: &Expression<'_>) -> bool {
    match value.get_inner_expression() {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            named_reactive_kind(analysis, name).is_some()
                || is_named_function(analysis, name)
                || matches!(
                    analysis.get_binding_type(name),
                    Some(BindingType::SetupRef | BindingType::SetupReactiveConst)
                )
                || (analysis.get_binding_type(name) == Some(BindingType::SetupMaybeRef)
                    && !is_known_plain_snapshot(analysis, name))
        }
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(callee) => {
                ReactiveKind::from_name(callee.name.as_str()).is_some()
                    || vize_croquis::call_graph::is_composable_name(callee.name.as_str())
            }
            _ => false,
        },
        _ => false,
    }
}

fn is_named_function(analysis: &vize_croquis::Croquis, name: &str) -> bool {
    analysis.scopes.iter().any(|scope| {
        matches!(
            scope.data(),
            ScopeData::Closure(data) if data.name.as_deref() == Some(name)
        )
    })
}

fn is_known_plain_snapshot(analysis: &vize_croquis::Croquis, name: &str) -> bool {
    vize_croquis::facts::reactivity_losses(analysis)
        .into_iter()
        .any(|loss| match loss.kind {
            ReactivityLossKind::ReactiveDestructure {
                destructured_props, ..
            }
            | ReactivityLossKind::RefValueDestructure {
                destructured_props, ..
            }
            | ReactivityLossKind::PropsDestructure { destructured_props } => {
                destructured_props.iter().any(|property| property == name)
            }
            ReactivityLossKind::RefValueExtract { target_name, .. }
            | ReactivityLossKind::ReactivePropertyExtract { target_name, .. }
            | ReactivityLossKind::GetterCallExtract { target_name, .. } => target_name == name,
            ReactivityLossKind::PlainValueAlias {
                alias_name,
                target_name,
                ..
            } => alias_name == name || target_name == name,
            ReactivityLossKind::FunctionArgumentExtract { .. }
            | ReactivityLossKind::ReactiveSpread { .. }
            | ReactivityLossKind::ReactiveReassign { .. } => false,
        })
}

pub(super) fn provide_key_display(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) | ProvideKey::Symbol(s) => s.clone(),
    }
}

pub(super) fn provide_key_identity(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) => cstr!("string:{s}"),
        ProvideKey::Symbol(s) => cstr!("symbol:{s}"),
    }
}

#[cfg(test)]
mod tests {
    use super::is_reactive_service;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    fn analyze(source: &str) -> vize_croquis::Croquis {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(source);
        analyzer.finish()
    }

    #[test]
    fn provided_value_wrappers_preserve_runtime_kinds_and_plain_negatives() {
        use super::{ProvidedValueClassification, classify_provided_value};
        use vize_croquis::reactivity::ReactiveKind;
        let analysis = analyze(
            "const source = ref(0); const value = computed(() => source.value); const state = reactive({count: 1}); const plain = 1; const snapshot = source.value;",
        );
        let cases = [
            ("value", Some(ReactiveKind::Computed), false),
            ("value as Ref<number>", Some(ReactiveKind::Computed), false),
            (
                "((value as unknown) as Ref<number>)",
                Some(ReactiveKind::Computed),
                false,
            ),
            (
                "value satisfies Ref<number>",
                Some(ReactiveKind::Computed),
                false,
            ),
            ("value!", Some(ReactiveKind::Computed), false),
            ("<Ref<number>>value", Some(ReactiveKind::Computed), false),
            ("source as Ref<number>", Some(ReactiveKind::Ref), false),
            ("state as typeof state", Some(ReactiveKind::Reactive), false),
            (
                "ref<number>(0) as Ref<number>",
                Some(ReactiveKind::Ref),
                false,
            ),
            (
                "(computed(() => source.value)) as Ref<number>",
                Some(ReactiveKind::Computed),
                false,
            ),
            ("(ref)(0) as Ref<number>", Some(ReactiveKind::Ref), false),
            (
                "readonly(source) as Ref<number>",
                Some(ReactiveKind::Readonly),
                false,
            ),
            ("1 as unknown as Ref<number>", None, false),
            ("plain as unknown as Ref<number>", None, false),
            ("({value: 1}) as unknown as Ref<number>", None, false),
            ("source.value as unknown as Ref<number>", None, false),
            ("snapshot as unknown as Ref<number>", None, false),
            ("getValue() as Ref<number>", None, false),
            (
                "(true ? source : plain) as unknown as Ref<number>",
                None,
                false,
            ),
            ("({source, update: () => {}}) as unknown", None, true),
            ("useService() as unknown", None, true),
            ("unresolved", None, false),
            ("(unresolved as unknown)!", None, false),
        ];
        let actual = cases.map(|(value, _, _)| classify_provided_value(&analysis, value));
        let expected =
            cases.map(
                |(_, reactive_kind, is_reactive_service)| ProvidedValueClassification {
                    reactive_kind,
                    is_reactive_service,
                },
            );
        assert_eq!(actual, expected);
        let actual_kinds =
            cases.map(|(value, _, _)| super::provided_value_reactive_kind(&analysis, value));
        assert_eq!(actual_kinds, cases.map(|(_, kind, _)| kind));
    }

    #[test]
    fn reactive_and_function_members_form_a_service() {
        let analysis =
            analyze("const state = ref(0); function update() {}; const snapshot = state.value");
        assert!(is_reactive_service(
            &analysis,
            "{ show: (message: string) => console.log(message), close() {} }"
        ));
        assert!(is_reactive_service(&analysis, "{ state, update }"));
        assert!(!is_reactive_service(&analysis, "{ state, snapshot }"));
        assert!(!is_reactive_service(
            &analysis,
            "{ show: () => {}, count: 0 }"
        ));
        assert!(!is_reactive_service(
            &analysis,
            "reactive({ show: () => {} })"
        ));
        assert!(!is_reactive_service(&analysis, "{}"));
    }

    #[test]
    fn composable_results_form_services_but_arbitrary_calls_do_not() {
        let analysis = analyze("const { state, open } = useToast()");
        assert!(is_reactive_service(&analysis, "{ state, open }"));
        assert!(is_reactive_service(&analysis, "useToast()"));
        assert!(!is_reactive_service(&analysis, "getToast()"));
    }
}
