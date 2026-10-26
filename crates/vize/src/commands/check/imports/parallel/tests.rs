use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use vize_canon::PackageRouteResolver;

use super::super::super::imports_aliases::PathAliasResolver;
use super::super::super::path_cache::CanonicalPathCache;
use super::super::{
    ImportFileOptions, LocalImportSession, collect_transitive_local_imports_with_session,
    registration,
};

static PACKAGE_IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static PACKAGE_MAX: AtomicUsize = AtomicUsize::new(0);
static PACKAGE_ENTERED: AtomicUsize = AtomicUsize::new(0);
static PATH_LOOKUPS: AtomicUsize = AtomicUsize::new(0);

fn write(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    path
}

fn note_package_lookup() {
    let now = PACKAGE_IN_FLIGHT.fetch_add(1, Ordering::SeqCst) + 1;
    PACKAGE_MAX.fetch_max(now, Ordering::SeqCst);
    PACKAGE_ENTERED.fetch_add(1, Ordering::SeqCst);
    let start = Instant::now();
    while PACKAGE_ENTERED.load(Ordering::SeqCst) < 2 && start.elapsed() < Duration::from_secs(2) {
        std::thread::yield_now();
    }
    std::thread::sleep(Duration::from_millis(40));
    PACKAGE_IN_FLIGHT.fetch_sub(1, Ordering::SeqCst);
}

fn note_path_lookup() {
    PATH_LOOKUPS.fetch_add(1, Ordering::SeqCst);
}

fn sorted(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort();
    paths
}

#[test]
fn frontier_collects_the_same_files_and_overlaps_independent_packages() {
    PACKAGE_IN_FLIGHT.store(0, Ordering::SeqCst);
    PACKAGE_MAX.store(0, Ordering::SeqCst);
    PACKAGE_ENTERED.store(0, Ordering::SeqCst);
    PATH_LOOKUPS.store(0, Ordering::SeqCst);

    let root = tempfile::tempdir().unwrap();
    let base = root.path();
    std::fs::write(
        base.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["src/*"]}}}"#,
    )
    .unwrap();
    let main = write(
        base,
        "src/main.ts",
        "import { leaf } from './leaf';\nimport { left } from 'left';\nimport { right } from 'right';\nimport { shared } from '@/shared';\nexport const value = leaf + left + right + shared;\n",
    );
    let other = write(
        base,
        "src/other.ts",
        "import { left } from 'left';\nimport { right } from 'right';\nimport { shared } from '@/shared';\nexport const value = left + right + shared;\n",
    );
    let leaf = write(base, "src/leaf.ts", "export const leaf = 1;\n");
    let shared_file = write(base, "src/shared.ts", "export const shared = 1;\n");
    write(
        base,
        "node_modules/left/package.json",
        r#"{"name":"left","types":"./index.d.ts"}"#,
    );
    write(
        base,
        "node_modules/left/index.d.ts",
        "export declare const left: number;\n",
    );
    write(
        base,
        "node_modules/right/package.json",
        r#"{"name":"right","types":"./index.d.ts"}"#,
    );
    write(
        base,
        "node_modules/right/index.d.ts",
        "export declare const right: number;\n",
    );

    let aliases = PathAliasResolver::from_tsconfig(Some(base.join("tsconfig.json").as_path()));
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::new(&mut packages);
    session.package_lookup_probe = Some(note_package_lookup);
    session.path_lookup_probe = Some(note_path_lookup);
    let discovered = collect_transitive_local_imports_with_session(
        &[main.clone(), other.clone()],
        base,
        &mut CanonicalPathCache::default(),
        false,
        Some(&aliases),
        &mut packages,
        &mut session,
    );

    let leaf = leaf.canonicalize().unwrap();
    let shared_file = shared_file.canonicalize().unwrap();
    let authored = sorted(discovered.authored);
    assert_eq!(authored, sorted(vec![leaf.clone(), shared_file]));
    assert_eq!(discovered.registrations, vec![leaf]);
    assert!(discovered.package_routes.is_empty());
    assert_eq!(
        PACKAGE_ENTERED.load(Ordering::SeqCst),
        2,
        "left and right are looked up once each"
    );
    assert!(
        PACKAGE_MAX.load(Ordering::SeqCst) >= 2,
        "independent package specifiers must overlap, not take one serial turn"
    );
    assert_eq!(
        PATH_LOOKUPS.load(Ordering::SeqCst),
        3,
        "left, right, and @/shared are resolved once across both importers"
    );
    let metrics = packages.metrics();
    assert_eq!(metrics.cache_misses, 2);
    assert_eq!(metrics.cache_hits, 0);

    let entered = PACKAGE_ENTERED.load(Ordering::SeqCst);
    let path_lookups = PATH_LOOKUPS.load(Ordering::SeqCst);
    let again = collect_transitive_local_imports_with_session(
        &[main, other],
        base,
        &mut CanonicalPathCache::default(),
        false,
        Some(&aliases),
        &mut packages,
        &mut session,
    );
    assert_eq!(sorted(again.authored), authored);
    assert_eq!(PACKAGE_ENTERED.load(Ordering::SeqCst), entered);
    assert_eq!(PATH_LOOKUPS.load(Ordering::SeqCst), path_lookups);
}

