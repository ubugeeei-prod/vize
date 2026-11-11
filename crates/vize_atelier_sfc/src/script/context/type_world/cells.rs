//! Per-snapshot dependency cells that concurrent world walks share.
//!
//! Walks of one snapshot reach the same dependencies in the same order: every
//! component of a library imports the same barrels, and each walk is a
//! depth-first stack. Waiting on whichever walk resolves a module therefore
//! queued every other walk behind one read, parse and target resolution at a
//! time, so a host burst resolved its shared dependency graph serially. A walk
//! that finds its next module in progress resolves a module some walk has
//! already queued and nobody has started, and waits only when none is left.
//! Each module is still resolved once per snapshot, by whichever walk claims it,
//! and every walk visits modules in its own order, so worlds are unchanged.

#![expect(
    clippy::disallowed_types,
    reason = "cells share resolved dependency facts across the walks of a snapshot"
)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use super::{ResolvedDependency, TypeSourceSnapshot};

/// One dependency's resolution in a snapshot.
#[derive(Debug, Default)]
pub(in crate::script::context) struct ResolvedSlot {
    /// Taken by the walk that resolves the module, so a waiting walk never
    /// starts a module another one is already on.
    claimed: AtomicBool,
    value: OnceLock<Option<Arc<ResolvedDependency>>>,
}

impl ResolvedSlot {
    /// The module's resolution, once some walk has finished it.
    pub(in crate::script::context) fn get(&self) -> Option<&Option<Arc<ResolvedDependency>>> {
        self.value.get()
    }

    /// Whether this walk is the first to take the module on.
    fn claim(&self) -> bool {
        !self.claimed.swap(true, Ordering::AcqRel)
    }

    fn resolve(
        &self,
        path: &Path,
        sources: &TypeSourceSnapshot,
        resolve: impl FnOnce(&Path, &TypeSourceSnapshot) -> Option<Arc<ResolvedDependency>>,
    ) -> Option<Arc<ResolvedDependency>> {
        self.value.get_or_init(|| resolve(path, sources)).clone()
    }
}

type Slot = Arc<ResolvedSlot>;

/// Modules some walk has queued and will visit, nearest last.
pub(in crate::script::context) type Wanted = Vec<(PathBuf, Slot)>;

/// The snapshot's slot for `path`. The map lock covers only the lookup.
fn slot(path: &Path, sources: &TypeSourceSnapshot) -> Option<Slot> {
    let mut resolved = sources.resolved.lock().ok()?;
    Some(Arc::clone(match resolved.get(path) {
        Some(slot) => slot,
        None => resolved.entry(path.to_path_buf()).or_default(),
    }))
}

/// Resolve `path` once per snapshot; later walks reuse the `Arc`.
pub(super) fn read(
    path: &Path,
    sources: &TypeSourceSnapshot,
    resolve: fn(&Path, &TypeSourceSnapshot) -> Option<Arc<ResolvedDependency>>,
) -> Option<Arc<ResolvedDependency>> {
    let slot = slot(path, sources)?;
    if let Some(resolved) = slot.get() {
        return resolved.clone();
    }
    if !slot.claim() {
        while slot.get().is_none()
            && let Some((next, other)) = take_wanted(sources)
        {
            // A panic stays with the walks that visit the module, as when
            // they resolve it themselves: they find it unresolved and retry.
            let _ = catch_unwind(AssertUnwindSafe(|| other.resolve(&next, sources, resolve)));
        }
    }
    slot.resolve(path, sources, resolve)
}

/// Offer the modules a walk has just queued to walks that would otherwise wait.
pub(super) fn want(follow: &[PathBuf], sources: &TypeSourceSnapshot) {
    let mut queued = Vec::new();
    {
        let Ok(mut resolved) = sources.resolved.lock() else {
            return;
        };
        for path in follow {
            let slot = match resolved.get(path.as_path()) {
                Some(slot) => slot,
                None => resolved.entry(path.clone()).or_default(),
            };
            if slot.get().is_none() && !slot.claimed.load(Ordering::Acquire) {
                queued.push((path.clone(), Arc::clone(slot)));
            }
        }
    }
    if queued.is_empty() {
        return;
    }
    if let Ok(mut wanted) = sources.wanted.lock() {
        wanted.extend(queued);
    }
}

/// The latest queued module nobody has claimed, claimed for this walk.
fn take_wanted(sources: &TypeSourceSnapshot) -> Option<(PathBuf, Slot)> {
    let mut wanted = sources.wanted.lock().ok()?;
    while let Some((path, slot)) = wanted.pop() {
        if slot.get().is_none() && slot.claim() {
            return Some((path, slot));
        }
    }
    None
}
