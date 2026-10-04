use super::*;

fn write(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    std::fs::write(&path, content).unwrap();
    path.canonicalize().unwrap()
}

fn walk(
    roots: &[PathBuf],
    root: &Path,
    paths: &mut CanonicalPathCache,
    packages: &mut PackageRouteResolver,
    session: &mut LocalImportSession,
) -> TransitiveLocalImports {
    collect_transitive_local_imports_with_session(
        roots, root, paths, false, None, packages, session,
    )
}

#[test]
fn reused_session_observes_same_length_same_mtime_root_edits() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path();
    let first_source = "import Child from './Old.vue'; export default Child;";
    let second_source = "import Child from './New.vue'; export default Child;";
    assert_eq!(first_source.len(), second_source.len());
    let entry = write(root, "entry.ts", first_source);
    let old = write(root, "Old.vue", "<template />");
    let new = write(root, "New.vue", "<template />");
    let original_mtime = std::fs::metadata(&entry).unwrap().modified().unwrap();
    let mut paths = CanonicalPathCache::default();
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::with_source_reuse(&mut packages);
    assert_eq!(
        walk(
            std::slice::from_ref(&entry),
            root,
            &mut paths,
            &mut packages,
            &mut session
        )
        .registrations,
        vec![old]
    );
    std::fs::write(&entry, second_source).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&entry)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(original_mtime))
        .unwrap();
    assert_eq!(
        std::fs::metadata(&entry).unwrap().modified().unwrap(),
        original_mtime
    );
    assert_eq!(
        walk(
            std::slice::from_ref(&entry),
            root,
            &mut paths,
            &mut packages,
            &mut session
        )
        .registrations,
        vec![new]
    );
}

#[test]
fn cached_occurrences_resolve_newly_created_import_targets() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path();
    let entry = write(root, "entry.ts", "import Child from './Created.vue';");
    let mut paths = CanonicalPathCache::default();
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::with_source_reuse(&mut packages);
    assert!(
        walk(
            std::slice::from_ref(&entry),
            root,
            &mut paths,
            &mut packages,
            &mut session
        )
        .registrations
        .is_empty()
    );
    let created = write(root, "Created.vue", "<template />");
    assert_eq!(
        walk(
            std::slice::from_ref(&entry),
            root,
            &mut paths,
            &mut packages,
            &mut session
        )
        .registrations,
        vec![created]
    );
    assert_eq!(session.source_occurrences.counts(), (2, 1));
}

#[test]
fn failed_reads_do_not_keep_cached_sources_or_negative_entries() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path();
    let path = root.join("entry.ts");
    let mut cache = session::SourceOccurrenceCache::with_reuse();
    assert!(cache.occurrences(&path).is_err());
    std::fs::write(&path, "import './Old.vue';").unwrap();
    assert_eq!(
        cache.occurrences(&path).unwrap()[0].specifier.as_str(),
        "./Old.vue"
    );
    std::fs::remove_file(&path).unwrap();
    assert!(cache.occurrences(&path).is_err());
    std::fs::write(&path, "import './Old.vue';").unwrap();
    assert_eq!(
        cache.occurrences(&path).unwrap()[0].specifier.as_str(),
        "./Old.vue"
    );
    assert_eq!(cache.counts(), (2, 0));
}

#[test]
fn cached_lexical_occurrences_preserve_every_resolution_mode() {
    use vize_canon::PackageResolutionMode::{Contextual, Import, Require};
    let case = tempfile::tempdir().unwrap();
    let source = r#"<script setup lang="ts">
import value from 'same-package';
const dynamic = import('same-package');
const commonjs = require('same-package');
import type { Public } from 'same-package' with { "resolution-mode": "require" };
// import('ignored-package')
const malformed = import(/* webpackChunkName: "view" */ 'same-package'
</script><template />"#;
    let path = write(case.path(), "entry.vue", source);
    let mut cache = session::SourceOccurrenceCache::with_reuse();
    let first = cache.occurrences(&path).unwrap();
    let second = cache.occurrences(&path).unwrap();
    assert_eq!(&*first, extract_module_specifier_occurrences(source));
    assert_eq!(&*second, &*first);
    assert_eq!(
        first
            .iter()
            .map(|occurrence| occurrence.mode)
            .collect::<Vec<_>>(),
        vec![Contextual, Import, Require, Require, Import]
    );
    assert_eq!(cache.counts(), (1, 1));

    let mut one_pass = session::SourceOccurrenceCache::default();
    assert_eq!(one_pass.occurrences(&path).unwrap(), *first);
    assert_eq!(one_pass.occurrences(&path).unwrap(), *first);
    assert_eq!(one_pass.counts(), (2, 0));
}

#[test]
fn repeated_program_walks_scan_each_source_once_with_fresh_resolution() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path();
    let shared = write(root, "shared.ts", "export const value = 1;");
    let mut roots = (0..500)
        .map(|index| {
            write(
                root,
                &cstr!("Comp{index}.vue"),
                "<script setup lang=\"ts\">import { value } from './shared';</script><template>{{ value }}</template>",
            )
        })
        .collect::<Vec<_>>();
    let mut paths = CanonicalPathCache::default();
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::with_source_reuse(&mut packages);
    let first = walk(&roots, root, &mut paths, &mut packages, &mut session);
    assert_eq!(first.registrations, vec![shared]);
    roots.extend(first.registrations);
    let second = walk(&roots, root, &mut paths, &mut packages, &mut session);
    assert!(second.registrations.is_empty());
    assert_eq!(session.source_occurrences.counts(), (501, 501));
}
