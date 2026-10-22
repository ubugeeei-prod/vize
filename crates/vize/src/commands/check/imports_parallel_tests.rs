use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use vize_canon::PackageRouteResolver;

use super::super::super::imports_aliases::PathAliasResolver;
use super::super::super::path_cache::CanonicalPathCache;
use super::super::{LocalImportSession, collect_transitive_local_imports_with_session};

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
        &[main, other],
        base,
        &mut CanonicalPathCache::default(),
        false,
        Some(&aliases),
        &mut packages,
        &mut session,
    );

    let leaf = leaf.canonicalize().unwrap();
    let shared_file = shared_file.canonicalize().unwrap();
    assert_eq!(
        sorted(discovered.authored),
        sorted(vec![leaf.clone(), shared_file])
    );
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
}
