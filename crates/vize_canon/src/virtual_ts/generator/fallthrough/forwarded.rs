//! What a generic component forwards to its component root.
//!
//! The non-generic surface is `__VizeComponentFallthroughProps<typeof Root>`,
//! a type of the root alone. A generic component instantiates its root with
//! its own type parameters, so its forwarded props are read from the template
//! scope instead (`scope::forwarded_roots`) and joined to the props every
//! instantiation of the component declares.

use super::{
    FallthroughComponentScope, FallthroughRootTarget,
    types::{fallthrough_targets, push_omitted_keys},
};
use crate::virtual_ts::{
    component_reference::resolved_component_binding_reference,
    scope::{FORWARDED_RETURN_KEY, FORWARDED_ROOT_HELPERS, SLOTS_RETURN_KEY},
};
use vize_carton::{String, append};
use vize_relief::RootNode;

/// The setup return field the forwarded props leave the setup scope through.
const SETUP_FIELD: &str = "__vize_forwarded";

#[derive(Default)]
pub(in crate::virtual_ts::generator) struct ForwardedRoots {
    /// Template-relative starts of the roots, the identity of their usages.
    starts: Vec<u32>,
    /// `checkRequiredFallthroughAttributes`: the props the roots bind
    /// themselves, which the parent is never asked for.
    required_except: Option<Vec<String>>,
}

impl ForwardedRoots {
    /// `template_ast` is the template when its scope is checked, and
    /// `instantiated` whether this component's props are read back from a
    /// generic setup scope.
    pub(in crate::virtual_ts::generator) fn plan(
        scope: &FallthroughComponentScope<'_>,
        template_ast: Option<&RootNode<'_>>,
        instantiated: bool,
    ) -> Self {
        let Some(template_ast) =
            template_ast.filter(|_| instantiated && scope.resolve_component_roots)
        else {
            return Self::default();
        };
        let mut starts = Vec::new();
        let mut authored_keys = Vec::new();
        for target in fallthrough_targets(scope.summary, template_ast).unwrap_or_default() {
            let FallthroughRootTarget::Component(root) = target else {
                continue;
            };
            if resolved_component_binding_reference(
                scope.summary,
                scope.options,
                scope.syntactic_type_only_imported_names,
                root.tag.as_str(),
            )
            .is_some()
            {
                starts.push(root.start);
                authored_keys.extend(root.authored_keys);
            }
        }
        Self {
            required_except: (scope.check_required && !starts.is_empty()).then_some(authored_keys),
            starts,
        }
    }

    pub(in crate::virtual_ts::generator) fn starts(&self) -> &[u32] {
        &self.starts
    }

    pub(in crate::virtual_ts::generator) fn any(&self) -> bool {
        !self.starts.is_empty()
    }

    /// The captured template value, or the slots inside it once the template
    /// scope returns the forwarded props next to them.
    pub(in crate::virtual_ts::generator) fn template_slots_value(&self) -> String {
        let mut value = String::from("__vize_template");
        if self.any() {
            append!(value, ".{SLOTS_RETURN_KEY}");
        }
        value
    }

    pub(in crate::virtual_ts::generator) fn push_return_field(&self, fields: &mut Vec<String>) {
        if self.any() {
            let mut field = String::default();
            append!(
                field,
                "{SETUP_FIELD}: __vize_template.{FORWARDED_RETURN_KEY}"
            );
            fields.push(field);
        }
    }

    pub(in crate::virtual_ts::generator) fn emit_helpers(&self, ts: &mut String) {
        if self.any() {
            ts.push_str(FORWARDED_ROOT_HELPERS);
        }
    }

    /// The forwarded props of one instantiation, as the tail of its `Props`.
    /// They are optional to the parent unless it is asked for the root's
    /// required ones; what the root binds itself is never the parent's.
    pub(in crate::virtual_ts::generator) fn push_props_tail(&self, ts: &mut String, setup: &str) {
        if !self.any() {
            return;
        }
        let mut forwarded = String::default();
        append!(forwarded, "{setup}[\"{SETUP_FIELD}\"]");
        ts.push_str(" & ");
        match self.required_except.as_deref() {
            None => append!(*ts, "Partial<{forwarded}>"),
            Some([]) => ts.push_str(forwarded.as_str()),
            Some(keys) => {
                append!(*ts, "Omit<{forwarded}, ");
                push_omitted_keys(ts, keys);
                ts.push('>');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FallthroughComponentScope, ForwardedRoots};
    use crate::virtual_ts::types::VirtualTsOptions;
    use vize_carton::{Allocator, String};
    use vize_croquis::{Analyzer, AnalyzerOptions};

    /// `(root starts, slots value, setup return fields, props tail)`.
    fn plan(
        template: &str,
        instantiated: bool,
        check_required: bool,
    ) -> (Vec<u32>, String, Vec<String>, String) {
        let allocator = Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer
            .analyze_script_setup("import Basic from './basic.vue'\ndefineProps<{ bar?: T }>()");
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let roots = ForwardedRoots::plan(
            &FallthroughComponentScope {
                summary: &summary,
                options: &VirtualTsOptions::default(),
                syntactic_type_only_imported_names: &Default::default(),
                resolve_component_roots: true,
                check_required,
            },
            Some(&root),
            instantiated,
        );
        let mut fields = Vec::new();
        roots.push_return_field(&mut fields);
        let mut tail = String::default();
        roots.push_props_tail(&mut tail, "Setup<T>");
        (
            roots.starts().to_vec(),
            roots.template_slots_value(),
            fields,
            tail,
        )
    }

    #[test]
    fn a_generic_component_reads_its_component_root_from_the_template() {
        assert_eq!(
            plan("<Basic :foo=\"bar\" @close=\"bar\" />", true, false),
            (
                vec![0],
                "__vize_template.__vizeSlots".into(),
                vec!["__vize_forwarded: __vize_template.__vizeForwarded".into()],
                " & Partial<Setup<T>[\"__vize_forwarded\"]>".into(),
            )
        );
    }

    #[test]
    fn required_forwarded_props_exclude_what_the_root_binds_itself() {
        assert_eq!(
            plan("<Basic :foo=\"bar\" @close=\"bar\" />", true, true).3,
            " & Omit<Setup<T>[\"__vize_forwarded\"], \"foo\" | \"onClose\">"
        );
        assert_eq!(
            plan("<Basic />", true, true).3,
            " & Setup<T>[\"__vize_forwarded\"]"
        );
    }

    #[test]
    fn every_other_root_keeps_the_declared_surface() {
        let declared = (
            Vec::new(),
            String::from("__vize_template"),
            Vec::new(),
            String::default(),
        );
        // A component that is not generic, a native root, a tag that is not a
        // setup binding, and a template that forwards nothing.
        assert_eq!(plan("<Basic :foo=\"bar\" />", false, false), declared);
        assert_eq!(plan("<div :id=\"bar\" />", true, false), declared);
        assert_eq!(plan("<Unknown :foo=\"bar\" />", true, false), declared);
        assert_eq!(plan("<Basic /><Basic />", true, false), declared);
    }
}
