//! Imported component surfaces consume production Croquis alpha contracts.

use vize_croquis::croquis::alpha::{
    AlphaSchema, PropContract, SignatureContract, SlotContract, TypeEnvironment, declaration_key,
};
use vize_davinci::summary::{AlphaEntry, AlphaPages, Facet};

use super::component_meta::{ComponentMetadata, ComponentProp, ComponentSlot};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) fn export_component_interface(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    filename: &str,
    options_api: bool,
    legacy_vue2: bool,
) -> Option<AlphaPages> {
    export_component_interface_with_sources(
        descriptor,
        filename,
        options_api,
        legacy_vue2,
        &vize_atelier_sfc::script::TypeSourceSnapshot::default(),
    )
}

pub(super) fn export_component_interface_with_sources(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    filename: &str,
    options_api: bool,
    legacy_vue2: bool,
    sources: &vize_atelier_sfc::script::TypeSourceSnapshot,
) -> Option<AlphaPages> {
    let allocator = vize_l0::Allocator::new();
    let root = descriptor
        .template
        .as_ref()
        .map(|template| vize_armature::parse(&allocator, template.content.as_ref()).0);
    let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor_resolved_with_sources(
        descriptor,
        root.as_ref(),
        vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
        options_api,
        legacy_vue2,
        filename,
        sources,
    );
    let generic = descriptor
        .script_setup
        .as_ref()
        .and_then(|script| script.attrs.get("generic"))
        .map(|value| value.as_ref());
    let mut pages = analysis.croquis.alpha_pages(filename, generic).ok()?;
    let mut signature: SignatureContract = serde_json::from_str(&pages.signature.params).ok()?;
    if let Some(root) = root.as_ref() {
        for name in super::slot_outlets::extract_slot_names_from_root(root) {
            let key = declaration_key(&name);
            if pages.slots.iter().any(|entry| entry.name == key) {
                continue;
            }
            pages.slots.push(AlphaEntry {
                name: key,
                contract: serde_json::to_string(&SlotContract {
                    schema: AlphaSchema,
                    name: name.clone().into(),
                    props: None,
                    type_dependencies: TypeEnvironment {
                        complete: false,
                        declarations: vec![],
                    },
                })
                .ok()?
                .into(),
            });
            signature.slot_order.push(name.into());
        }
    }
    pages.signature.params = serde_json::to_string(&signature).ok()?.into();
    Some(pages)
}

pub(super) fn component_metadata_from_interface(
    summary: &vize_resident::ComponentSurface,
) -> Option<ComponentMetadata> {
    let signature: SignatureContract =
        serde_json::from_str(&summary.signature.as_ref()?.contract).ok()?;
    let mut props = Vec::new();
    let mut slots = Vec::new();
    for (facet, _, contract) in summary.iter() {
        match facet {
            Facet::Prop => {
                let prop: PropContract = serde_json::from_str(contract).ok()?;
                props.push(ComponentProp {
                    name: prop.name.into(),
                    type_detail: prop.prop_type.map(Into::into),
                    required: prop.required.unwrap_or(false),
                    default_value: prop.default_value.map(Into::into),
                });
            }
            Facet::Slot => {
                let slot: SlotContract = serde_json::from_str(contract).ok()?;
                slots.push(ComponentSlot {
                    name: slot.name.into(),
                    props_type: slot.props.map(Into::into),
                });
            }
            _ => {}
        }
    }
    let prop_order: vize_l0::FxHashMap<_, _> = signature
        .prop_order
        .iter()
        .enumerate()
        .map(|(ordinal, name)| (name.as_str(), ordinal))
        .collect();
    let slot_order: vize_l0::FxHashMap<_, _> = signature
        .slot_order
        .iter()
        .enumerate()
        .map(|(ordinal, name)| (name.as_str(), ordinal))
        .collect();
    props.sort_by_key(|prop| {
        prop_order
            .get(prop.name.as_str())
            .copied()
            .unwrap_or(usize::MAX)
    });
    slots.sort_by_key(|slot| {
        slot_order
            .get(slot.name.as_str())
            .copied()
            .unwrap_or(usize::MAX)
    });
    Some(ComponentMetadata { props, slots })
}
