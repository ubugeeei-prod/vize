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

#[test]
fn shared_import_session_scans_each_file_once_across_walks() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    let main = write(
        base.as_path(),
        "src/main.ts",
        "import { helper } from './helper';\nexport const value = helper;\n",
    );
    let helper = write(
        base.as_path(),
        "src/helper.ts",
        "import type { Shape } from './shape';\nexport const helper: Shape = {};\n",
    );
    let shape = write(base.as_path(), "src/shape.ts", "export type Shape = {};\n");
    let walk = |roots: &[PathBuf],
                session: &mut LocalImportSession,
                resolver: &mut PackageRouteResolver| {
        collect_transitive_local_imports_with_session(
            roots,
            base.as_path(),
            &mut CanonicalPathCache::default(),
            false,
            None,
            resolver,
            session,
        )
    };

    let mut resolver = PackageRouteResolver::default();
    let mut session = LocalImportSession::new(&mut resolver);
    let first = walk(std::slice::from_ref(&main), &mut session, &mut resolver);
    assert_eq!(first.authored, vec![helper.clone(), shape.clone()]);
    assert_eq!(session.module_specifier_entries(), 3);

    // A default run walks its roots again after adding hidden ambient files.
    let roots = [main.clone(), helper.clone()];
    let second = walk(&roots, &mut session, &mut resolver);
    assert_eq!(session.module_specifier_entries(), 3);
    let mut fresh_resolver = PackageRouteResolver::default();
    let mut fresh_session = LocalImportSession::new(&mut fresh_resolver);
    let fresh = walk(&roots, &mut fresh_session, &mut fresh_resolver);
    assert_eq!(second.registrations, fresh.registrations);
    assert_eq!(second.authored, fresh.authored);
    assert_eq!(second.authored, vec![shape]);
}
