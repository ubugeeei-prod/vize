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
    let discovered = collect_transitive_local_imports_with_resolver(
        &[first, second],
        root.path(),
        &mut CanonicalPathCache::default(),
        false,
        None,
        &mut resolver,
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
