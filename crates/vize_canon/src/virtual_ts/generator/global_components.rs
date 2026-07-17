use vize_carton::{FxHashSet, String, append, camelize, capitalize};
use vize_croquis::{Croquis, ScopeData};

use crate::virtual_ts::helpers::to_safe_identifier;
use crate::virtual_ts::types::VirtualTsOptions;

use super::imports::extract_declared_name;

pub(super) struct GlobalComponentPlan<'a> {
    slot_component_names: FxHashSet<&'a str>,
    include_all: bool,
}

impl<'a> GlobalComponentPlan<'a> {
    pub(super) fn new(summary: &'a Croquis, legacy_vue2: bool, include_all: bool) -> Self {
        let slot_component_names = if legacy_vue2 {
            FxHashSet::default()
        } else {
            summary
                .scopes
                .iter()
                .filter_map(|scope| match scope.data() {
                    ScopeData::VSlot(data) => data.component.as_deref(),
                    _ => None,
                })
                .collect()
        };
        Self {
            slot_component_names,
            include_all,
        }
    }

    pub(super) fn enabled(&self) -> bool {
        self.include_all || !self.slot_component_names.is_empty()
    }

    pub(super) fn keeps_unresolved_binding(&self, name: &str) -> bool {
        self.include_all || self.slot_component_names.contains(name)
    }

    pub(super) fn emit(
        &self,
        ts: &mut String,
        summary: &Croquis,
        options: &VirtualTsOptions,
        imported_names: &FxHashSet<&str>,
    ) {
        if !self.enabled() || summary.component_usages.is_empty() {
            return;
        }

        let external_template_bindings = options
            .external_template_bindings
            .iter()
            .map(|name| name.as_str())
            .collect::<FxHashSet<_>>();
        let auto_import_stub_names = options
            .auto_import_stubs
            .iter()
            .filter_map(|stub| extract_declared_name(stub))
            .collect::<FxHashSet<_>>();

        let mut emitted_refs = FxHashSet::default();
        let mut has_header = false;
        for usage in &summary.component_usages {
            let name = usage.name.as_str();
            if !self.include_all && !self.slot_component_names.contains(name) {
                continue;
            }
            let camel_name = camelize(name);
            let pascal_name = capitalize(camel_name.as_str());
            let candidates = [name, camel_name.as_str(), pascal_name.as_str()];
            if candidates.iter().any(|candidate| {
                summary.bindings.bindings.contains_key(*candidate)
                    || imported_names.contains(candidate)
                    || external_template_bindings.contains(candidate)
                    || auto_import_stub_names.contains(candidate)
            }) {
                continue;
            }

            let component_ref = to_safe_identifier(name);
            if !emitted_refs.insert(component_ref.clone()) {
                continue;
            }

            if !has_header {
                ts.push_str("\n// Global component stubs (vue module augmentations)\n");
                has_header = true;
            }

            append!(
                *ts,
                "declare const {component_ref}: import(\"vue\").GlobalComponents extends {{ \"{name}\": infer __C }} ? __C"
            );
            if pascal_name.as_str() == name {
                ts.push_str(" : any;\n");
            } else {
                append!(
                    *ts,
                    " : import(\"vue\").GlobalComponents extends {{ \"{pascal_name}\": infer __C }} ? __C : any;\n"
                );
            }
        }
    }
}
