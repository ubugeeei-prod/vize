//! Production interface export from Croquis to the six Davinci α facets.
//!
//! Contracts are compact JSON with fixed field order. Type spellings and
//! defaults are kept verbatim, including whitespace inside literal types.
//! Declaration offsets, private bindings and implementation bodies, and
//! template expression text never enter these pages. An authored public
//! default expression is interface metadata and is preserved exactly,
//! including a function expression used as that default.

#[cfg(test)]
mod emit_tests;
mod environment;
#[cfg(test)]
mod environment_tests;
#[cfg(test)]
mod environment_world_tests;
mod exports;
mod schema;
#[cfg(test)]
mod tests;
mod type_refs;
mod types;

pub use environment::{TypeDependency, TypeEnvironment};
pub use schema::{ALPHA_CONTRACT_SCHEMA, AlphaSchema};
pub use types::{
    ComponentContract, EmitContract, ExposeContract, PropContract, ReactivityContract,
    SignatureContract, SlotContract,
};

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use vize_carton::{CompactString, cstr};
use vize_l2::summary::{AlphaEntry, AlphaPages, Signature};

use super::{ComponentShape, Croquis};
use crate::BindingType;

impl Croquis {
    /// Version of the structured JSON contracts this producer exports.
    pub const ALPHA_CONTRACT_SCHEMA: u16 = ALPHA_CONTRACT_SCHEMA;

    /// Export this component's declared interface from authoritative facts.
    ///
    /// Unknown types stay null. Options API props known only through binding
    /// metadata have unknown required/default/type facts. Reactivity exports
    /// only explicitly exposed members and model bindings, so changing private
    /// setup state cannot invalidate a consuming component's interface.
    /// Authored annotations are not TypeScript inference results: unresolved
    /// types are explicit, and no private function body is copied to infer one.
    ///
    /// # Errors
    ///
    /// Returns the serializer error if a contract cannot be encoded.
    pub fn alpha_pages(
        &self,
        signature_name: &str,
        generic: Option<&str>,
    ) -> Result<AlphaPages, serde_json::Error> {
        let mut props = BTreeMap::new();
        for prop in self.macros.props() {
            props
                .entry(prop.name.clone())
                .or_insert_with(|| PropContract {
                    schema: AlphaSchema,
                    name: prop.name.clone(),
                    prop_type: prop.prop_type.clone(),
                    required: Some(prop.required),
                    default_value: prop.default_value.clone(),
                    model_modifiers: None,
                    type_dependencies: self
                        .prop_type_environment(prop.prop_type.as_deref(), generic),
                });
        }
        let mut payloads: BTreeMap<CompactString, Vec<Option<CompactString>>> = BTreeMap::new();
        for emit in self.macros.emits() {
            payloads
                .entry(emit.name.clone())
                .or_default()
                .push(emit.payload_type.clone());
        }
        let mut emits: BTreeMap<_, _> = payloads
            .into_iter()
            .map(|(name, overload_payloads)| {
                let type_dependencies = self.emit_type_environment(
                    &name,
                    overload_payloads.iter().map(|payload| payload.as_deref()),
                    generic,
                );
                let contract = EmitContract {
                    schema: AlphaSchema,
                    name: name.clone(),
                    payload: overload_payloads.first().cloned().flatten(),
                    unresolved_type_arguments: overload_payloads
                        .iter()
                        .any(Option::is_none)
                        .then(|| {
                            self.macros
                                .define_emits()
                                .and_then(|call| call.type_args.clone())
                        })
                        .flatten(),
                    validator_signatures: self.macros.emit_validator_signatures(&name).to_vec(),
                    validator_type_annotations: self
                        .macros
                        .emit_validator_type_annotations(&name)
                        .to_vec(),
                    overload_payloads,
                    type_dependencies,
                };
                (name, contract)
            })
            .collect();
        for model in self.macros.models() {
            props
                .entry(model.name.clone())
                .or_insert_with(|| PropContract {
                    schema: AlphaSchema,
                    name: model.name.clone(),
                    prop_type: model.model_type.clone(),
                    required: Some(model.required),
                    default_value: model.default_value.clone(),
                    model_modifiers: self
                        .macros
                        .model_modifier_type(&model.name)
                        .map(CompactString::new),
                    type_dependencies: self.model_type_environment(model, generic),
                });
            let name = cstr!("update:{}", model.name);
            emits.entry(name.clone()).or_insert_with(|| EmitContract {
                schema: AlphaSchema,
                name,
                payload: model.model_type.clone(),
                overload_payloads: vec![model.model_type.clone()],
                unresolved_type_arguments: None,
                validator_signatures: Vec::new(),
                validator_type_annotations: Vec::new(),
                type_dependencies: self.type_environment(model.model_type.as_deref(), generic),
            });
        }
        for (name, kind) in self.bindings.iter() {
            if kind == BindingType::Props {
                props
                    .entry(CompactString::new(name))
                    .or_insert_with(|| PropContract {
                        schema: AlphaSchema,
                        name: CompactString::new(name),
                        prop_type: None,
                        required: None,
                        default_value: None,
                        model_modifiers: None,
                        type_dependencies: self.type_environment(None, generic),
                    });
            }
        }
        let mut slots = BTreeMap::new();
        for slot in self.macros.slots() {
            slots
                .entry(slot.name.clone())
                .or_insert_with(|| SlotContract {
                    schema: AlphaSchema,
                    name: slot.name.clone(),
                    props: slot.props_type.clone(),
                    type_dependencies: self.type_environment(slot.props_type.as_deref(), generic),
                });
        }
        Ok(AlphaPages {
            signature: Signature {
                name: declaration_key(signature_name),
                params: serialize(&self.signature_contract(signature_name, generic))?,
            },
            props: entries(props)?,
            emits: entries(emits)?,
            slots: entries(slots)?,
            reactivity: self.export_reactivity(generic)?,
            components: self.export_components()?,
        })
    }

