use super::*;

fn write(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn reuses_non_vue_package_closure_classification_across_importers() {
    let root = tempfile::tempdir().unwrap();
    let first = write(
        root.path(),
        "src/first.ts",
        "import type { Public } from 'pkg';\nexport type First = Public;\n",
    );
    let second = write(
        root.path(),
        "src/second.ts",
        "import type { Public } from 'pkg';\nexport type Second = Public;\n",
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/index.d.ts",
        "export type { Dep as Public } from 'dep';\n",
    );
    write(
        root.path(),
        "node_modules/dep/package.json",
        r#"{"name":"dep","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/dep/index.d.ts",
        "export interface Dep { value: string }\n",
    );

    let mut resolver = PackageRouteResolver::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let mut session = LocalImportSession::new(&mut resolver);
    let discovered = collect_transitive_local_imports_with_session(
        &[first, second],
        root.path(),
        &mut canonical_paths,
        false,
        None,
        &mut resolver,
        &mut session,
    );

    assert!(discovered.registrations.is_empty());
    assert!(discovered.package_routes.is_empty());
    let metrics = resolver.metrics();
    assert_eq!(metrics.cache_misses, 2, "pkg and its dep resolve once");
    assert_eq!(
        metrics.cache_hits, 0,
        "the coherent collection snapshot bypasses repeated resolver validation"
    );
}

#[test]
fn shared_import_session_reuses_package_lookup_across_walks() {
    let root = tempfile::tempdir().unwrap();
    let first = write(
        root.path(),
        "src/first.ts",
        "import type { Public } from 'pkg';\nexport type First = Public;\n",
    );
    let second = write(
        root.path(),
        "src/second.ts",
        "import type { Public } from 'pkg';\nexport type Second = Public;\n",
    );
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/index.d.ts",
        "export interface Public { value: string }\n",
    );

    let mut resolver = PackageRouteResolver::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let mut session = LocalImportSession::new(&mut resolver);
    let first_discovered = collect_transitive_local_imports_with_session(
        &[first],
        root.path(),
        &mut canonical_paths,
        false,
        None,
        &mut resolver,
        &mut session,
    );
    assert!(first_discovered.registrations.is_empty());
    assert_eq!(session.package_lookup_entries(), 1);
    let first_metrics = resolver.metrics();

    let second_discovered = collect_transitive_local_imports_with_session(
        &[second],
        root.path(),
        &mut canonical_paths,
        false,
        None,
        &mut resolver,
        &mut session,
    );
    assert!(second_discovered.registrations.is_empty());
    let second_metrics = resolver.metrics();

    assert_eq!(
        second_metrics.cache_misses, first_metrics.cache_misses,
        "the second import walk should reuse the package lookup from the first"
    );
    assert_eq!(
        second_metrics.cache_hits, first_metrics.cache_hits,
        "the shared session bypasses resolver validation on repeated lookups"
    );
}
