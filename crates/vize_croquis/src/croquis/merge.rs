use super::Croquis;
use crate::scope::{ScopeData, ScopeKind};
use vize_carton::CompactString;

impl Croquis {
    /// Merge a regular `<script>` croquis into a `<script setup>` croquis.
    ///
    /// The receiver keeps precedence for setup-local data, while module-level
    /// facts from the regular script are retained for virtual TS, lint, and
    /// cross-file consumers.
    pub fn merge_plain_script(&mut self, plain: Self) {
        self.merge_plain_script_scopes(&plain);

        let plain_bindings = plain.bindings;
        self.bindings.is_script_setup |= plain_bindings.is_script_setup;
        for (name, binding_type) in plain_bindings.bindings {
            self.bindings.bindings.entry(name).or_insert(binding_type);
        }
        for (local, prop) in plain_bindings.props_aliases {
            self.bindings.props_aliases.entry(local).or_insert(prop);
        }

        self.reactivity.extend(plain.reactivity);
        self.race_conditions.extend(plain.race_conditions);
        self.provide_inject.extend(plain.provide_inject);
        self.setup_context.extend(plain.setup_context);
        self.types.merge_keep_existing(plain.types);
        self.type_exports.extend(plain.type_exports);
        self.import_statements.extend(plain.import_statements);
        self.re_exports.extend(plain.re_exports);
        self.component_registrations
            .extend(plain.component_registrations);

        for (name, span) in plain.binding_spans {
            self.binding_spans.entry(name).or_insert(span);
        }
    }

    fn merge_plain_script_scopes(&mut self, plain: &Self) {
        for plain_script_scope in plain
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::NonScriptSetup)
        {
            let ScopeData::NonScriptSetup(data) = plain_script_scope.data() else {
                continue;
            };
            self.scopes.enter_non_script_setup_scope(
                data.clone(),
                plain_script_scope.span.start,
                plain_script_scope.span.end,
            );

            for import_scope in plain.scopes.iter().filter(|scope| {
                scope.kind == ScopeKind::ExternalModule
                    && scope.parent() == Some(plain_script_scope.id)
            }) {
                let ScopeData::ExternalModule(import_data) = import_scope.data() else {
                    continue;
                };
                self.scopes.enter_external_module_scope(
                    import_data.clone(),
                    import_scope.span.start,
                    import_scope.span.end,
                );
                for (name, binding) in import_scope.bindings() {
                    self.scopes.add_binding(CompactString::new(name), *binding);
                }
                self.scopes.exit_scope();
            }
            self.scopes.exit_scope();
        }
    }
}