    fn signature_contract(&self, name: &str, generic: Option<&str>) -> SignatureContract {
        let exposes: BTreeMap<_, _> = self
            .macros
            .exposes()
            .iter()
            .map(|expose| {
                (
                    expose.name.clone(),
                    ExposeContract {
                        name: expose.name.clone(),
                        expose_type: expose.expose_type.clone(),
                    },
                )
            })
            .collect();
        let fallback: Vec<_> = self
            .bindings
            .iter()
            .filter(|(_, kind)| *kind == BindingType::Props)
            .map(|(name, _)| CompactString::new(name))
            .collect();
        let prop_order = ordered_names(
            self.macros
                .props()
                .iter()
                .map(|prop| prop.name.clone())
                .chain(self.macros.models().iter().map(|model| model.name.clone()))
                .chain(fallback),
        );
        let slot_order = ordered_names(self.macros.slots().iter().map(|slot| slot.name.clone()));
        SignatureContract {
            schema: AlphaSchema,
            name: CompactString::new(name),
            declared_name: self.macros.define_options_name().map(CompactString::new),
            generic: generic.map(CompactString::new),
            component_shape: CompactString::new(match self.component_shape {
                ComponentShape::Unspecified => "unspecified",
                ComponentShape::ClassApi => "class-api",
            }),
            script_setup: self.bindings.is_script_setup,
            prop_order,
            props_complete: self.types.resolved_props_complete().unwrap_or(false),
            slot_order,
            prop_type_arguments: self
                .macros
                .define_props()
                .and_then(|call| call.type_args.clone()),
            with_defaults: self
                .macros
                .with_defaults_expression()
                .map(CompactString::new),
            emit_type_arguments: self
                .macros
                .define_emits()
                .and_then(|call| call.type_args.clone()),
            slot_type_arguments: self
                .macros
                .define_slots()
                .and_then(|call| call.type_args.clone()),
            exposes_complete: self.macros.expose_is_complete(),
            exposes: exposes.into_values().collect(),
            type_dependencies: self.signature_type_environment(generic),
        }
    }
}

/// An injective folio-safe key; contracts retain the original name.
///
/// Ordinary names are unchanged. Empty names, whitespace, `=`, and the
/// reserved `%` prefix are encoded as `%` followed by hexadecimal UTF-8.
#[must_use]
pub fn declaration_key(name: &str) -> CompactString {
    if !name.is_empty()
        && !name.starts_with('%')
        && !name.chars().any(|ch| ch.is_whitespace() || ch == '=')
    {
        return CompactString::new(name);
    }
    let mut encoded = CompactString::new("%");
    for byte in name.bytes() {
        for nibble in [byte >> 4, byte & 15] {
            encoded.push(char::from(if nibble < 10 {
                b'0' + nibble
            } else {
                b'A' + nibble - 10
            }));
        }
    }
    encoded
}

fn serialize(value: &impl Serialize) -> Result<CompactString, serde_json::Error> {
    serde_json::to_string(value).map(CompactString::from)
}

fn entries<T: Serialize>(
    contracts: BTreeMap<CompactString, T>,
) -> Result<Vec<AlphaEntry>, serde_json::Error> {
    contracts
        .into_iter()
        .map(|(name, contract)| {
            Ok(AlphaEntry {
                name: declaration_key(&name),
                contract: serialize(&contract)?,
            })
        })
        .collect()
}

fn ordered_names(names: impl Iterator<Item = CompactString>) -> Vec<CompactString> {
    let mut seen = BTreeSet::new();
    names.filter(|name| seen.insert(name.clone())).collect()
}
