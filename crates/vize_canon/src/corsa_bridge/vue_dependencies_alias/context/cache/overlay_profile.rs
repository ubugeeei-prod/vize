//! Same-runner observation packet, also compiled unchanged into the parent source.
#![expect(
    clippy::disallowed_types,
    reason = "profile observes shared context ownership"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "profile retains complete mapping debug representations"
)]

use super::super::{AliasContext, PreparedAliasContext};
use crate::corsa_bridge::{
    CorsaMaterializedSource, EditorMirrorSession, vue_document::CorsaProjectEnvironment,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use vize_carton::{FxHashMap, cstr};

fn open(
    session: &EditorMirrorSession,
    path: &Path,
    content: &str,
    overlays: &FxHashMap<PathBuf, &str>,
) -> PreparedAliasContext {
    AliasContext::for_host_cached(
        path,
        content,
        overlays,
        Default::default(),
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

fn source_packet(source: &CorsaMaterializedSource) -> serde_json::Value {
    serde_json::json!({
        "source_path": source.source_path,
        "materialized_path": source.materialized_path,
        "source": source.source,
        "code": source.code,
        "mapping": format!("{:?}", source.mapping),
        "import_source_map": format!("{:?}", source.import_source_map),
        "mapping_kind": format!("{:?}", source.mapping_kind),
    })
}

fn residency() -> serde_json::Value {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |name: &str| {
        status.lines().find_map(|line| {
            line.strip_prefix(name)?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()
        })
    };
    serde_json::json!({"rss_kib":field("VmRSS:"),"high_water_kib":field("VmHWM:")})
}

#[test]
#[ignore = "opt-in paired Actions observation; complete source/map oracles remain assertions"]
fn profile_unrelated_overlays() {
    let mut scenarios = Vec::new();
    for direct in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("tsconfig.json"), r#"{"include":["**/*"]}"#).unwrap();
        for index in 0..300 {
            let source = if index < 299 {
                cstr!(
                    "<script setup lang='ts'>import Next from './N{}.vue'; const n = {index};</script><template><Next />{{{{ n }}}}</template>",
                    index + 1
                )
            } else {
                cstr!(
                    "<script setup lang='ts'>const n = {index};</script><template>{{{{ n }}}}</template>"
                )
            };
            std::fs::write(root.join(cstr!("N{index}.vue")), source.as_bytes()).unwrap();
        }
        let a = root.join("A.vue");
        let b = root.join("B.vue");
        let a_source = "<script setup lang='ts'>import Next from './N0.vue'; const token = 'é';</script><template><Next />{{ token }}</template>";
        let b_source =
            "<script setup lang='ts'>const label = 0;</script><template>{{ label }}</template>";
        std::fs::write(&a, a_source).unwrap();
        std::fs::write(&b, b_source).unwrap();
        let session = EditorMirrorSession::new();
        let mut overlays = FxHashMap::from_iter([(a.clone(), a_source), (b.clone(), b_source)]);
        let cold = Instant::now();
        let original = open(&session, &a, a_source, &overlays);
        let cold_ms = cold.elapsed().as_secs_f64() * 1000.0;
        let mut expected = original
            .materialized_sources()
            .into_iter()
            .map(|source| (source.materialized_path.clone(), source_packet(&source)))
            .collect::<Vec<_>>();
        expected.sort_by(|left, right| left.0.cmp(&right.0));
        let sibling = open(&session, &b, b_source, &overlays);
        let b_path = sibling.mirror_virtual_path(&b).unwrap();
        drop(sibling);
        let mut warm_ms = Vec::new();
        for _ in 0..5 {
            let started = Instant::now();
            let warm = open(&session, &a, a_source, &overlays);
            warm_ms.push(started.elapsed().as_secs_f64() * 1000.0);
            assert!(Arc::ptr_eq(&original.context, &warm.context));
        }
        let mut rows = Vec::new();
        let mut last_native = Vec::new();
        let mut last_sibling = serde_json::Value::Null;
        let edits = (1..=8).map(|cycle| cstr!("<script setup lang='ts'>const label = {cycle};</script><template>{{{{ label }}}}</template>")).collect::<Vec<_>>();
        for (index, changed) in edits.iter().enumerate() {
            let cycle = index + 1;
            overlays.insert(b.clone(), changed.as_str());
            let started = Instant::now();
            if !direct {
                drop(open(&session, &b, changed, &overlays));
            }
            let query = open(&session, &a, a_source, &overlays);
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            for (path, packet) in &expected {
                assert_eq!(
                    source_packet(&query.source_catalog.get(path).unwrap()),
                    *packet
                );
            }
            let current_b = query.source_catalog.get(&b_path).unwrap();
            assert_eq!(current_b.source.as_str(), changed.as_str());
            assert_eq!(
                std::fs::read_to_string(&b_path).unwrap(),
                current_b.code.as_str()
            );
            let mut native = Vec::new();
            query.source_catalog.extend_native_project_documents(
                query.mirror.as_ref().unwrap().virtual_root(),
                &mut native,
            );
            native.sort();
            let mut expected_native = expected
                .iter()
                .filter(|(_, packet)| packet["mapping_kind"] != "Synthetic")
                .map(|(path, packet)| {
                    (
                        crate::file_uri::path_to_file_uri(path),
                        vize_carton::String::from(packet["code"].as_str().unwrap()),
                    )
                })
                .collect::<Vec<_>>();
            expected_native.push((
                crate::file_uri::path_to_file_uri(&b_path),
                current_b.code.clone(),
            ));
            expected_native.sort();
            assert_eq!(native, expected_native);
            rows.push(serde_json::json!({
                "cycle": cycle, "elapsed_ms": elapsed_ms,
                "host_context_reused": Arc::ptr_eq(&original.context, &query.context),
                "host_source_registrations": query.mirror.as_ref().unwrap().source_registrations(),
                "materialized_delta": format!("{:?}", query.materialized_changes),
                "residency": residency(),
            }));
            last_native = native;
            last_sibling = source_packet(&current_b);
            overlays.remove(&b);
        }
        scenarios.push(serde_json::json!({
            "mode":if direct {"query-unchanged-host-first"} else {"query-edited-sibling-first"},
            "files":302,"cold_ms":cold_ms,"warm_ms":warm_ms,"edits":rows,
            "complete_unchanged_sources":expected,"complete_final_native_documents":last_native,"complete_final_sibling":last_sibling,
        }));
    }
    let output = std::env::var_os("VIZE_OVERLAY_PROFILE_OUTPUT").expect("explicit artifact path");
    std::fs::write(output, serde_json::to_vec_pretty(&serde_json::json!({
        "schema":"vize.lsp.overlay-graph-profile.v1", "scenarios":scenarios,
        "scope":"editor preparation only; fresh session cold, warm graph hits, unsaved sibling edits; Rust process RSS including retained old query", 
    })).unwrap()).unwrap();
}
