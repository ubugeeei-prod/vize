//! Complete projection equality and strong invalidation for warm editor surfaces.

use std::path::Path;

use crate::virtual_ts::VirtualTsOptions;

use super::*;
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

const HOST: &str = "<script setup lang='ts'>import Child from './Child.vue'; const title = '日本語 🦀';</script><template><Child :title=\"title\" /></template>";
const CHILD: &str = "<script setup lang='ts'>defineProps<{ title: string }>();</script><template><pre>{{ title }}</pre></template>";

fn environment(session: &EditorMirrorSession) -> CorsaProjectEnvironment<'_> {
    // Static defaults keep this test helper's returned borrow explicit.
    static OPTIONS: std::sync::LazyLock<VirtualTsOptions> =
        std::sync::LazyLock::new(VirtualTsOptions::default);
    static ROUTES: std::sync::LazyLock<crate::PackageRouteResolver> =
        std::sync::LazyLock::new(crate::PackageRouteResolver::default);
    CorsaProjectEnvironment {
        virtual_ts_options: &OPTIONS,
        package_routes: &ROUTES,
        project_root: None,
        tsconfig_path: None,
        editor_session: session,
    }
}

fn prepare(
    session: &EditorMirrorSession,
    path: &Path,
    source: &str,
    overlays: &[(PathBuf, &str)],
    requested: &[(PathBuf, &str)],
) -> CorsaVueVirtualProject {
    build_vue_virtual_workspace_project(
        path,
        source,
        Default::default(),
        overlays,
        requested,
        environment(session),
    )
    .unwrap()
}

fn context(
    session: &EditorMirrorSession,
    path: &Path,
    source: &str,
    overlays: &[(PathBuf, &str)],
) -> impl std::ops::Deref<Target = AliasContext> {
    let overlays = overlays.iter().map(|(p, s)| (p.clone(), *s)).collect();
    AliasContext::for_hosts_cached(
        path,
        source,
        &overlays,
        &[],
        Default::default(),
        environment(session),
    )
    .unwrap()
}

