//! Authoritative reactivity and resolved component identity projections.

use std::collections::{BTreeMap, BTreeSet};

use vize_carton::CompactString;
use vize_davinci::summary::AlphaEntry;
use vize_l3::lattice::EffectKind;

use super::{AlphaSchema, ComponentContract, ReactivityContract, entries, serialize};
use crate::Croquis;
use crate::facts::{SourceFact, component_identity, reactivity_sources};
use crate::reactivity::ReactiveKind;

impl Croquis {
    pub(super) fn export_reactivity(
        &self,
        generic: Option<&str>,
    ) -> Result<Vec<AlphaEntry>, serde_json::Error> {
        let sources = reactivity_sources(self);
        let mut contracts = BTreeMap::new();
        for expose in self.macros.exposes() {
            let binding = self
                .macros
                .expose_bindings()
                .iter()
                .find(|binding| binding.name == expose.name);
            let source = binding.and_then(|binding| {
                let local = binding.local_name.as_deref()?;
                sources.iter().rev().find(|source| {
                    source.name == local
                        && binding.declaration_span.is_some_and(|(start, end)| {
                            start <= source.declaration_offset && source.declaration_offset < end
                        })
                })
            });
            contracts.insert(
                expose.name.clone(),
                self.reactivity_contract(
                    &expose.name,
                    expose.expose_type.as_deref(),
                    source,
                    generic,
                ),
            );
        }
        for model in self.macros.models() {
            let source = sources
                .iter()
                .rev()
                .find(|source| source.name == model.local_name);
            contracts.entry(model.name.clone()).or_insert_with(|| {
                self.reactivity_contract(&model.name, model.model_type.as_deref(), source, generic)
            });
        }
        entries(contracts)
    }

    pub(super) fn export_components(&self) -> Result<Vec<AlphaEntry>, serde_json::Error> {
        let identities: BTreeSet<_> = self
            .used_components
            .iter()
            .map(|name| component_identity(self, name))
            .chain(
                self.component_usages
                    .iter()
                    .map(|usage| component_identity(self, &usage.name)),
            )
            .collect();
        let mut contracts = BTreeMap::new();
        for identity in identities {
            let contract = ComponentContract {
                schema: AlphaSchema,
                module: identity.module,
                export: identity.export_name,
            };
            // A JSON tuple is injective even when names contain ':' or '/'.
            let name = serialize(&(&contract.module, &contract.export))?;
            contracts.insert(name, contract);
        }
        entries(contracts)
    }
}

impl Croquis {
    fn reactivity_contract(
        &self,
        name: &str,
        binding_type: Option<&str>,
        source: Option<&SourceFact>,
        generic: Option<&str>,
    ) -> ReactivityContract {
        ReactivityContract {
            schema: AlphaSchema,
            name: CompactString::new(name),
            binding_type: binding_type.map(CompactString::new),
            kind: source.map(|source| CompactString::new(kind_name(source.kind))),
            class: source.map(|source| CompactString::new(source.class.as_str())),
            verdict: CompactString::new(source.map_or("unknown", |source| source.verdict.as_str())),
            effects: source
                .map(|source| {
                    [
                        EffectKind::Freeze,
                        EffectKind::Capture,
                        EffectKind::ReadProp,
                        EffectKind::ReadReactive,
                        EffectKind::MutateLocal,
                        EffectKind::MutateGlobal,
                        EffectKind::CallUnknown,
                        EffectKind::Allocate,
                    ]
                    .into_iter()
                    .filter(|effect| source.effects.contains(*effect))
                    .map(|effect| CompactString::new(effect.as_str()))
                    .collect()
                })
                .unwrap_or_default(),
            type_dependencies: self.type_environment(binding_type, generic),
        }
    }
}

fn kind_name(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref => "ref",
        ReactiveKind::ShallowRef => "shallowRef",
        ReactiveKind::Reactive => "reactive",
        ReactiveKind::ShallowReactive => "shallowReactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly => "readonly",
        ReactiveKind::ShallowReadonly => "shallowReadonly",
        ReactiveKind::ToRef => "toRef",
        ReactiveKind::ToRefs => "toRefs",
    }
}
