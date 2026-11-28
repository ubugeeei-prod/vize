//! Where each `useTemplateRef` call runs, from the owners the walk recorded.

use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashSet;

use super::owners::{DeferredApi, Owner, Walk};
use super::template_uses::template_uses;
use crate::rules::script::SfcScriptContext;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Placement {
    /// A component instance runs the call, or its owner is unknown.
    Instance,
    /// Module evaluation.
    Module,
    /// A callback of this API.
    Deferred(DeferredApi),
    /// A local function that only runs after setup.
    Local(SymbolId),
}

/// The placement of every site of `walk`, in order.
pub(super) fn place(walk: &Walk<'_>, sfc: &SfcScriptContext<'_>) -> Vec<Placement> {
    let needs_locals = walk
        .sites
        .iter()
        .any(|site| matches!(site.owner, Owner::Local(_)));
    let deferred = if needs_locals {
        deferred_locals(walk, sfc)
    } else {
        FxHashSet::default()
    };
    walk.sites
        .iter()
        .map(|site| match site.owner {
            Owner::Module => Placement::Module,
            Owner::Deferred(api) => Placement::Deferred(api),
            Owner::Local(symbol) if deferred.contains(&symbol) => Placement::Local(symbol),
            Owner::Local(_) | Owner::Setup | Owner::Unknown => Placement::Instance,
        })
        .collect()
}

/// The local functions setup reaches by calling them, directly or through
/// other local functions.
fn reached_from_setup(walk: &Walk<'_>) -> FxHashSet<SymbolId> {
    let mut reached = FxHashSet::default();
    loop {
        let before = reached.len();
        for (owner, callee) in &walk.edges {
            let runs_in_instance = match owner {
                Owner::Setup => true,
                Owner::Local(function) => reached.contains(function),
                Owner::Module | Owner::Deferred(_) | Owner::Unknown => false,
            };
            if runs_in_instance {
                reached.insert(*callee);
            }
        }
        if reached.len() == before {
            return reached;
        }
    }
}

/// The local functions that provably run only after setup.
///
/// A candidate is a local function setup never calls, that is never read
/// except by a call or as a deferred callback, and that the template reads at
/// most as a `v-on` handler. A candidate called from code that is not itself a
/// candidate (or a deferred callback) is dropped, and the rest must be reached
/// from evidence: a deferred callback, a template handler, or a candidate.
fn deferred_locals(walk: &Walk<'_>, sfc: &SfcScriptContext<'_>) -> FxHashSet<SymbolId> {
    let reached = reached_from_setup(walk);
    let mut candidates: Vec<SymbolId> = walk
        .locals
        .iter()
        .copied()
        .filter(|local| !reached.contains(local) && !walk.escapes.contains(local))
        .collect();
    if candidates.is_empty() {
        return FxHashSet::default();
    }
    let mut evidence = FxHashSet::default();
    if sfc.is_script_setup {
        let names: Vec<&str> = candidates
            .iter()
            .map(|symbol| walk.symbol_name(*symbol))
            .collect();
        let uses = template_uses(sfc, &names);
        let mut kept = Vec::with_capacity(candidates.len());
        for (symbol, found) in candidates.iter().zip(uses) {
            if found.other {
                continue;
            }
            if found.handler {
                evidence.insert(*symbol);
            }
            kept.push(*symbol);
        }
        candidates = kept;
    }
    let mut candidates: FxHashSet<SymbolId> = candidates.into_iter().collect();
    loop {
        let before = candidates.len();
        for (owner, callee) in &walk.edges {
            let caller_runs_later = match owner {
                Owner::Deferred(_) => true,
                Owner::Local(function) => candidates.contains(function),
                Owner::Module | Owner::Setup | Owner::Unknown => false,
            };
            if !caller_runs_later {
                candidates.remove(callee);
            }
        }
        if candidates.len() == before {
            break;
        }
    }
    evidence.retain(|symbol| candidates.contains(symbol));
    for symbol in &candidates {
        if walk.deferred_passes.contains(symbol) {
            evidence.insert(*symbol);
        }
    }
    for (owner, callee) in &walk.edges {
        if matches!(owner, Owner::Deferred(_)) && candidates.contains(callee) {
            evidence.insert(*callee);
        }
    }
    loop {
        let before = evidence.len();
        for (owner, callee) in &walk.edges {
            if let Owner::Local(function) = owner
                && evidence.contains(function)
                && candidates.contains(callee)
            {
                evidence.insert(*callee);
            }
        }
        if evidence.len() == before {
            return evidence;
        }
    }
}
