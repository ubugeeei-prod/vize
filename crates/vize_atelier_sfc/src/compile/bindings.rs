//! Binding metadata conversion and registration for SFC compilation.
//!
//! Registers compiler-specific setup bindings and the bindings of normal
//! `<script>` blocks.

mod options;
mod ordinary;
mod vapor;
pub(super) use options::options_api;
pub(super) use ordinary::ordinary;
pub(super) use vapor::for_vapor;

use oxc_ast::ast::{Program, Statement};

use crate::types::{BindingMetadata, BindingType};

/// Preserve the script compiler's static enum classification in the template
/// metadata. The setup facts record runtime enum names as setup constants; the
/// SFC compiler also knows which declarations can be hoisted as literal
/// constants.
pub(super) fn register_setup_enum_bindings(bindings: &mut BindingMetadata, program: &Program<'_>) {
    for statement in &program.body {
        if let Statement::TSEnumDeclaration(enumeration) = statement
            && !enumeration.declare
        {
            crate::script::register_enum(bindings, enumeration);
        }
    }
}

/// Merge normal script bindings without changing the target metadata flags.
pub(super) fn merge_normal_script_bindings(
    target: &mut BindingMetadata,
    normal_bindings: &BindingMetadata,
) {
    for (name, binding_type) in &normal_bindings.bindings {
        target.bindings.entry(name.clone()).or_insert(*binding_type);
    }
}

/// Register the `defineEmits` binding visible to the render scope.
///
/// Returns whether `$emit` was synthesized for an inline render and therefore
/// must stay out of the authored setup-state return.
pub(super) fn register_setup_emit_binding(
    bindings: &mut BindingMetadata,
    binding_name: Option<&str>,
    separates_template: bool,
) -> bool {
    if let Some(binding_name) = binding_name {
        bindings
            .bindings
            .entry(binding_name.into())
            .or_insert(BindingType::SetupConst);
        return false;
    }
    if separates_template || bindings.bindings.contains_key("$emit") {
        return false;
    }
    bindings
        .bindings
        .insert("$emit".into(), BindingType::SetupConst);
    true
}

/// Copy render bindings into setup state without an inline-only `$emit` alias.
pub(super) fn copy_setup_state_bindings(
    target: &mut BindingMetadata,
    render_bindings: &BindingMetadata,
    synthetic_inline_dollar_emit: bool,
) {
    *target = render_bindings.clone();
    if synthetic_inline_dollar_emit {
        target.bindings.remove("$emit");
    }
}

#[cfg(test)]
mod tests {
    use super::{merge_normal_script_bindings, ordinary};
    use crate::types::{BindingMetadata, BindingType};

    #[test]
    fn normal_script_imports_preserve_vue_ref_boundaries() {
        let source = r#"
import { label, onPing as handler } from "./store";
import defaultValue from "./unknown";
import * as namespace from "./store";
import { ref as makeRef } from "vue";
import VueDefault from "vue";
import Component from "./Child.vue";
import { default as ComponentAlias } from "./Child.vue";
import { "default" as StringComponentAlias } from "./Child.vue";
import { namedComponent } from "./Child.vue";
import type TypeOnly from "./types";
import type { OtherType } from "./types";
import { type MixedType, runtimeValue } from "./mixed";
"#;
        let bindings = ordinary(source, Some("ts"));
        for name in [
            "label",
            "handler",
            "defaultValue",
            "namedComponent",
            "runtimeValue",
        ] {
            assert_eq!(
                bindings.bindings.get(name),
                Some(&BindingType::SetupMaybeRef),
                "{name}"
            );
        }
        for name in [
            "namespace",
            "makeRef",
            "VueDefault",
            "Component",
            "ComponentAlias",
            "StringComponentAlias",
        ] {
            assert_eq!(
                bindings.bindings.get(name),
                Some(&BindingType::SetupConst),
                "{name}"
            );
        }
        for name in ["TypeOnly", "OtherType", "MixedType"] {
            assert!(!bindings.bindings.contains_key(name), "{name}");
        }
        assert!(!bindings.is_script_setup);

        let mut target = BindingMetadata {
            is_script_setup: true,
            ..Default::default()
        };
        target
            .bindings
            .insert("label".into(), BindingType::SetupRef);
        merge_normal_script_bindings(&mut target, &bindings);
        assert!(target.is_script_setup);
        assert_eq!(target.bindings.get("label"), Some(&BindingType::SetupRef));
        assert_eq!(
            target.bindings.get("handler"),
            Some(&BindingType::SetupMaybeRef)
        );
    }
}
