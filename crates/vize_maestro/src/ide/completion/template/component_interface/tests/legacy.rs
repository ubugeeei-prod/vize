//! Pre-migration metadata projection retained only as the parity oracle.
use super::super::super::component_meta::{ComponentMetadata, ComponentProp, ComponentSlot};
use std::collections::BTreeSet;
use vize_relief::BindingType;
pub(super) fn legacy_metadata(
    descriptor: Option<&vize_atelier_sfc::SfcDescriptor<'_>>,
    filename: &str,
    options_api: bool,
    legacy_vue2: bool,
) -> ComponentMetadata {
    let Some(descriptor) = descriptor else {
        return ComponentMetadata {
            props: Vec::new(),
            slots: Vec::new(),
            emits: Vec::new(),
        };
    };

    let mut props = Vec::new();
    let mut slots = Vec::new();
    let mut seen_props = BTreeSet::new();
    let mut seen_slots = BTreeSet::new();

    if descriptor.script_setup.is_some() || descriptor.script.is_some() {
        let summary = vize_atelier_sfc::croquis::analyze_sfc_descriptor_resolved(
            descriptor,
            None,
            vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
            options_api,
            legacy_vue2,
            filename,
        )
        .croquis;

        for prop in summary.macros.props() {
            if seen_props.insert(prop.name.to_string()) {
                props.push(ComponentProp {
                    name: prop.name.to_string(),
                    type_detail: prop.prop_type.as_ref().map(|ty| ty.to_string()),
                    required: prop.required,
                    default_value: prop.default_value.as_ref().map(|d| d.to_string()),
                });
            }
        }

        // defineModel<T>() introduces a prop alongside an `update:NAME`
        // event. Prop completion only knew about defineProps before, so
        // child components using defineModel showed no prop suggestions.
        // See #686.
        for model in summary.macros.models() {
            if seen_props.insert(model.name.to_string()) {
                props.push(ComponentProp {
                    name: model.name.to_string(),
                    type_detail: model.model_type.as_ref().map(|ty| ty.to_string()),
                    required: model.required,
                    default_value: model.default_value.as_ref().map(|d| d.to_string()),
                });
            }
        }

        if options_api || legacy_vue2 {
            for (name, binding_type) in summary.bindings.iter() {
                if binding_type == BindingType::Props && seen_props.insert(name.to_string()) {
                    props.push(ComponentProp {
                        name: name.to_string(),
                        type_detail: None,
                        required: false,
                        default_value: None,
                    });
                }
            }
        }

        for slot in summary.macros.slots() {
            let name = slot.name.to_string();
            if seen_slots.insert(name.clone()) {
                slots.push(ComponentSlot {
                    name,
                    props_type: slot.props_type.as_ref().map(|props| props.to_string()),
                });
            }
        }
    }

    if let Some(template) = descriptor.template.as_ref() {
        for name in super::super::super::slot_outlets::extract_template_slot_names(
            template.content.as_ref(),
        ) {
            if seen_slots.insert(name.clone()) {
                slots.push(ComponentSlot {
                    name,
                    props_type: None,
                });
            }
        }
    }

    ComponentMetadata {
        props,
        slots,
        emits: Vec::new(),
    }
}
