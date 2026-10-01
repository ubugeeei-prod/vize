use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, cstr};
use vize_croquis::provide::ProvideKey;
use vize_croquis::reactivity::ReactiveKind;

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

/// A service object whose direct properties are functions has no reactive
/// properties to lose when a consumer destructures it.
pub(super) fn is_function_only_object(value: &str) -> bool {
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, value, SourceType::ts()).parse_expression() else {
        return false;
    };
    let mut expression = &expression;
    loop {
        expression = match expression {
            Expression::TSAsExpression(wrapper) => &wrapper.expression,
            Expression::TSSatisfiesExpression(wrapper) => &wrapper.expression,
            Expression::TSNonNullExpression(wrapper) => &wrapper.expression,
            Expression::ParenthesizedExpression(wrapper) => &wrapper.expression,
            _ => break,
        };
    }
    let Expression::ObjectExpression(object) = expression else {
        return false;
    };
    !object.properties.is_empty()
        && object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            matches!(
                &property.value,
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
            )
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
    use super::is_function_only_object;

    #[test]
    fn only_function_valued_object_properties_form_a_service() {
        assert!(is_function_only_object(
            "{ show: (message: string) => console.log(message), close() {} }"
        ));
        assert!(!is_function_only_object("{ show: () => {}, count: 0 }"));
        assert!(!is_function_only_object("reactive({ show: () => {} })"));
        assert!(!is_function_only_object("{}"));
    }
}
