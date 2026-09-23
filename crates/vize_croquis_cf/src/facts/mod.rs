//! The cross-file rules' declared fact demands (Davinci P4-3).
//!
//! Every cross-file rule that reads a Croquis fact group names its demand
//! here as const data, so the debug detector (TS-35) refuses any read
//! outside it and the demand set of the whole lane reads in one place.

mod render_tree;
#[cfg(test)]
mod render_tree_tests;

use vize_croquis::facts::{Bindings, Demand, FactConsumer, FactGroup};
use vize_davinci::fact::{FactManager, FactRegistry};

pub(crate) use render_tree::resolve_module;
pub use render_tree::{RenderEdgeKey, RenderSite, RenderTree, component_usage_targets};

use crate::registry::ModuleRegistry;

/// `cross-file/error-boundary`: script bindings (`onErrorCaptured`).
pub struct ErrorBoundaryRule;

impl FactConsumer for ErrorBoundaryRule {
    const NAME: &'static str = "cross-file/error-boundary";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// `cross-file/component-resolution`: script bindings that register a
/// template component locally.
pub struct ComponentResolutionRule;

impl FactConsumer for ComponentResolutionRule {
    const NAME: &'static str = "cross-file/component-resolution";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// The edge builder reads the project render tree.
pub struct RenderTreeReader;

impl FactConsumer for RenderTreeReader {
    const NAME: &'static str = "cross-file/render-tree";
    const DEMAND: Demand = Demand::NONE.with(RenderTree::ID);
}

/// Every project-level fact group.
pub const PROJECT_FACTS: FactRegistry<ModuleRegistry> =
    FactRegistry::new(&[vize_davinci::fact::ProducerEntry::of::<RenderTree>()]);

/// Facts computed over one module registry.
pub struct ProjectFacts<'a> {
    registry: &'a ModuleRegistry,
    manager: FactManager<'static, ModuleRegistry>,
}

impl<'a> ProjectFacts<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        Self {
            registry,
            manager: FactManager::new(&PROJECT_FACTS),
        }
    }

    pub fn prepare<C: FactConsumer>(&mut self) -> vize_davinci::fact::FactView<'_> {
        match self.manager.prepare::<C>(self.registry) {
            Ok(view) => view,
            Err(error) => panic!(
                "{} demands an unregistered project fact group: {error:?}",
                C::NAME
            ),
        }
    }
}
