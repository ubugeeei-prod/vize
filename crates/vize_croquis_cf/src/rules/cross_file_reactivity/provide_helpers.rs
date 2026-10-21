use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, cstr};
use vize_croquis::BindingType;
use vize_croquis::ScopeData;
use vize_croquis::provide::ProvideKey;
use vize_croquis::reactivity::{ReactiveKind, ReactivityLossKind};

pub(crate) fn provided_value_reactive_kind(
    analysis: &vize_croquis::Croquis,
    value: &str,
) -> Option<ReactiveKind> {
    let value = value.trim();

    if let Some(source) = vize_croquis::facts::reactivity_sources(analysis)
        .into_iter()
        .find(|source| source.name.as_str() == value)
    {
        return Some(source.kind);
    }

    let callee = value
        .split_once('(')
        .map(|(callee, _)| callee.trim())
        .unwrap_or_default();

    match callee {
        "ref" => Some(ReactiveKind::Ref),
        "shallowRef" => Some(ReactiveKind::ShallowRef),
        "reactive" => Some(ReactiveKind::Reactive),
        "shallowReactive" => Some(ReactiveKind::ShallowReactive),
        "computed" => Some(ReactiveKind::Computed),
        "readonly" => Some(ReactiveKind::Readonly),
        "shallowReadonly" => Some(ReactiveKind::ShallowReadonly),
        "toRef" => Some(ReactiveKind::ToRef),
        "toRefs" => Some(ReactiveKind::ToRefs),
        _ => None,
    }
}

/// Whether a provided service keeps its state live through refs/reactive values.
///
/// Unknown call results stay reportable, except for Vue composables: `useXxx()`
/// is the established contract for returning a setup service. Object members
/// must each be a known reactive binding, a function, or a value that may be a
/// ref and has not already been proven to be a plain reactive snapshot.
pub(super) fn is_reactive_service(analysis: &vize_croquis::Croquis, value: &str) -> bool {
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, value, SourceType::ts()).parse_expression() else {
        return false;
    };

    is_reactive_service_expression(analysis, unwrap_expression(&expression))
}

fn unwrap_expression<'a>(mut expression: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        expression = match expression {
            Expression::TSAsExpression(wrapper) => &wrapper.expression,
            Expression::TSSatisfiesExpression(wrapper) => &wrapper.expression,
            Expression::TSNonNullExpression(wrapper) => &wrapper.expression,
            Expression::ParenthesizedExpression(wrapper) => &wrapper.expression,
            _ => break,
        };
    }
    expression
}

fn is_reactive_service_expression(
    analysis: &vize_croquis::Croquis,
    expression: &Expression<'_>,
) -> bool {
    match unwrap_expression(expression) {
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
    match unwrap_expression(value) {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            provided_value_reactive_kind(analysis, name).is_some()
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
