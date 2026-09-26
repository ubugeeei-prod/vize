#![expect(
    clippy::disallowed_types,
    reason = "test project snapshots use the bridge's shared Arc"
)]
#![expect(clippy::string_slice, reason = "tests verify exact mapped byte ranges")]

use std::path::{Path, PathBuf};
use vize_carton::{FxHashMap, cstr};

use super::super::{AliasContext, PreparedAliasContext};
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_document::{CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions};

const VUE: &str = "<script setup lang='ts'>const label = 'é'; const token = 41;</script><template>{{ token }}</template>";

fn open(
    session: &EditorMirrorSession,
    path: &Path,
    source: &str,
    overlays: &FxHashMap<PathBuf, &str>,
) -> PreparedAliasContext {
    AliasContext::for_host_cached(
        path,
        source,
        overlays,
        CorsaVueVirtualDocumentOptions {
            jsx_typecheck: true,
            ..Default::default()
        },
        CorsaProjectEnvironment {
            virtual_ts_options: &Default::default(),
            package_routes: &Default::default(),
            project_root: None,
            tsconfig_path: None,
            editor_session: session,
        },
    )
    .unwrap()
}

fn mapped_token(context: &AliasContext, path: &Path, source: &str) {
    let documents = context.materialized_sources();
    let document = documents
        .iter()
        .find(|document| document.source_path == path && document.mapping_kind.is_mappable())
        .unwrap();
    assert_eq!(document.source.as_str(), source);
    let offset = source.find("token").unwrap();
    let before_rewrite = document
        .mapping
        .spans()
        .iter()
        .filter(|span| offset >= span.src_range.start && offset < span.src_range.end)
        .min_by_key(|span| span.src_range.end - span.src_range.start)
        .map_or(offset, |span| {
            span.gen_range.start + offset - span.src_range.start
        });
    let generated = document
        .import_source_map
        .get_virtual_offset(before_rewrite as u32) as usize;
    assert_eq!(&document.code[generated..generated + 5], "token");
}

#[test]
fn distinct_hosts_keep_bounded_contexts_and_linear_member_source_facts() {
    let root = tempfile::tempdir().unwrap();
    let root_path = root.path().canonicalize().unwrap();
    std::fs::write(root_path.join("tsconfig.json"), "{\"include\":[\"**/*\"]}").unwrap();
    let hosts: Vec<_> = (0..24)
        .map(|index| {
            let (extension, source) = match index % 3 {
                0 => ("vue", VUE),
                1 => (
                    "tsx",
                    "const label = 'é'; export const token = () => <div />;",
                ),
                _ => ("ts", "const label = 'é'; export const token = 41;"),
            };
            let path = root_path.join(cstr!("Host{index}.{extension}"));
            std::fs::write(&path, source).unwrap();
            (path, source)
        })
        .collect();
    let overlays: FxHashMap<_, _> = hosts
        .iter()
        .map(|(path, source)| (path.clone(), *source))
        .collect();
    let session = EditorMirrorSession::new();
    let mut mirror_root = None;
    let mut query_paths = Vec::new();
    for (path, source) in &hosts {
        let context = open(&session, path, source, &overlays);
        let mirror = context.mirror.as_ref().unwrap();
        assert_eq!(
            mirror.registered_original_paths_sorted(),
            vec![path.clone()]
        );
        mapped_token(&context, path, source);
        mirror_root = Some(mirror.virtual_root().to_path_buf());
        query_paths.push(
            mirror
                .preferred_materialized_path_for_original(path)
                .unwrap(),
        );
    }
    let cache = session.cache();
    let members = &cache.project_members[&mirror_root.unwrap()];
    assert_eq!(cache.slots.len(), 8);
    assert_eq!(members.len(), 24);
    assert_eq!(
        members
            .values()
            .map(|member| member.source_paths.len())
            .sum::<usize>(),
        24
    );
    assert_eq!(
        query_paths
            .iter()
            .map(|path| path.is_file())
            .collect::<Vec<_>>(),
        vec![true; 24]
    );
    drop(cache);
    let reopened = open(&session, &hosts[0].0, hosts[0].1, &overlays);
    assert_eq!(
        reopened
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![hosts[0].0.clone()]
    );
}

#[test]
fn epoch_changes_rebuild_exact_live_sources_and_close_rename_remove_old_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let root_path = root.path().canonicalize().unwrap();
    std::fs::write(root_path.join("tsconfig.json"), "{\"include\":[\"**/*\"]}").unwrap();
    let hosts = ["A.vue", "B.vue", "C.vue"].map(|name| root_path.join(name));
    for path in &hosts {
        std::fs::write(path, VUE).unwrap();
    }
    let mut overlays: FxHashMap<_, _> = hosts.iter().map(|path| (path.clone(), VUE)).collect();
    let session = EditorMirrorSession::new();
    let queries: Vec<_> = hosts
        .iter()
        .map(|path| {
            open(&session, path, VUE, &overlays)
                .mirror
                .as_ref()
                .unwrap()
                .preferred_materialized_path_for_original(path)
                .unwrap()
        })
        .collect();
    let edited = VUE.replace("41", "42");
    overlays.insert(hosts[0].clone(), &edited);
    let refreshed = open(&session, &hosts[1], VUE, &overlays);
    assert_eq!(
        refreshed
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        hosts.to_vec()
    );
    mapped_token(&refreshed, &hosts[0], &edited);
    let source = refreshed
        .materialized_sources()
        .into_iter()
        .find(|source| source.source_path == hosts[0])
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&queries[0]).unwrap(),
        source.code.as_str()
    );
    std::fs::write(&hosts[0], &edited).unwrap();
    let saved = open(&session, &hosts[0], &edited, &overlays);
    mapped_token(&saved, &hosts[0], &edited);
    overlays.remove(&hosts[0]);
    let closed = open(&session, &hosts[1], VUE, &overlays);
    assert_eq!(
        closed
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![hosts[1].clone(), hosts[2].clone()]
    );
    assert!(!queries[0].exists());
    let renamed = root_path.join("D.vue");
    std::fs::rename(&hosts[2], &renamed).unwrap();
    AliasContext::forget_cached_sources(&session, &[hosts[2].clone()]);
    overlays.remove(&hosts[2]);
    overlays.insert(renamed.clone(), VUE);
    let rename_context = open(&session, &renamed, VUE, &overlays);
    assert_eq!(
        rename_context
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![hosts[1].clone(), renamed.clone()]
    );
    assert!(!queries[2].exists());
    mapped_token(&rename_context, &renamed, VUE);
    std::fs::remove_file(&renamed).unwrap();
    overlays.remove(&renamed);
    AliasContext::forget_cached_sources(&session, &[renamed]);
    let deleted = open(&session, &hosts[1], VUE, &overlays);
    assert_eq!(
        deleted
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![hosts[1].clone()]
    );
}