fn complete_surface(actual: &QuerySurface, expected: &QuerySurface) {
    // Keep every public projection field and all ordered dependency packets.
    let host = |s: &QuerySurface| {
        let h = &s.host;
        (
            h.request_uri.clone(),
            h.code.clone(),
            h.pre_rewrite_code.clone(),
            h.mapping.clone(),
            h.import_source_map.clone(),
            h.source_type,
            h.virtual_suffix,
            h.resolved_dependencies.clone(),
            h.session_project_root.clone(),
        )
    };
    assert_eq!(host(actual), host(expected));
    let deps = |s: &QuerySurface| {
        s.host
            .dependencies
            .iter()
            .map(|d| {
                (
                    d.source_path.clone(),
                    d.source.clone(),
                    d.request_uri.clone(),
                    d.code.clone(),
                    d.mapping.clone(),
                    d.import_source_map.clone(),
                    d.source_type,
                    d.virtual_suffix,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(deps(actual), deps(expected));
    let sources = |s: &QuerySurface| {
        s.host
            .materialized_sources
            .iter()
            .map(|d| {
                (
                    d.materialized_path.clone(),
                    d.source_path.clone(),
                    d.source.clone(),
                    d.code.clone(),
                    d.mapping.clone(),
                    d.import_source_map.clone(),
                    d.mapping_kind,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(sources(actual), sources(expected));
    assert_eq!(actual.documents, expected.documents);
    assert_eq!(actual.session_config_path, expected.session_config_path);
}

#[test]
fn warm_surface_matches_uncached_whole_projection_and_retains_catalog() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    std::fs::write(&host, HOST).unwrap();
    std::fs::write(&child, CHILD).unwrap();
    let session = EditorMirrorSession::new();
    let first = prepare(&session, &host, HOST, &[], &[]);
    let initial = context(&session, &host, HOST, &[]);
    let surface = initial.query_surface.get().unwrap();
    let fresh = build_surface(
        &host,
        HOST,
        Default::default(),
        &Default::default(),
        &[],
        environment(&session),
        &ImportRewriter::new(),
        &initial,
    )
    .unwrap();
    complete_surface(surface, &fresh);
    for _ in 0..5 {
        let warm = prepare(&session, &host, HOST, &[], &[]);
        let current = context(&session, &host, HOST, &[]);
        assert!(std::ptr::eq(surface, current.query_surface.get().unwrap()));
        assert!(
            first
                .host
                .source_catalog
                .shares_revision_with(&warm.host.source_catalog)
        );
        complete_surface(current.query_surface.get().unwrap(), &fresh);
        assert_eq!(first.host.code, warm.host.code);
        assert!(warm.materialized_changes.is_empty());
    }
}

#[test]
fn same_mtime_dependency_edit_and_overlay_close_reopen_replace_whole_surface() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    std::fs::write(&host, HOST).unwrap();
    std::fs::write(&child, CHILD).unwrap();
    let session = EditorMirrorSession::new();
    prepare(&session, &host, HOST, &[], &[]);
    let first = context(&session, &host, HOST, &[]);
    let modified = std::fs::metadata(&child).unwrap().modified().unwrap();
    let edited = CHILD.replace("string", "number");
    assert_eq!(edited.len(), CHILD.len());
    std::fs::write(&child, &edited).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&child)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    let changed = prepare(&session, &host, HOST, &[], &[]);
    let second = context(&session, &host, HOST, &[]);
    assert!(!std::ptr::eq(
        first.query_surface.get().unwrap(),
        second.query_surface.get().unwrap()
    ));
    assert_eq!(changed.host.dependencies[0].source, edited);
    let overlays = [(child.clone(), CHILD)];
    let opened = prepare(&session, &host, HOST, &overlays, &[]);
    assert_eq!(opened.host.dependencies[0].source, CHILD);
    let closed = prepare(&session, &host, HOST, &[], &[]);
    assert_eq!(closed.host.dependencies[0].source, edited);
    let reopened = prepare(&session, &host, HOST, &overlays, &[]);
    assert_eq!(reopened.host.dependencies[0].source, CHILD);
    let new_process = EditorMirrorSession::new();
    let independent = prepare(&new_process, &host, HOST, &overlays, &[]);
    assert!(
        !reopened
            .host
            .source_catalog
            .shares_revision_with(&independent.host.source_catalog)
    );
    assert_ne!(reopened.host.request_uri, independent.host.request_uri);
}

#[test]
fn config_source_options_and_requested_membership_keep_distinct_revisions() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    std::fs::write(&host, HOST).unwrap();
    std::fs::write(&child, CHILD).unwrap();
    let session = EditorMirrorSession::new();
    prepare(&session, &host, HOST, &[], &[]);
    let first = context(&session, &host, HOST, &[]);
    let config = root.path().join("tsconfig.json");
    let strict = "{\"compilerOptions\":{\"strict\":true }}";
    let relaxed = "{\"compilerOptions\":{\"strict\":false}}";
    assert_eq!(strict.len(), relaxed.len());
    std::fs::write(&config, strict).unwrap();
    prepare(&session, &host, HOST, &[], &[]);
    let configured = context(&session, &host, HOST, &[]);
    assert!(!std::ptr::eq(
        first.query_surface.get().unwrap(),
        configured.query_surface.get().unwrap()
    ));
    let modified = std::fs::metadata(&config).unwrap().modified().unwrap();
    std::fs::write(&config, relaxed).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&config)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    prepare(&session, &host, HOST, &[], &[]);
    let reconfigured = context(&session, &host, HOST, &[]);
    assert!(!std::ptr::eq(
        configured.query_surface.get().unwrap(),
        reconfigured.query_surface.get().unwrap()
    ));
    let unrelated = root.path().join("Other.vue");
    std::fs::write(&unrelated, "<template>other</template>").unwrap();
    let requested = prepare(
        &session,
        &host,
        HOST,
        &[],
        &[(unrelated, "<template>other</template>")],
    );
    assert!(
        requested
            .host
            .materialized_sources
            .iter()
            .any(|s| s.source == "<template>other</template>")
    );
    let ordinary = prepare(&session, &host, HOST, &[], &[]);
    assert!(
        !ordinary
            .host
            .materialized_sources
            .iter()
            .any(|s| s.source == "<template>other</template>")
    );
    let options = CorsaVueVirtualDocumentOptions {
        preserve_event_navigation: true,
        ..Default::default()
    };
    let optioned =
        build_vue_virtual_workspace_project(&host, HOST, options, &[], &[], environment(&session))
            .unwrap();
    assert_ne!(ordinary.host.request_uri, optioned.host.request_uri);
}

#[test]
fn host_edit_reuses_dependency_documents_instead_of_copying_the_graph() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    std::fs::write(&host, HOST).unwrap();
    std::fs::write(&child, CHILD).unwrap();
    let session = EditorMirrorSession::new();
    drop(prepare(&session, &host, HOST, &[], &[]));
    let initial = context(&session, &host, HOST, &[]);
    let surface = initial.query_surface.get().expect("surface");
    let original_code = surface.host.code.clone();
    let host_uri = surface.host.request_uri.clone();
    let dependency_slots = surface
        .documents
        .iter()
        .enumerate()
        .filter(|(_, (uri, _))| uri != &host_uri)
        .map(|(index, document)| (index, document as *const _))
        .collect::<Vec<_>>();
    assert!(
        !dependency_slots.is_empty(),
        "the child document is part of the prepared graph"
    );
    let dependency_code = surface.host.dependencies[0].code.as_str().as_ptr();
    let child_materialized = surface
        .host
        .materialized_sources
        .iter()
        .find(|source| source.source_path == child.canonicalize().unwrap())
        .expect("child mirror document")
        .code
        .as_str()
        .as_ptr();
    drop(initial);

    let edited = HOST.replace("日本語 🦀", "changed 🦀");
    drop(prepare(&session, &host, &edited, &[], &[]));
    let revised = context(&session, &host, &edited, &[]);
    let surface = revised.query_surface.get().expect("revised surface");
    assert_ne!(surface.host.code, original_code);
    assert_eq!(
        surface
            .host
            .materialized_sources
            .iter()
            .find(|source| source.source_path == host.canonicalize().unwrap())
            .expect("edited host mirror document")
            .source,
        edited
    );
    for (index, previous) in dependency_slots {
        assert!(
            std::ptr::eq(previous, &surface.documents[index]),
            "unchanged dependency documents stay in the prepared graph"
        );
    }
    assert_eq!(
        surface.host.dependencies[0].code.as_str().as_ptr(),
        dependency_code
    );
    assert_eq!(
        surface
            .host
            .materialized_sources
            .iter()
            .find(|source| source.source_path == child.canonicalize().unwrap())
            .expect("child mirror document")
            .code
            .as_str()
            .as_ptr(),
        child_materialized
    );
    session.clear();
    drop(prepare(&session, &host, &edited, &[], &[]));
    let cold = context(&session, &host, &edited, &[]);
    complete_surface(surface, cold.query_surface.get().expect("cold surface"));
}

#[test]
fn identical_source_in_separate_roots_keeps_authored_membership() {
    let roots = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    let session = EditorMirrorSession::new();
    let mut projects = Vec::new();
    for root in &roots {
        let host = root.path().join("Host.vue");
        std::fs::write(&host, HOST).unwrap();
        std::fs::write(root.path().join("Child.vue"), CHILD).unwrap();
        projects.push(prepare(&session, &host, HOST, &[], &[]));
    }
    assert_ne!(projects[0].host.request_uri, projects[1].host.request_uri);
    for (project, root) in projects.iter().zip(&roots) {
        assert_eq!(project.host.dependencies[0].source, CHILD);
        assert_eq!(
            project.host.dependencies[0].source_path,
            root.path().join("Child.vue").canonicalize().unwrap()
        );
        let mut members = project
            .host
            .materialized_sources
            .iter()
            .map(|source| source.source_path.clone())
            .collect::<Vec<_>>();
        members.sort();
        let mut expected = vec![
            root.path().join("Child.vue").canonicalize().unwrap(),
            root.path().join("Host.vue").canonicalize().unwrap(),
        ];
        expected.sort();
        assert_eq!(members, expected);
    }
}

#[test]
fn fingerprint_hit_hashes_changed_files_and_a_miss_hashes_the_mirror() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    std::fs::write(&host, HOST).unwrap();
    std::fs::write(&child, CHILD).unwrap();
    let session = EditorMirrorSession::new();
    let _ = crate::batch::virtual_project::take_materialized_hash_count();
    let cold = prepare(&session, &host, HOST, &[], &[]);
    let cold_hashes = crate::batch::virtual_project::take_materialized_hash_count();
    assert!(
        cold_hashes > 3,
        "a cold build hashes every mirror file ({cold_hashes})"
    );
    let child_path = cold
        .host
        .materialized_sources
        .iter()
        .find(|source| source.source_path == child.canonicalize().unwrap())
        .expect("child mirror")
        .materialized_path
        .clone();
    let host_path = cold
        .host
        .materialized_sources
        .iter()
        .find(|source| source.source_path == host.canonicalize().unwrap())
        .expect("host mirror")
        .materialized_path
        .clone();
    let child_bytes = std::fs::read(&child_path).unwrap();
    drop(cold);

    let edited = HOST.replace("日本語 🦀", "changed 🦀");
    let hit = prepare(&session, &host, &edited, &[], &[]);
    let hit_hashes = crate::batch::virtual_project::take_materialized_hash_count();
    assert!(hit_hashes > 0, "the edited host file is hashed");
    assert!(
        hit_hashes < cold_hashes,
        "a fingerprint hit hashed {hit_hashes} files; the cold mirror hashed {cold_hashes}"
    );
    assert_eq!(std::fs::read(&child_path).unwrap(), child_bytes);
    let host_on_disk = std::fs::read_to_string(&host_path).unwrap();
    assert_eq!(host_on_disk, hit.host.code);
    assert_eq!(
        hit.host
            .materialized_sources
            .iter()
            .find(|source| source.source_path == host.canonicalize().unwrap())
            .expect("edited host mirror document")
            .source,
        edited
    );
    assert_eq!(
        hit.materialized_changes,
        crate::batch::virtual_project::MaterializedFileDelta {
            changed: vec![host_path],
            ..Default::default()
        },
        "only the edited host changes; the child and graph topology remain intact"
    );
    let revised = context(&session, &host, &edited, &[]);
    drop(hit);
    session.clear();
    drop(prepare(&session, &host, &edited, &[], &[]));
    let rebuilt = context(&session, &host, &edited, &[]);
    complete_surface(
        revised.query_surface.get().expect("hit surface"),
        rebuilt.query_surface.get().expect("cold surface"),
    );

    let session = EditorMirrorSession::new();
    let _ = crate::batch::virtual_project::take_materialized_hash_count();
    drop(prepare(&session, &host, &edited, &[], &[]));
    let warm_hashes = crate::batch::virtual_project::take_materialized_hash_count();
    std::fs::write(&child, CHILD.replace("title", "missTitle")).unwrap();
    let _ = crate::batch::virtual_project::take_materialized_hash_count();
    let miss = prepare(&session, &host, &edited, &[], &[]);
    let miss_hashes = crate::batch::virtual_project::take_materialized_hash_count();
    assert_eq!(
        miss_hashes, warm_hashes,
        "a fingerprint miss still hashes the whole mirror"
    );
    assert_eq!(
        miss.host.dependencies[0].source,
        CHILD.replace("title", "missTitle")
    );
    let missed = context(&session, &host, &edited, &[]);
    drop(miss);
    session.clear();
    drop(prepare(&session, &host, &edited, &[], &[]));
    let cold_miss = context(&session, &host, &edited, &[]);
    complete_surface(
        missed.query_surface.get().expect("miss surface"),
        cold_miss.query_surface.get().expect("cold miss surface"),
    );
}

#[cfg(test)]
mod native_union;

#[cfg(windows)]
mod windows_overlays;

mod shared_payload;

mod host_overlay;
