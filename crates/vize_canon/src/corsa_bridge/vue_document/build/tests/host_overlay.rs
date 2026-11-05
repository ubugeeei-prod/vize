//! Same-graph host edits through the normal unsaved editor overlay path.

use super::*;

fn ordered_native_packet(project: &CorsaVueVirtualProject) -> Vec<(String, String)> {
    let mut original = project.documents.to_vec();
    let mut shared = project.documents.clone();
    let root = project.session_project_root.as_deref().unwrap();
    project
        .host
        .source_catalog
        .extend_native_project_documents(root, &mut original);
    project
        .host
        .source_catalog
        .extend_native_project_document_batch(root, &mut shared);
    assert_eq!(shared.to_vec(), original);
    original
}

#[test]
fn own_unsaved_overlay_edit_retains_dependencies_and_hashes_only_the_host() {
    const CHILD_WITH_LEAF: &str = "<script setup lang='ts'>import Leaf from './Leaf.vue'; defineProps<{ title: string }>();</script><template><Leaf /><pre>{{ title }}</pre></template>";
    const LEAF: &str = "<script setup lang='ts'>const leaf = '依存 😀';</script>\r\n<template>{{ leaf }}</template>";
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("A.vue");
    let child = root.join("Child.vue");
    let leaf = root.join("Leaf.vue");
    for (path, source) in [(&host, HOST), (&child, CHILD_WITH_LEAF), (&leaf, LEAF)] {
        std::fs::write(path, source).unwrap();
    }
    let session = EditorMirrorSession::new();
    session.trust_watched_disk_inputs();
    let overlays = [(host.clone(), HOST)];
    let _ = crate::batch::virtual_project::take_materialized_hash_count();
    let first = prepare(&session, &host, HOST, &overlays, &[]);
    let cold_hashes = crate::batch::virtual_project::take_materialized_hash_count();
    let mut expected_native = ordered_native_packet(&first);
    let host_path = first
        .host
        .materialized_sources
        .iter()
        .find(|source| source.source_path == host)
        .unwrap()
        .materialized_path
        .clone();
    drop(first);
    let initial = context(&session, &host, HOST, &overlays);
    assert_eq!(
        cold_hashes,
        initial
            .mirror
            .as_ref()
            .unwrap()
            .expected_materialized_files()
            .len(),
        "the cold observation hashes the entire mirror"
    );
    let surface = initial.query_surface.get().unwrap();
    let mut expected_host = surface.host.clone();
    let mut expected_documents = surface.documents.to_vec();
    let expected_config = surface.session_config_path.clone();
    let host_uri = surface.host.request_uri.clone();
    let dependency_slots = surface
        .documents
        .iter()
        .enumerate()
        .filter(|(_, (uri, _))| uri != &host_uri)
        .map(|(index, packet)| (index, packet as *const _))
        .collect::<Vec<_>>();
    assert_eq!(
        surface
            .host
            .dependencies
            .iter()
            .map(|source| source.source_path.clone())
            .collect::<Vec<_>>(),
        vec![child, leaf]
    );
    let dependency_allocations = surface
        .host
        .dependencies
        .iter()
        .map(|source| source.code.as_str().as_ptr())
        .collect::<Vec<_>>();
    let mirror_allocations = surface
        .host
        .materialized_sources
        .iter()
        .filter(|source| source.source_path != host)
        .map(|source| {
            (
                source.materialized_path.clone(),
                source.code.as_str().as_ptr(),
                std::fs::read(&source.materialized_path).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    drop(initial);

    let edited = HOST
        .replace("日本語 🦀", "changed 雪😀")
        .replace("</script>", "\r\n</script>\r\n");
    let overlays = [(host.clone(), edited.as_str())];
    let _ = crate::batch::virtual_project::take_materialized_hash_count();
    let changed = prepare(&session, &host, &edited, &overlays, &[]);
    assert_eq!(
        crate::batch::virtual_project::take_materialized_hash_count(),
        1,
        "the ordinary host overlay hashes only its changed artifact"
    );
    assert_eq!(
        changed.materialized_changes,
        crate::batch::virtual_project::MaterializedFileDelta {
            changed: vec![host_path.clone()],
            ..Default::default()
        }
    );
    assert_eq!(std::fs::read_to_string(&host).unwrap(), HOST);
    assert_eq!(
        std::fs::read_to_string(&host_path).unwrap(),
        changed.host.code
    );
    let native = ordered_native_packet(&changed);
    for (uri, code) in &mut expected_native {
        if uri == &host_uri {
            *code = changed.host.code.clone();
        }
    }
    assert_eq!(native, expected_native, "the whole ordered native packet");
    drop(changed);
    let revised = context(&session, &host, &edited, &overlays);
    let surface = revised.query_surface.get().unwrap();
    for (index, previous) in dependency_slots {
        assert!(std::ptr::eq(previous, &surface.documents[index]));
    }
    assert_eq!(
        surface
            .host
            .dependencies
            .iter()
            .map(|source| source.code.as_str().as_ptr())
            .collect::<Vec<_>>(),
        dependency_allocations
    );
    for (path, allocation, bytes) in mirror_allocations {
        let retained = surface
            .host
            .materialized_sources
            .iter()
            .find(|source| source.materialized_path == path)
            .unwrap();
        assert_eq!(retained.code.as_str().as_ptr(), allocation);
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    // Only the host's regenerated fields may differ. All dependency source,
    // code, maps, membership, configuration and ordered tuples stay exact.
    assert_ne!(expected_host.code, surface.host.code);
    expected_host.code = surface.host.code.clone();
    expected_host.pre_rewrite_code = surface.host.pre_rewrite_code.clone();
    expected_host.mapping = surface.host.mapping.clone();
    expected_host.import_source_map = surface.host.import_source_map.clone();
    let current_host = surface
        .host
        .materialized_sources
        .iter()
        .find(|source| source.source_path == host)
        .unwrap();
    assert_eq!(current_host.source, edited);
    *expected_host
        .materialized_sources
        .iter_mut()
        .find(|source| source.source_path == host)
        .unwrap() = current_host.clone();
    for (uri, code) in &mut expected_documents {
        if uri == &host_uri {
            *code = surface.host.code.clone();
        }
    }
    complete_surface(
        surface,
        &QuerySurface {
            host: expected_host,
            documents: expected_documents.into(),
            session_config_path: expected_config,
        },
    );

    session.clear();
    let cold = prepare(&session, &host, &edited, &overlays, &[]);
    assert_eq!(native, ordered_native_packet(&cold));
    let cold_context = context(&session, &host, &edited, &overlays);
    complete_surface(surface, cold_context.query_surface.get().unwrap());
}
