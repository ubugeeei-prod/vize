//! The facts a JS plugin can demand, served through the P4-1 fact API.
//!
//! A JS manifest declares its demands as a static list of group **names**
//! (`demands: ["templateScopes"]`). The host resolves each name against this
//! registry's [`GroupDesc`](vize_davinci::fact::GroupDesc) names into one
//! [`Demand`], computes exactly its closure with a [`FactManager`] (each
//! group at most once per document, however many plugins demand it), and
//! serializes only the declared groups into the plugin's batch. The host is
//! the Rust-side [`FactConsumer`]; the per-plugin declared set is enforced
//! twice — the batch never carries an undeclared group, and the SDK's
//! `ctx.facts(name)` throws on one (the TS-35 rule, on the JS side).
//!
//! Until the P4-3 waves register production producers there is one
//! JS-visible group, `templateScopes`, derived from the S2 page. It takes
//! the first fixture-range id: this registry never shares a manager with
//! production groups (the `fact::ids` rule), and P6-7 replaces it with the
//! production groups' α pages (P4-2).

#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use serde::Serialize;
use vize_davinci::fact::ids::FIXTURE_BASE;
use vize_davinci::fact::{
    Demand, FactConsumer, FactGroup, FactManager, FactProducer, FactRegistry, FactTable, FactView,
    ProducerEntry,
};
use vize_davinci::pass::AnalysisId;

use super::document::{PluginDocument, PluginNode};
use super::error::HostError;

/// `templateScopes` — for every binding-introducing op, the names it binds
/// and at which position (`value` / `key` / `index` for `ui.for`, `slot`
/// for scoped-slot parameters). Destructuring positions bind no entry.
pub struct TemplateScopes;

impl FactGroup for TemplateScopes {
    const ID: AnalysisId = AnalysisId::new(FIXTURE_BASE);
    const NAME: &'static str = "templateScopes";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = Vec<ScopeEntry>;
}

/// One name a scope binds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeEntry {
    pub name: String,
    pub position: &'static str,
}

impl FactProducer<PluginDocument> for TemplateScopes {
    fn produce(document: &PluginDocument, _: &FactView<'_>) -> FactTable<Self> {
        let entries = |node: &PluginNode| scope_entries(document, node).map(|e| (node.id, e));
        document.nodes.iter().filter_map(entries).collect()
    }
}

fn scope_entries(document: &PluginDocument, node: &PluginNode) -> Option<Vec<ScopeEntry>> {
    if let Some(alias) = &node.alias {
        let positions = [
            ("value", Some(&alias.value)),
            ("key", alias.key.as_ref()),
            ("index", alias.index.as_ref()),
        ];
        let bound = positions.into_iter().filter_map(|(position, text)| {
            let name = text?.trim();
            is_identifier(name).then(|| ScopeEntry {
                name: name.to_owned(),
                position,
            })
        });
        return Some(bound.collect());
    }
    let (_, names) = document.scopes.iter().find(|(id, _)| *id == node.id)?;
    let slot = |name: &String| ScopeEntry {
        name: name.clone(),
        position: "slot",
    };
    Some(names.iter().map(slot).collect())
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let head = |c: char| c.is_alphabetic() || c == '_' || c == '$';
    chars.next().is_some_and(head) && chars.all(|c| head(c) || c.is_ascii_digit())
}

/// Every group a JS plugin may demand.
pub const REGISTRY: FactRegistry<PluginDocument> =
    FactRegistry::new(&[ProducerEntry::of::<TemplateScopes>()]);

/// The names of [`REGISTRY`]'s groups, as the unknown-demand error lists them.
pub const JS_VISIBLE: &[&str] = &[TemplateScopes::NAME];

/// The host reads facts on the plugins' behalf; its demand is every
/// JS-visible group.
pub struct JsPluginHost;

impl FactConsumer for JsPluginHost {
    const NAME: &'static str = "js-plugin-host";
    const DEMAND: Demand = Demand::NONE.with(TemplateScopes::ID);
}

/// Resolve a manifest's demand names.
///
/// # Errors
///
/// [`HostError::UnknownFact`] for the first name no group carries.
pub fn resolve_demands(plugin: &str, names: &[String]) -> Result<Demand, HostError> {
    names.iter().try_fold(Demand::NONE, |demand, name| {
        let entry = REGISTRY.producers().iter().find(|e| e.desc.name == name);
        match entry {
            Some(entry) => Ok(demand.with(entry.desc.id)),
            None => Err(HostError::UnknownFact {
                plugin: plugin.to_owned(),
                name: name.clone(),
                known: JS_VISIBLE,
            }),
        }
    })
}

/// Compute `demand` and serialize exactly its groups, keyed by name.
pub fn demanded_facts(
    manager: &mut FactManager<'_, PluginDocument>,
    document: &PluginDocument,
    demand: Demand,
) -> serde_json::Map<String, serde_json::Value> {
    let mut facts = serde_json::Map::new();
    if demand.is_empty() {
        return facts;
    }
    // Every name was resolved against REGISTRY, so nothing is unregistered.
    let _ = manager.compute(document, demand);
    let view = manager.view::<JsPluginHost>();
    if demand.contains(TemplateScopes::ID)
        && let Ok(table) = view.get::<TemplateScopes>()
    {
        let rows: Vec<_> = table.iter().collect();
        facts.insert(TemplateScopes::NAME.to_owned(), serde_json::json!(rows));
    }
    facts
}
