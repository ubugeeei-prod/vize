use oxc_ast::ast::{CallExpression, Expression};

use crate::reactivity::ReactiveKind;
use crate::setup_context::SetupContextViolationKind;
use vize_carton::{CompactString, FxHashMap};
use vize_relief::BindingType;

use super::super::ScriptParseResult;

pub fn detect_reactivity_call(
    call: &CallExpression<'_>,
    reactivity_aliases: &FxHashMap<CompactString, CompactString>,
) -> Option<(ReactiveKind, BindingType)> {
    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return None,
    };

    // Resolve alias to original API name if needed
    let resolved_name = reactivity_aliases
        .get(callee_name)
        .map(|s| s.as_str())
        .unwrap_or(callee_name);

    match resolved_name {
        "ref" | "shallowRef" => Some((ReactiveKind::Ref, BindingType::SetupRef)),
        "computed" => Some((ReactiveKind::Computed, BindingType::SetupRef)),
        "reactive" | "shallowReactive" => {
            Some((ReactiveKind::Reactive, BindingType::SetupReactiveConst))
        }
        "toRef" => Some((ReactiveKind::ToRef, BindingType::SetupRef)),
        "toRefs" => Some((ReactiveKind::ToRefs, BindingType::SetupRef)),
        "customRef" => Some((ReactiveKind::Ref, BindingType::SetupRef)),
        // useTemplateRef returns a ShallowRef; the template accesses it via
        // `.value`, so the binding is a setup ref (matches @vue/compiler-sfc).
        "useTemplateRef" => Some((ReactiveKind::ShallowRef, BindingType::SetupRef)),
        "readonly" => Some((ReactiveKind::Readonly, BindingType::SetupReactiveConst)),
        "shallowReadonly" => Some((
            ReactiveKind::ShallowReadonly,
            BindingType::SetupReactiveConst,
        )),
        _ => None,
    }
}

/// Detect Vue API calls that violate setup context (CSRP/Memory Leak risks)
/// Returns true if a violation was detected and recorded
pub fn detect_setup_context_violation(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
) -> bool {
    // Only detect in non-setup scripts
    if !result.is_non_setup_script {
        return false;
    }

    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return false,
    };

    if let Some(kind) = SetupContextViolationKind::from_api_name(callee_name) {
        result.setup_context.record_violation(
            kind,
            CompactString::new(callee_name),
            call.span.start,
            call.span.end,
        );
        return true;
    }

    false
}
