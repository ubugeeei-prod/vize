//! What the template scope hands back to the setup scope.
//!
//! The template is a function nested in the setup scope, and some types only
//! exist inside it: the slots it infers, the props a generic component
//! forwards to its root, and the instances its component refs are
//! instantiated to. It returns the slots alone for as long as that is all
//! there is, and a record of all three once there is more.

use super::fallthrough::{FallthroughComponentScope, ForwardedRoots};
use crate::virtual_ts::scope::{REFS_RETURN_KEY, SLOTS_RETURN_KEY, instantiated_ref_starts};
use vize_carton::{String, append};
use vize_relief::RootNode;

/// The type of the template's own `$refs` and of the public instance's.
const DOLLAR_REFS_TYPE: &str = "__VizeDollarRefs";

#[derive(Default)]
pub(super) struct TemplateRecord {
    pub(super) forwarded: ForwardedRoots,
    /// Template-relative starts of the component refs the template instantiates.
    ref_starts: Vec<u32>,
    /// `inferTemplateDollarRefs`: the template's `$refs` is its ref registry.
    template_dollar_refs: bool,
    /// `inferComponentDollarRefs`: so is the public instance's.
    component_dollar_refs: bool,
}

impl TemplateRecord {
    /// `template_ast` is the template when its scope is checked, `generic`
    /// whether the props are read back from a generic setup scope, and `script`
    /// the script that may name `useTemplateRef`.
    pub(super) fn plan(
        scope: &FallthroughComponentScope<'_>,
        template_ast: Option<&RootNode<'_>>,
        generic: bool,
        script: Option<&str>,
    ) -> Self {
        let registers_refs = super::setup_helpers::registers_template_refs(script, scope.checks);
        Self {
            forwarded: ForwardedRoots::plan(scope, template_ast, generic),
            ref_starts: if registers_refs && template_ast.is_some() && scope.checks.check_props {
                instantiated_ref_starts(scope.summary)
            } else {
                Vec::new()
            },
            template_dollar_refs: scope.checks.infer_template_dollar_refs,
            component_dollar_refs: scope.checks.infer_component_dollar_refs,
        }
    }

    /// A record that instantiates the component refs at `ref_starts`.
    #[cfg(test)]
    pub(super) fn instantiating(ref_starts: Vec<u32>) -> Self {
        Self {
            ref_starts,
            ..Self::default()
        }
    }

    /// Whether the template scope returns a record instead of its slots.
    pub(super) fn any(&self) -> bool {
        self.forwarded.any() || !self.ref_starts.is_empty()
    }

    pub(super) fn ref_starts(&self) -> &[u32] {
        &self.ref_starts
    }

    /// Whether the public instance holds the template's refs.
    pub(super) fn has_instance_refs(&self) -> bool {
        self.component_dollar_refs
    }

    /// Whether the ref registry types `$refs` at all.
    pub(super) fn types_dollar_refs(&self) -> bool {
        self.template_dollar_refs || self.component_dollar_refs
    }

    /// The type of the template's own `$refs`, when the registry types it.
    pub(super) fn template_refs_type(&self) -> Option<&'static str> {
        self.template_dollar_refs.then_some(DOLLAR_REFS_TYPE)
    }

    /// The captured template value, or the slots inside the record it is.
    pub(super) fn slots_value(&self) -> String {
        let mut value = String::from("__vize_template");
        if self.any() {
            append!(value, ".{SLOTS_RETURN_KEY}");
        }
        value
    }

    /// The instance a registry entry holds, given its declared one.
    pub(super) fn ref_instance(&self, start: u32, declared: &str, component: &str) -> String {
        if !self.ref_starts.contains(&start) {
            return declared.into();
        }
        let mut instance = String::default();
        append!(
            instance,
            "__VizeTemplateRefInstance<ReturnType<typeof __vize_template.{REFS_RETURN_KEY}[\"{start}\"]>, typeof {component}>"
        );
        instance
    }

    /// What the setup scope returns of its template: the root element it
    /// infers, the slots it infers, and the rest of this record.
    pub(super) fn push_template_return(
        &self,
        fields: &mut Vec<String>,
        inferred_slots: bool,
        has_root_el: bool,
    ) {
        if has_root_el {
            fields.push("__vize_root_el".into());
        }
        if inferred_slots {
            let mut slots = String::from("__vize_template_slots: ");
            slots.push_str(self.slots_value().as_str());
            fields.push(slots);
        }
        self.forwarded.push_return_field(fields);
        if self.component_dollar_refs {
            fields.push("__vize_refs".into());
        }
    }
}
