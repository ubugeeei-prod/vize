//! Exact work/closure controls; no native checker or wall-time oracle.

use std::cell::Cell;
use std::path::{Path, PathBuf};

use crate::PackageResolutionMode;
use crate::package_route::PackageRouteKey;
use vize_carton::{FxHashMap, FxHashSet};

use super::super::VirtualProject;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Work {
    calls: usize,
    cold_calls: usize,
    pushed: usize,
    popped: usize,
    max_pending: usize,
}

thread_local! {
    static WORK: Cell<Work> = const { Cell::new(Work {
        calls: 0, cold_calls: 0, pushed: 0, popped: 0, max_pending: 0,
    }) };
}

fn update(change: impl FnOnce(&mut Work)) {
    WORK.with(|cell| {
        let mut work = cell.get();
        change(&mut work);
        cell.set(work);
    });
}

pub(super) fn record_call(cold: bool) {
    update(|work| {
        work.calls += 1;
        work.cold_calls += usize::from(cold);
    });
}

pub(super) fn record_push(pending: usize) {
    update(|work| {
        work.pushed += 1;
        work.max_pending = work.max_pending.max(pending);
    });
}

pub(super) fn record_pop() {
    update(|work| work.popped += 1);
}

fn reset() {
    WORK.with(|cell| cell.set(Work::default()));
}

fn work() -> Work {
    WORK.with(Cell::get)
}

fn key(package: &Path, index: usize) -> PackageRouteKey {
    PackageRouteKey {
        importer_path: package.join(vize_carton::cstr!("src/Node{index}.ts").as_str()),
        specifier: "@fixture/kit".into(),
        occurrence_mode: PackageResolutionMode::Contextual,
    }
}

/// The previous pop-then-deduplicate algorithm, as an independent closure model.
/// Its operation count is not an observed baseline binary timing.
fn reference(
    roots: &FxHashMap<PathBuf, FxHashSet<PackageRouteKey>>,
    seeds: Vec<PackageRouteKey>,
) -> (FxHashSet<PackageRouteKey>, usize) {
    let mut pending = seeds;
    let mut visited = FxHashSet::default();
    let mut popped = 0;
    while let Some(key) = pending.pop() {
        popped += 1;
        if !visited.insert(key.clone()) {
            continue;
        }
        for ancestor in key.importer_path.ancestors() {
            if let Some(owners) = roots.get(ancestor) {
                pending.extend(owners.iter().cloned());
            }
        }
    }
    (visited, popped)
}

#[test]
fn dense_warm_cycles_queue_each_key_once_and_keep_the_complete_closure() {
    const OWNERS: usize = 64;
    let directory = tempfile::tempdir().unwrap();
    let package = directory.path().join("packages/kit");
    let keys = (0..OWNERS).map(|i| key(&package, i)).collect::<Vec<_>>();
    let mut project = VirtualProject::new(directory.path()).unwrap();
    // Isolated worklist control; the separate fixture exercises real initialization.
    project.package_shadows_initialized = true;
    project
        .package_route_roots
        .insert(package, keys.iter().cloned().collect::<FxHashSet<_>>());
    let roots = project.package_route_roots.clone();
    let seeds = vec![keys[0].clone(), keys[0].clone()];
    let (expected, reference_pops) = reference(&roots, seeds.clone());
    assert_eq!(reference_pops, 2 + OWNERS * OWNERS);
    project.package_shadow_dirty_keys.insert(keys[0].clone());
    reset();
    project.mark_package_shadow_dependents(seeds);
    assert_eq!(project.package_shadow_dirty_keys, expected);
    assert_eq!(project.package_route_roots, roots);
    assert_eq!(work().calls, 1);
    assert_eq!(work().cold_calls, 0);
    assert_eq!(work().pushed, OWNERS);
    assert_eq!(work().popped, OWNERS);
    assert!(work().max_pending <= OWNERS);
}

#[test]
fn an_already_dirty_seed_reaches_new_owners_on_the_next_call() {
    let directory = tempfile::tempdir().unwrap();
    let p = directory.path().join("packages/p");
    let q = directory.path().join("packages/q");
    let a = key(&p, 0);
    let b = key(&q, 1);
    let c = key(&p, 2);
    let unrelated = key(&directory.path().join("outside"), 3);
    let mut project = VirtualProject::new(directory.path()).unwrap();
    project.package_shadows_initialized = true;
    project
        .package_route_roots
        .insert(p.clone(), FxHashSet::from_iter([a.clone()]));
    project
        .package_route_roots
        .insert(q, FxHashSet::from_iter([c.clone()]));
    project.package_shadow_dirty_keys.insert(unrelated.clone());
    project.mark_package_shadow_dependents([a.clone()]);
    assert_eq!(
        project.package_shadow_dirty_keys,
        FxHashSet::from_iter([a.clone(), unrelated.clone()])
    );

    // A new edge makes a cycle and must propagate through the already dirty a.
    project
        .package_route_roots
        .get_mut(&p)
        .unwrap()
        .insert(b.clone());
    let (mut expected, _) = reference(&project.package_route_roots, vec![a.clone()]);
    expected.insert(unrelated);
    reset();
    project.mark_package_shadow_dependents([a]);
    assert_eq!(project.package_shadow_dirty_keys, expected);
    assert_eq!(work().popped, 3);
    assert_eq!(work().pushed, 3);
}

#[test]
fn cold_propagation_retains_explicit_missing_and_removed_seed_identities() {
    let directory = tempfile::tempdir().unwrap();
    let p = directory.path().join("packages/p");
    let a = key(&p, 0);
    let owner = key(&p, 1);
    let missing = key(&directory.path().join("missing"), 2);
    let mut project = VirtualProject::new(directory.path()).unwrap();
    project
        .package_route_roots
        .insert(p, FxHashSet::from_iter([owner]));
    reset();
    project.mark_package_shadow_dependents([a.clone(), missing.clone(), a.clone()]);
    assert_eq!(
        project.package_shadow_dirty_keys,
        FxHashSet::from_iter([a, missing])
    );
    assert_eq!(
        work(),
        Work {
            calls: 1,
            cold_calls: 1,
            ..Work::default()
        }
    );
}

#[cfg(unix)]
mod fixture;
