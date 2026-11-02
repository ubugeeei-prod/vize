//! Dotted spelling alone never establishes a value owner or a property type.
use super::{component_binding_reference_with_provider, dotted_component_root};
use crate::virtual_ts::types::VirtualTsOptions;
use vize_carton::{CompactString, FxHashSet};
use vize_croquis::{BindingType, Croquis};

#[test]
fn dotted_component_uses_exact_root_value_without_case_or_property_enumeration() {
    let mut summary = Croquis::new();
    summary.bindings.is_script_setup = true;
    summary.bindings.add("Parts", BindingType::SetupConst);
    let options = VirtualTsOptions::default();
    let type_only = FxHashSet::default();
    for name in ["Parts.Foo", "Parts.Nested.Foo", "Parts.Missing"] {
        let (reference, actual_value) =
            component_binding_reference_with_provider(&summary, &options, &type_only, name);
        assert_eq!(reference, name);
        assert!(actual_value);
    }
    let (_, actual_value) =
        component_binding_reference_with_provider(&summary, &options, &type_only, "parts.Foo");
    assert!(!actual_value, "namespace root identity is case sensitive");
}

#[test]
fn type_only_and_unknown_roots_never_become_component_value_bindings() {
    let mut summary = Croquis::new();
    summary.bindings.is_script_setup = true;
    summary.bindings.add("Parts", BindingType::SetupConst);
    let options = VirtualTsOptions::default();
    let type_only = FxHashSet::from_iter([CompactString::new("Parts")]);
    for name in ["Parts.Foo", "Missing.Foo", "Parts..Foo", "Parts.Foo-Bar"] {
        let (_, actual_value) =
            component_binding_reference_with_provider(&summary, &options, &type_only, name);
        assert!(!actual_value, "{name}");
    }
    for name in ["Parts", ".Foo", "Parts.", "Parts..Foo", "Parts.Foo-Bar"] {
        assert!(dotted_component_root(name).is_none(), "{name}");
    }
    assert_eq!(dotted_component_root("部品.Foo"), Some("部品"));
}
