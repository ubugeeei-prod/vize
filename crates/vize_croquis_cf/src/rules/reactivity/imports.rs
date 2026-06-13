use vize_carton::FxHashSet;

pub(super) fn extract_vue_imports(analysis: &vize_croquis::Croquis) -> FxHashSet<&str> {
    use vize_croquis::ScopeKind;

    let mut vue_imports = FxHashSet::default();

    for scope in analysis.scopes.iter() {
        if scope.kind == ScopeKind::ExternalModule
            && let vize_croquis::ScopeData::ExternalModule(data) = scope.data()
        {
            // Check if this is a vue import
            if data.source.as_str() == "vue" || data.source.starts_with("vue/") {
                // Collect all bindings from this import
                for (name, _) in scope.bindings() {
                    vue_imports.insert(name);
                }
            }
        }
    }

    vue_imports
}
