//! Real editor builds keep all artifacts while sharing syntax facts across waves.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{SourceRevision, build_with_import_facts};
use crate::batch::ModuleSpecifierFactsCache;
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

const HOST: &str = r#"<script setup lang="ts">
import Root from 'root-widget';
import Future from 'future-widget';
import { local } from '@/bridge';
</script><template><Root :root-count="local" /><Future /></template>"#;
const ROOT: &str = r#"<script setup lang="ts">
import Leaf from 'leaf-widget';
defineProps<{ rootCount: number }>();
</script><template><Leaf /></template>"#;
const LEAF: &str = r#"<script setup lang="ts">
import Terminal from 'terminal-widget';
defineProps<{ leafLabel: string }>();
</script><template><Terminal />{{ leafLabel }}</template>"#;
const LOCAL: &str = r#"<script setup lang="ts">defineProps<{ diskCount: number }>()</script><template>{{ diskCount }}</template>"#;
const OVERLAY: &str = r#"<script setup lang="ts">
import OverlayOnly from './OverlayOnly.vue';
defineProps<{ unsavedCount: number }>();
</script><template><OverlayOnly />{{ unsavedCount }}</template>"#;

#[test]
fn syntax_fact_reuse_preserves_editor_fixpoint_and_fresh_revisions() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let session = EditorMirrorSession::new();
    let host = root.join("src/App.vue");
    let local = root.join("src/Local.vue");
    let future = root.join("node_modules/future-widget/Future.vue");
    let leaf = root.join("node_modules/leaf-widget/Leaf.vue");
    let browser = root.join("node_modules/leaf-widget/Browser.vue");
    let replacement = root.join("node_modules/leaf-widget/Replacement.vue");
    write(&host, HOST);
    write_config(&root, "browser");
    write(
        &root.join("src/bridge.ts"),
        "export const local = 1; export { default as Local } from './Local.vue';",
    );
    write(&local, LOCAL);
    write(
        &root.join("src/OverlayOnly.vue"),
        "<template><span>overlay dependency</span></template>",
    );
    package(&root, "root-widget", "./Root.vue");
    write(&root.join("node_modules/root-widget/Root.vue"), ROOT);
    write(
        &root.join("node_modules/leaf-widget/package.json"),
        r#"{"name":"leaf-widget","exports":{".":{"browser":"./Browser.vue","default":"./Leaf.vue"}}}"#,
    );
    write(&leaf, LEAF);
    write(&browser, LEAF);
    package(&root, "terminal-widget", "./Terminal.vue");
    write(
        &root.join("node_modules/terminal-widget/Terminal.vue"),
        "<template><strong>terminal wave</strong></template>",
    );
    // A real manifest with a missing target keeps the failed probe watchable.
    package(&root, "future-widget", "./Future.vue");

    let baseline = compare_builds(&root, &host, &FxHashMap::default(), &session);
    let mirror = baseline.mirror.as_ref().unwrap();
    assert!(mirror.find_by_original(&leaf).is_some());
    assert!(mirror.find_by_original(&browser).is_some());
    assert!(
        mirror
            .find_by_original(&root.join("node_modules/terminal-widget/Terminal.vue"))
            .is_some()
    );
    assert!(mirror.find_by_original(&future).is_none());
    assert!(baseline.route_inputs.contains(&future));
    assert!(
        mirror
            .editor_script_document(&root.join("src/bridge.ts"))
            .unwrap()
            .1
            .code
            .contains("Local.vue.ts")
    );
    let local_map = mirror.editor_vue_document(&local).unwrap().1;
    assert!(!local_map.mapping.spans().is_empty());
    assert!(local_map.code.contains("diskCount"));

    let overlays = FxHashMap::from_iter([(local.clone(), OVERLAY)]);
    let unsaved = compare_builds(&root, &host, &overlays, &session);
    let mirror = unsaved.mirror.as_ref().unwrap();
    assert!(
        mirror
            .find_by_original(&root.join("src/OverlayOnly.vue"))
            .is_some()
    );
    let code = &mirror.find_by_original(&local).unwrap().content;
    assert!(code.contains("unsavedCount"));
    assert!(!code.contains("diskCount"));
    assert_eq!(std::fs::read_to_string(&local).unwrap(), LOCAL);

    // Identical importer bytes must discover new files and changed conditions
    // in a fresh preparation, without keeping old route candidates alive.
    write(
        &future,
        "<template><i>newly available package</i></template>",
    );
    write(&replacement, LEAF);
    package(&root, "leaf-widget", "./Replacement.vue");
    write_config(&root, "development");
    let changed = compare_builds(&root, &host, &FxHashMap::default(), &session);
    let mirror = changed.mirror.as_ref().unwrap();
    assert!(mirror.find_by_original(&future).is_some());
    assert!(mirror.find_by_original(&replacement).is_some());
    assert!(mirror.find_by_original(&leaf).is_none());
    assert!(mirror.find_by_original(&browser).is_none());
    assert!(
        mirror
            .preparation_import_facts_snapshot()
            .bindings
            .iter()
            .all(|binding| binding.context.active_conditions == ["development"])
    );
    let code = &mirror.find_by_original(&local).unwrap().content;
    assert!(code.contains("diskCount"));
    assert!(!code.contains("unsavedCount"));
    assert!(
        mirror
            .find_by_original(&root.join("src/OverlayOnly.vue"))
            .is_none()
    );

    std::fs::remove_file(&local).unwrap();
    let deleted = compare_builds(&root, &host, &FxHashMap::default(), &session);
    let mirror = deleted.mirror.as_ref().unwrap();
    assert!(mirror.find_by_original(&local).is_none());
    assert!(mirror.editor_resolution_inputs().contains(&local));

    let repaired_source = LOCAL.replace("diskCount", "repairedCount");
    write(&local, &repaired_source);
    let repaired = compare_builds(&root, &host, &FxHashMap::default(), &session);
    let code = &repaired
        .mirror
        .as_ref()
        .unwrap()
        .find_by_original(&local)
        .unwrap()
        .content;
    assert!(code.contains("repairedCount"));
    assert!(!code.contains("diskCount"));
}

