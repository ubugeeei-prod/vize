//! Borrowed, site-owned lexical refusal before any component-name fallback.

use vize_croquis::analysis::ComponentUsage;
use vize_croquis::facts::components::component_reference_name;
use vize_croquis::{Croquis, ScopeKind};

/// Exact-name lookups over lexical scope parents; never enumerate all visible
/// bindings or compare script-relative with template-relative byte offsets.
pub(crate) fn template_usage_is_shadowed(analysis: &Croquis, usage: &ComponentUsage) -> bool {
    let declared_name = component_reference_name(analysis, usage.name.as_str());
    let mut current = Some(usage.scope_id);
    for _ in 0..analysis.scopes.len() {
        let Some(scope) = current.and_then(|id| analysis.scopes.get_scope(id)) else {
            return false;
        };
        match scope.kind {
            ScopeKind::ScriptSetup
            | ScopeKind::NonScriptSetup
            | ScopeKind::Module
            | ScopeKind::Universal
            | ScopeKind::JsGlobalUniversal
            | ScopeKind::JsGlobalBrowser
            | ScopeKind::JsGlobalNode
            | ScopeKind::JsGlobalDeno
            | ScopeKind::JsGlobalBun
            | ScopeKind::VueGlobal
            | ScopeKind::ExternalModule => return false,
            _ => {}
        }
        if scope.get_binding(usage.name.as_str()).is_some()
            || declared_name.is_some_and(|name| scope.get_binding(name).is_some())
        {
            return true;
        }
        current = scope.parent();
    }
    // A malformed cyclic host scope graph cannot establish a root reference.
    current.is_some()
}
