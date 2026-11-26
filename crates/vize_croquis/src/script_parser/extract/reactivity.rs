use oxc_ast::ast::{CallExpression, Expression};

use crate::reactivity::ReactiveKind;
use crate::scope::ScopeKind;
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
        "ref" => Some((ReactiveKind::Ref, BindingType::SetupRef)),
        "shallowRef" => Some((ReactiveKind::ShallowRef, BindingType::SetupRef)),
        "computed" => Some((ReactiveKind::Computed, BindingType::SetupRef)),
        "reactive" => Some((ReactiveKind::Reactive, BindingType::SetupReactiveConst)),
        "shallowReactive" => Some((
            ReactiveKind::ShallowReactive,
            BindingType::SetupReactiveConst,
        )),
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

/// Modules whose exports are Vue's runtime API.
fn is_vue_api_source(source: &str) -> bool {
    matches!(
        source,
        "vue" | "#imports" | "@vue/runtime-core" | "@vue/runtime-dom"
    )
}

/// The Vue API a module-scope callee names, by the origin of its binding: an
/// import from a Vue module names the export it imports, under any local
/// name; a local alias of an API (`const r = ref`) names that API; an unbound
/// name is the auto-imported API of that spelling (Nuxt, unplugin-auto-import).
/// An import from any other module, or any other local declaration, names
/// no Vue API.
pub(crate) fn callee_vue_api<'r>(
    result: &'r ScriptParseResult,
    callee: &'r str,
) -> Option<&'r str> {
    let definitions = result.types.definitions();
    if let Some(source) = definitions.imported_types.get(callee) {
        return is_vue_api_source(source)
            .then(|| definitions.imported_type_export(callee))
            .flatten();
    }
    if let Some(api) = result.reactivity_aliases.get(callee) {
        return Some(api.as_str());
    }
    (!result.bindings.contains(callee)).then_some(callee)
}

/// Whether `call` is Vue's `useTemplateRef(key)`. Its result is wired to a
/// template ref by the key string, never by the name of the binding that
/// holds it.
pub fn is_template_ref_call(result: &ScriptParseResult, call: &CallExpression<'_>) -> bool {
    let Expression::Identifier(callee) = &call.callee else {
        return false;
    };
    callee_vue_api(result, callee.name.as_str()) == Some("useTemplateRef")
}

/// Record `name` as a template ref handle when it is initialized by Vue's
/// `useTemplateRef(key)`.
pub fn record_template_ref_handle(
    result: &mut ScriptParseResult,
    name: &str,
    init: Option<&Expression<'_>>,
) {
    if init
        .and_then(super::common::extract_call_expression)
        .is_some_and(|call| is_template_ref_call(result, call))
    {
        result.template_ref_handles.insert(CompactString::new(name));
    }
}

/// Detect Vue API calls that violate setup context (CSRP/Memory Leak risks)
/// Returns true if a violation was detected and recorded
pub fn detect_setup_context_violation(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
) -> bool {
    // Only record calls made directly in a plain script's module scope.
    if !result.is_non_setup_script
        || result.scopes.current_scope().kind != ScopeKind::NonScriptSetup
    {
        return false;
    }

    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return false,
    };

    // `import { useTemplateRef } from './helpers'` is not Vue's API.
    if let Some(kind) =
        callee_vue_api(result, callee_name).and_then(SetupContextViolationKind::from_api_name)
    {
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