fn compare_builds(
    root: &Path,
    host: &Path,
    overlays: &FxHashMap<PathBuf, &str>,
    session: &EditorMirrorSession,
) -> AliasContext {
    let mut cached = ModuleSpecifierFactsCache::default();
    let mut uncached = ModuleSpecifierFactsCache::uncached();
    let context = build(root, host, overlays, session, &mut cached);
    let reference = build(root, host, overlays, session, &mut uncached);
    let cached_work = cached.stats();
    let uncached_work = uncached.stats();
    assert!(cached_work.hits > 0, "{cached_work:?}");
    assert_eq!(cached_work.parses, cached_work.retained_entries);
    assert_eq!(uncached_work.hits, 0);
    assert_eq!(uncached_work.retained_entries, 0);
    assert_eq!(
        uncached_work.parses,
        cached_work.parses + cached_work.hits,
        "every syntax request still executes; identical inputs parse once"
    );
    assert_eq!(context.project_root, reference.project_root);
    assert_eq!(context.aliases, reference.aliases);
    assert_eq!(context.route_inputs, reference.route_inputs);
    assert_eq!(context.refreshed_sources, reference.refreshed_sources);
    let routes = |context: &AliasContext| {
        context
            .package_routes
            .iter()
            .map(|(key, route)| (key.clone(), route.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(routes(&context), routes(&reference));
    assert_eq!(
        context
            .mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot(),
        reference
            .mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot()
    );
    context
}

fn build(
    root: &Path,
    host: &Path,
    overlays: &FxHashMap<PathBuf, &str>,
    session: &EditorMirrorSession,
    facts: &mut ModuleSpecifierFactsCache,
) -> AliasContext {
    let source = std::fs::read_to_string(host).unwrap();
    let shared_routes = crate::PackageRouteResolver::default();
    let mut resolver = shared_routes.clone();
    build_with_import_facts(
        host,
        &source,
        overlays,
        SourceRevision {
            requested_sources: &[],
            overlay_identity: 0,
        },
        &mut resolver,
        Default::default(),
        crate::corsa_bridge::vue_document::CorsaProjectEnvironment {
            virtual_ts_options: &Default::default(),
            package_routes: &shared_routes,
            project_root: Some(root),
            tsconfig_path: None,
            editor_session: session,
        },
        facts,
    )
    .expect("bounded package-wave preparation")
}

fn write_config(root: &Path, condition: &str) {
    write(
        &root.join("tsconfig.json"),
        &vize_carton::cstr!(
            r#"{{"compilerOptions":{{"moduleResolution":"Bundler","customConditions":["{condition}"],"paths":{{"@/*":["./src/*"]}}}}}}"#
        ),
    );
}

fn package(root: &Path, name: &str, target: &str) {
    write(
        &root.join("node_modules").join(name).join("package.json"),
        &vize_carton::cstr!(r#"{{"name":"{name}","exports":"{target}"}}"#),
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
