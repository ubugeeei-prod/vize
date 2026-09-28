//! The actual imported-component metadata consumer as a backdated query.

use vize_davinci::summary::{AlphaEntry, Facet, SfcSummary};

use crate::{ResidentDocuments, SharedDescriptor, SummaryInput, sfc_summary};

/// Prop, event, and slot contracts used by imported-component IDE metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSurface {
    /// Producer metadata for authored prop and slot presentation order.
    pub signature: Option<AlphaEntry>,
    /// Current props, in canonical declaration order.
    pub props: Vec<AlphaEntry>,
    /// Current declared component events.
    pub emits: Vec<AlphaEntry>,
    /// Current slots, in canonical declaration order.
    pub slots: Vec<AlphaEntry>,
}

impl ComponentSurface {
    /// The clean projection used by TS-42 and metadata parity comparisons.
    #[must_use]
    pub fn from_summary(summary: &SfcSummary) -> Self {
        let project = |facet| {
            summary
                .iter()
                .filter(|(kind, _, _)| *kind == facet)
                .map(|(_, name, contract)| AlphaEntry {
                    name: name.into(),
                    contract: contract.into(),
                })
                .collect()
        };
        Self {
            signature: summary
                .iter()
                .find(|(facet, _, _)| *facet == Facet::Signature)
                .map(|(_, name, contract)| AlphaEntry {
                    name: name.into(),
                    contract: contract.into(),
                }),
            props: project(Facet::Prop),
            emits: project(Facet::Emit),
            slots: project(Facet::Slot),
        }
    }

    /// Exactly the declarations the IDE consumer decodes.
    pub fn iter(&self) -> impl Iterator<Item = (Facet, &str, &str)> {
        self.signature
            .iter()
            .map(|entry| {
                (
                    Facet::Signature,
                    entry.name.as_str(),
                    entry.contract.as_str(),
                )
            })
            .chain(
                self.props
                    .iter()
                    .map(|entry| (Facet::Prop, entry.name.as_str(), entry.contract.as_str()))
                    .chain(
                        self.emits
                            .iter()
                            .map(|entry| {
                                (Facet::Emit, entry.name.as_str(), entry.contract.as_str())
                            })
                            .chain(self.slots.iter().map(|entry| {
                                (Facet::Slot, entry.name.as_str(), entry.contract.as_str())
                            })),
                    ),
            )
    }
}

#[salsa::tracked(returns(ref), lru = 128)]
pub fn component_surface(
    db: &dyn salsa::Database,
    input: SummaryInput,
) -> Option<ComponentSurface> {
    sfc_summary(db, input)
        .as_ref()
        .ok()
        .map(ComponentSurface::from_summary)
}

impl ResidentDocuments {
    /// Publish the provider, then read the memoized production metadata
    /// consumer. Equal summaries backdate before this query executes.
    pub fn component_surface(
        &mut self,
        key: &str,
        filename: &str,
        text: &str,
        configuration: &str,
        export: impl FnOnce(&SharedDescriptor) -> Option<vize_davinci::summary::AlphaPages>,
    ) -> Option<ComponentSurface> {
        let _current = self.interface(key, filename, text, configuration, export)?;
        let input = self.interfaces.entries.get(key)?.input;
        let result = component_surface(&self.db, input).clone();
        if let Some(counts) = self.db.take_accounting().get("component_surface") {
            self.interfaces.stats.consumers = self
                .interfaces
                .stats
                .consumers
                .saturating_add(counts.executed);
            self.interfaces.stats.consumer_reuses = self
                .interfaces
                .stats
                .consumer_reuses
                .saturating_add(counts.reused);
        }
        result
    }
}
