use super::Croquis;
use crate::scope::{ScopeData, ScopeKind};
use crate::script_parser::ModuleReadFacts;
use vize_carton::{CompactString, FxHashSet};
use vize_relief::BindingType;

impl Croquis {
    /// Merge a regular `<script>` croquis into a `<script setup>` croquis.
    ///
    /// The receiver keeps precedence for setup-local data, while module-level
    /// facts from the regular script are retained for virtual TS, lint, and
    /// cross-file consumers.
    pub fn merge_plain_script(&mut self, mut plain: Self) {
        if let Some(reads) = plain.setup_context.take_module_read_facts() {
            self.apply_module_reads(reads);
        }
        self.merge_plain_script_scopes(&plain);
        // Every script block must independently establish the closed proof.
        self.nuxt_static_render_script = Some(
            self.nuxt_static_render_script == Some(true)
                && plain.nuxt_static_render_script == Some(true),
        );

        let plain_bindings = plain.bindings;
        self.bindings.is_script_setup |= plain_bindings.is_script_setup;
        for (name, binding_type) in plain_bindings.bindings {
            self.bindings.bindings.entry(name).or_insert(binding_type);
        }
        for (local, prop) in plain_bindings.props_aliases {
            self.bindings.props_aliases.entry(local).or_insert(prop);
        }

        // `export default defineComponent({ inheritAttrs: false })` in the
        // regular script disables fallthrough for the whole component, not just
        // for its own block; the setup block cannot re-enable it.
        self.template_info.inherit_attrs_disabled |= plain.template_info.inherit_attrs_disabled;

        self.reactivity.extend(plain.reactivity);
        self.race_conditions.extend(plain.race_conditions);
        self.provide_inject.extend(plain.provide_inject);
        self.setup_context.extend(plain.setup_context);
        self.types.merge_keep_existing(plain.types);
        self.type_exports.extend(plain.type_exports);
        self.import_statements.extend(plain.import_statements);
        self.re_exports.extend(plain.re_exports);
        self.re_export_forwards.extend(plain.re_export_forwards);
        self.component_registrations
            .extend(plain.component_registrations);
        self.template_component_registrations
            .extend(plain.template_component_registrations);

        for (name, span) in plain.binding_spans {
            self.binding_spans.entry(name).or_insert(span);
        }
    }

    fn apply_module_reads(&mut self, reads: ModuleReadFacts) {
        let ModuleReadFacts::Known(reads) = reads else {
            // Invalid syntax or eval cannot establish absence of a module read.
            self.unused_bindings.clear();
            return;
        };
        if reads.is_empty() || self.unused_bindings.is_empty() {
            return;
        }
        // This runs before merging normal-script scopes. Only original setup
        // imports are hoisted into the shared module; setup-local values remain
        // private even if a normal-script identifier has the same spelling.
        let imports: FxHashSet<&str> = self
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::ExternalModule)
            .flat_map(|scope| scope.bindings())
            .filter_map(|(name, binding)| {
                (binding.binding_type != BindingType::ExternalModule).then_some(name)
            })
            .collect();
        self.unused_bindings
            .retain(|name| !imports.contains(name.as_str()) || !reads.contains(name));
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

#[cfg(test)]
mod tests {
    use super::Croquis;

    // `<script>` + `<script setup>` analyse as two croquis; the regular
    // script's `inheritAttrs: false` must survive the merge into the setup one.
    #[test]
    fn plain_script_inherit_attrs_false_survives_the_merge() {
        let mut plain = Croquis::new();
        plain.template_info.inherit_attrs_disabled = true;
        let mut setup = Croquis::new();
        assert!(!setup.template_info.inherit_attrs_disabled);

        setup.merge_plain_script(plain);

        assert!(setup.template_info.inherit_attrs_disabled);

        let mut setup = Croquis::new();
        setup.template_info.inherit_attrs_disabled = true;
        setup.merge_plain_script(Croquis::new());
        assert!(setup.template_info.inherit_attrs_disabled);
    }
}