static CLOSURE_PACKAGE_LOOKUPS: AtomicUsize = AtomicUsize::new(0);

fn note_closure_package_lookup() {
    CLOSURE_PACKAGE_LOOKUPS.fetch_add(1, Ordering::SeqCst);
}

fn write_pkg(base: &Path) {
    write(
        base,
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        base,
        "node_modules/pkg/index.d.ts",
        "export interface Public { value: string }\n",
    );
}

#[test]
fn frontier_looks_ahead_through_relative_edges_before_the_walk_reaches_them() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path();
    let entry = write(base, "src/entry.ts", "import './a';\n");
    write(base, "src/a.ts", "import './nested/b';\n");
    write(
        base,
        "src/nested/b.ts",
        "import type { Public } from 'pkg';\nexport type B = Public;\n",
    );
    write_pkg(base);

    let mut packages = PackageRouteResolver::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let mut session = LocalImportSession::new(&mut packages);
    let entry = canonical_paths.canonicalize(&entry);
    super::prepare_frontier(
        &entry,
        &[],
        &mut session,
        &packages,
        None,
        ImportFileOptions::from(false),
        &mut canonical_paths,
    );

    assert_eq!(
        session.module_specifier_entries(),
        3,
        "one frontier reads entry, a and b"
    );
    assert_eq!(
        session.package_lookup_entries(),
        1,
        "b's package import resolves before the walk pops b"
    );
    assert_eq!(packages.metrics().cache_misses, 1);
}

#[test]
fn registration_closure_resolves_its_package_imports_with_the_frontier() {
    CLOSURE_PACKAGE_LOOKUPS.store(0, Ordering::SeqCst);
    let root = tempfile::tempdir().unwrap();
    let base = root.path();
    let barrel = write(base, "lib/barrel.ts", "export * from './leaf';\n");
    write(
        base,
        "lib/leaf.ts",
        "import type { Public } from 'pkg';\nexport type Leaf = Public;\n",
    );
    write_pkg(base);

    let mut packages = PackageRouteResolver::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let mut session = LocalImportSession::new(&mut packages);
    session.package_lookup_probe = Some(note_closure_package_lookup);
    let barrel = canonical_paths.canonicalize(&barrel);
    let mut discovery = registration::VirtualRegistrationDiscovery::default();
    let needs_registration = registration::non_relative_import_needs_virtual_registration(
        &barrel,
        &mut canonical_paths,
        ImportFileOptions::from(false),
        None,
        Some(&mut packages),
        &mut session,
        &mut discovery,
    );

    assert!(!needs_registration, "the closure reaches no Vue source");
    assert_eq!(
        CLOSURE_PACKAGE_LOOKUPS.load(Ordering::SeqCst),
        1,
        "leaf's package import went through the frontier, off the serial path"
    );
    assert_eq!(session.package_lookup_entries(), 1);
    assert_eq!(packages.metrics().cache_misses, 1);
}

#[test]
fn alias_targets_stay_with_the_configuration_that_resolved_them() {
    let case = tempfile::tempdir().unwrap();
    let base = case.path().canonicalize().unwrap();
    let left_target = write(&base, "left/src/x.ts", "export const x = 1;\n");
    let right_target = write(&base, "right/src/x.ts", "export const x = 2;\n");
    let left_main = write(&base, "left/main.ts", "import { x } from '@/x';\n");
    let right_main = write(&base, "right/main.ts", "import { x } from '@/x';\n");
    for side in ["left", "right"] {
        write(
            &base,
            &format!("{side}/tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["src/*"]}}}"#,
        );
    }
    let left = PathAliasResolver::from_tsconfig(Some(&base.join("left/tsconfig.json")));
    let right = PathAliasResolver::from_tsconfig(Some(&base.join("right/tsconfig.json")));
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::new(&mut packages);
    let mut canonical_paths = CanonicalPathCache::default();

    let first = collect_transitive_local_imports_with_session(
        &[left_main],
        &base,
        &mut canonical_paths,
        false,
        Some(&left),
        &mut packages,
        &mut session,
    );
    let second = collect_transitive_local_imports_with_session(
        &[right_main],
        &base,
        &mut canonical_paths,
        false,
        Some(&right),
        &mut packages,
        &mut session,
    );

    assert_eq!(sorted(first.authored), vec![left_target]);
    assert_eq!(
        sorted(second.authored),
        vec![right_target],
        "a walk under another `paths` configuration must not reuse the first walk's `@/x`"
    );
}
