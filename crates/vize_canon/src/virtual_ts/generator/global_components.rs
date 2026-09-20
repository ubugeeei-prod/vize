use vize_carton::{CompactString, FxHashSet, String, append, camelize, capitalize};
use vize_croquis::{Croquis, ScopeData};

use crate::virtual_ts::component_reference::{
    component_reference_alias, contains_compact_name, has_type_only_component_candidate,
};
use crate::virtual_ts::helpers::to_safe_identifier;
use crate::virtual_ts::scope::{ComponentBindingCheck, GlobalComponentCheck};
use crate::virtual_ts::types::{VirtualTsCheckOptions, VirtualTsOptions, VizeMapping};

use super::imports::extract_declared_name;

pub(super) struct GlobalComponentDiagnostics<'a> {
    pub(super) mappings: &'a mut Vec<VizeMapping>,
    pub(super) template_offset: u32,
}

impl<'a> GlobalComponentDiagnostics<'a> {
    pub(super) fn new(
        options: VirtualTsCheckOptions,
        mappings: &'a mut Vec<VizeMapping>,
        template_offset: u32,
    ) -> Option<Self> {
        (options.check_unknown_components && options.check_template_bindings).then_some(Self {
            mappings,
            template_offset,
        })
    }
}

pub(super) struct GlobalComponentPlan<'a> {
    slot_component_names: FxHashSet<&'a str>,
    component_check: GlobalComponentCheck,
    self_component_name: Option<String>,
}

impl<'a> GlobalComponentPlan<'a> {
    pub(super) fn new(
        summary: &'a Croquis,
        legacy_vue2: bool,
        include_all: bool,
        self_component_name: Option<&str>,
    ) -> Self {
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
            self_component_name: summary
                .macros
                .define_options_name()
                .or(self_component_name)
                .map(|name| capitalize(&camelize(name))),
            // Vue 3 projects can contribute component types through ambient
            // `GlobalComponents` augmentation without an SFC-local
            // reference-types directive. Vue 2 retains the explicit-reference
            // requirement because its global component surface differs.
            component_check: if include_all {
                GlobalComponentCheck::All
            } else if !legacy_vue2 {
                GlobalComponentCheck::PascalCase
            } else {
                GlobalComponentCheck::None
            },
        }
    }

    pub(super) fn enabled(&self) -> bool {
        !matches!(self.component_check, GlobalComponentCheck::None)
            || !self.slot_component_names.is_empty()
            || self.self_component_name.is_some()
    }

    fn is_self_component(&self, name: &str) -> bool {
        self.component_check().is_self(name)
    }

    pub(super) fn component_check(&self) -> ComponentBindingCheck<'_> {
        ComponentBindingCheck {
            globals: self.component_check,
            self_component_name: self.self_component_name.as_deref(),
        }
    }

    pub(super) fn keeps_unresolved_binding(&self, name: &str) -> bool {
        self.component_check.allows(name)
            || self.slot_component_names.contains(name)
            || self.is_self_component(name)
    }

    pub(super) fn emit(
        &self,
        ts: &mut String,
        summary: &Croquis,
        options: &VirtualTsOptions,
        imported_names: &FxHashSet<&str>,
        syntactic_type_only_imported_names: &FxHashSet<CompactString>,
        mut diagnostics: Option<GlobalComponentDiagnostics<'_>>,
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
        for (index, usage) in summary.component_usages.iter().enumerate() {
            let name = usage.name.as_str();
            let is_self = self.is_self_component(name);
            if !self.component_check.allows(name)
                && !self.slot_component_names.contains(name)
                && !is_self
            {
                continue;
            }
            let camel_name = camelize(name);
            let pascal_name = capitalize(camel_name.as_str());
            let candidates = [name, camel_name.as_str(), pascal_name.as_str()];
            if candidates.iter().any(|candidate| {
                (summary.bindings.bindings.contains_key(*candidate)
                    && !contains_compact_name(syntactic_type_only_imported_names, candidate))
                    || (imported_names.contains(candidate)
                        && !contains_compact_name(syntactic_type_only_imported_names, candidate))
                    || (external_template_bindings.contains(candidate)
                        && !contains_compact_name(syntactic_type_only_imported_names, candidate))
                    || auto_import_stub_names.contains(candidate)
            }) {
                continue;
            }

            let component_ref =
                if has_type_only_component_candidate(syntactic_type_only_imported_names, name) {
                    component_reference_alias(name)
                } else {
                    to_safe_identifier(name)
                };
            if let Some(diagnostics) = diagnostics.as_mut().filter(|_| !is_self) {
                append!(*ts, "const {{ ");
                let start = ts.len();
                crate::virtual_ts::helpers::push_ts_string_literal(ts, name);
                let end = ts.len();
                append!(*ts, ": __vize_global_component_{index} }} = {{}} as (");
                for candidate in [pascal_name.as_str(), camel_name.as_str(), name] {
                    crate::virtual_ts::helpers::push_ts_string_literal(ts, candidate);
                    ts.push_str(" extends keyof import('vue').GlobalComponents ? { ");
                    crate::virtual_ts::helpers::push_ts_string_literal(ts, name);
                    ts.push_str(": unknown } : ");
                }
                append!(*ts, "{{}});\nvoid __vize_global_component_{index};\n");
                let source_start = (diagnostics.template_offset + usage.start + 1) as usize;
                diagnostics.mappings.push(VizeMapping {
                    gen_range: start..end,
                    src_range: source_start..source_start + name.len(),
                    sub_spans: Vec::new(),
                });
            }
            if !emitted_refs.insert(component_ref.clone()) {
                continue;
            }

            if !has_header {
                ts.push_str("\n// Global component stubs (vue module augmentations)\n");
                has_header = true;
            }

            if is_self {
                append!(
                    *ts,
                    "declare const {component_ref}: typeof __vize_component__;\n"
                );
            } else {
                append_global_component_stub(
                    ts,
                    component_ref.as_str(),
                    name,
                    pascal_name.as_str(),
                );
            }
        }
    }
}

fn append_global_component_stub(
    ts: &mut String,
    component_ref: &str,
    name: &str,
    pascal_name: &str,
) {
    append!(
        *ts,
        "declare const {component_ref}: import(\"vue\").GlobalComponents extends {{ \"{name}\": infer __C }} ? __C"
    );
    if pascal_name == name {
        ts.push_str(" : any;\n");
    } else {
        append!(
            *ts,
            " : import(\"vue\").GlobalComponents extends {{ \"{pascal_name}\": infer __C }} ? __C : any;\n"
        );
    }
}
