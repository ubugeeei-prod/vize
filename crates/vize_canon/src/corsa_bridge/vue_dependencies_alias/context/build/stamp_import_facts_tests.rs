//! Build-local syntax facts never replace authoritative filesystem stamps.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::{SourceRevision, build_with_import_facts};
use crate::batch::ModuleSpecifierFactsCache;
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_dependencies_alias::context::{AliasContext, ContextFingerprint};
use crate::corsa_bridge::vue_document::CorsaProjectEnvironment;
use crate::package_route::stamp::InputStamp;

const HOST: &str = r#"<script setup lang="ts">
import Child from './Child.vue';
import { base } from '@/base';
import { late } from './late';
import { priority } from './priority.js';
import { linked } from './link';
import Package from 'tiny-widget';
</script><template><Child :value="base + late + priority + linked" /><Package /></template>"#;
const CHILD: &str = r#"<script setup lang="ts">defineProps<{ value: number }>()</script><template>{{ value }}</template>"#;
const WORKSPACE: &str = r#"<script setup lang="ts">import { workspace } from './workspace';</script><template>{{ workspace }}</template>"#;

#[test]
fn immediate_stamp_reuses_every_finalized_parse_and_keeps_exact_input_stamps() {
    let fixture = Fixture::new();
    for workspace in [false, true] {
        let requested = fixture.requested(workspace);
        let overlays = FxHashMap::default();
        let mut facts = ModuleSpecifierFactsCache::default();
        let mut uncached = ModuleSpecifierFactsCache::uncached();
        let context = fixture.build(&overlays, &requested, &mut facts);
        let reference = fixture.build(&overlays, &requested, &mut uncached);
        assert_context_equal(&context, &reference);
        assert!(!context.package_routes.is_empty());

        let before = facts.stats();
        let reference_before = uncached.stats();
        let mut fingerprint = fixture.fingerprint(&overlays, &requested);
        fingerprint.stamp_with_import_facts(&context, &mut facts);
        let mut reference_fingerprint = fixture.fingerprint(&overlays, &requested);
        reference_fingerprint.stamp_with_import_facts(&reference, &mut uncached);
        let mut legacy = fixture.fingerprint(&overlays, &requested);
        legacy.stamp(&reference);
        assert_eq!(
            fingerprint.input_stamps(),
            reference_fingerprint.input_stamps()
        );
        assert_eq!(fingerprint.input_stamps(), legacy.input_stamps());
        let after = facts.stats();
        let reference_after = uncached.stats();
        assert_eq!(
            after.parses, before.parses,
            "finalization already parsed every stamp input"
        );
        assert!(after.hits > before.hits);
        assert_eq!(
            reference_after.parses - reference_before.parses,
            after.hits - before.hits,
            "uncached stamping parses every input; cached stamping reuses each input"
        );
        assert_eq!(reference_after.retained_entries, 0);
        assert!(
            fingerprint
                .input_stamps()
                .iter()
                .all(InputStamp::is_current)
        );
        assert_eq!(
            context
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(&fixture.workspace)
                .is_some(),
            workspace
        );
    }
}

fn stamp_equal(
    fixture: &Fixture,
    context: &AliasContext,
    overlays: &FxHashMap<PathBuf, &str>,
    requested: &[(PathBuf, &str)],
    facts: &mut ModuleSpecifierFactsCache,
) -> Vec<InputStamp> {
    let before = facts.stats();
    let mut cached = fixture.fingerprint(overlays, requested);
    cached.stamp_with_import_facts(context, facts);
    let mut legacy = fixture.fingerprint(overlays, requested);
    legacy.stamp(context);
    let mut uncached = fixture.fingerprint(overlays, requested);
    let mut uncached_facts = ModuleSpecifierFactsCache::uncached();
    uncached.stamp_with_import_facts(context, &mut uncached_facts);
    assert_eq!(cached.input_stamps(), legacy.input_stamps());
    assert_eq!(cached.input_stamps(), uncached.input_stamps());
    let after = facts.stats();
    assert_eq!(after.parses, before.parses);
    assert_eq!(after.hits - before.hits, uncached_facts.stats().parses);
    assert!(uncached_facts.stats().parses > 0);
    cached.input_stamps()
}

fn assert_context_equal(actual: &AliasContext, expected: &AliasContext) {
    assert_eq!(actual.project_root, expected.project_root);
    assert_eq!(actual.aliases, expected.aliases);
    assert_eq!(actual.package_routes, expected.package_routes);
    assert_eq!(actual.route_inputs, expected.route_inputs);
    assert_eq!(actual.refreshed_sources, expected.refreshed_sources);
    assert_eq!(
        actual
            .mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot(),
        expected
            .mirror
            .as_ref()
            .unwrap()
            .preparation_import_facts_snapshot()
    );
}

fn assert_fresh_revision(
    fixture: &Fixture,
    overlays: &FxHashMap<PathBuf, &str>,
    requested: &[(PathBuf, &str)],
) -> AliasContext {
    let mut facts = ModuleSpecifierFactsCache::default();
    let actual = fixture.build(overlays, requested, &mut facts);
    let mut uncached = ModuleSpecifierFactsCache::uncached();
    let reference = fixture.build(overlays, requested, &mut uncached);
    assert_context_equal(&actual, &reference);
    let stamps = stamp_equal(fixture, &actual, overlays, requested, &mut facts);
    let mut reference_fingerprint = fixture.fingerprint(overlays, requested);
    reference_fingerprint.stamp_with_import_facts(&reference, &mut uncached);
    assert_eq!(stamps, reference_fingerprint.input_stamps());
    actual
}

struct Fixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    host: PathBuf,
    workspace: PathBuf,
    session: EditorMirrorSession,
    routes: crate::PackageRouteResolver,
    options: crate::virtual_ts::VirtualTsOptions,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let host = root.join("src/App.vue");
        let workspace = root.join("src/Workspace.vue");
        write(&host, HOST);
        write(&workspace, WORKSPACE);
        write(
            &root.join("src/workspace.ts"),
            "export const workspace = 1;",
        );
        write(&root.join("src/Child.vue"), CHILD);
        write(&root.join("src/base.ts"), "export const base = 1;");
        write(&root.join("src/priority.js"), "export const priority = 1;");
        write(&root.join("src/linked-a.ts"), "export const linked = 1;");
        write(&root.join("src/linked-b.ts"), "export const linked = 2;");
        write(
            &root.join("node_modules/tiny-widget/package.json"),
            r#"{"name":"tiny-widget","exports":"./Widget.vue"}"#,
        );
        write(
            &root.join("node_modules/tiny-widget/Widget.vue"),
            "<template><span>package target</span></template>",
        );
        #[cfg(unix)]
        std::os::unix::fs::symlink("linked-a.ts", root.join("src/link.ts")).unwrap();
        #[cfg(not(unix))]
        write(&root.join("src/link.ts"), "export const linked = 1;");
        write(
            &root.join("tsconfig.json"),
            r#"{"compilerOptions":{"moduleResolution":"Bundler","customConditions":["browser"],"paths":{"@/*":["./src/*"]}}}"#,
        );
        Self {
            _directory: directory,
            root,
            host,
            workspace,
            session: EditorMirrorSession::new(),
            routes: crate::PackageRouteResolver::default(),
            options: Default::default(),
        }
    }

    fn requested(&self, workspace: bool) -> Vec<(PathBuf, &'static str)> {
        if workspace {
            vec![(self.workspace.clone(), WORKSPACE)]
        } else {
            Vec::new()
        }
    }

    fn environment(&self) -> CorsaProjectEnvironment<'_> {
        CorsaProjectEnvironment {
            virtual_ts_options: &self.options,
            package_routes: &self.routes,
            project_root: Some(&self.root),
            tsconfig_path: None,
            editor_session: &self.session,
        }
    }

    fn fingerprint(
        &self,
        overlays: &FxHashMap<PathBuf, &str>,
        requested: &[(PathBuf, &str)],
    ) -> ContextFingerprint {
        AliasContext::request_fingerprint(
            &self.host,
            HOST,
            overlays,
            requested,
            Default::default(),
            self.environment(),
        )
    }

    fn build(
        &self,
        overlays: &FxHashMap<PathBuf, &str>,
        requested: &[(PathBuf, &str)],
        facts: &mut ModuleSpecifierFactsCache,
    ) -> AliasContext {
        let mut resolver = self.routes.clone();
        build_with_import_facts(
            &self.host,
            HOST,
            overlays,
            SourceRevision {
                requested_sources: requested,
                overlay_identity: self.fingerprint(overlays, requested).overlay_identity(),
            },
            &mut resolver,
            Default::default(),
            self.environment(),
            facts,
        )
        .expect("bounded build and stamp fixture")
    }
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[path = "stamp_import_facts_lifetime_tests.rs"]
mod lifetime_tests;

mod mutation_tests {
    //! Filesystem and overlay mutations remain authoritative with warm syntax facts.

    use super::*;

    #[test]
    fn warm_stamp_facts_observe_missing_priority_content_config_and_overlay_changes() {
        let fixture = Fixture::new();
        let overlays = FxHashMap::default();
        let requested = fixture.requested(false);
        let mut facts = ModuleSpecifierFactsCache::default();
        let context = fixture.build(&overlays, &requested, &mut facts);
        let mut previous = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);

        let late = fixture.root.join("src/late.ts");
        assert!(!late.exists());
        write(&late, "export const late = 1;");
        assert!(previous.iter().any(|stamp| !stamp.is_current()));
        let appeared = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
        assert_ne!(previous, appeared);
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        assert!(
            fresh
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(&late)
                .is_some()
        );
        previous = appeared;
        std::fs::remove_file(&late).unwrap();
        assert!(previous.iter().any(|stamp| !stamp.is_current()));
        let deleted = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
        assert_ne!(previous, deleted);
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        assert!(
            fresh
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(&late)
                .is_none()
        );
        previous = deleted;
        write(&late, "export const late = 2;");
        assert!(previous.iter().any(|stamp| !stamp.is_current()));
        previous = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        assert!(
            fresh
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(&late)
                .unwrap()
                .content
                .contains("late = 2")
        );

        // A `.ts` appearing beside an existing `.js` wins TypeScript extension
        // substitution. Reusing syntax must still rerun every failed probe.
        let priority = fixture.root.join("src/priority.ts");
        assert!(!priority.exists());
        write(&priority, "export const priority = 2;");
        assert!(previous.iter().any(|stamp| !stamp.is_current()));
        let changed = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
        assert_ne!(previous, changed);
        assert_eq!(
            crate::batch::virtual_project::dependency_scan::resolve_dependency(
                "./priority.js",
                &fixture.root.join("src"),
                &fixture.root,
                &context.aliases,
            ),
            Some(priority.clone())
        );
        previous = changed;
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        let mirror = fresh.mirror.as_ref().unwrap();
        assert!(mirror.find_by_original(&priority).is_some());
        assert!(
            mirror
                .find_by_original(&fixture.root.join("src/priority.js"))
                .is_none()
        );

        let base = fixture.root.join("src/base.ts");
        let modified = std::fs::metadata(&base).unwrap().modified().unwrap();
        let old_content = std::fs::read_to_string(&base).unwrap();
        let edited = old_content.replace("1", "2");
        assert_eq!(old_content.len(), edited.len());
        write(&base, &edited);
        std::fs::File::options()
            .write(true)
            .open(&base)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert!(previous.iter().any(|stamp| !stamp.is_current()));
        let changed = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
        assert_ne!(previous, changed);
        previous = changed;
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        assert!(
            fresh
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(&base)
                .unwrap()
                .content
                .contains("base = 2")
        );

        write(
            &fixture.root.join("tsconfig.json"),
            r#"{"compilerOptions":{"moduleResolution":"Bundler","customConditions":["development"],"paths":{"@/*":["./src/*"]}}}"#,
        );
        assert!(previous.iter().any(|stamp| !stamp.is_current()));
        assert_ne!(
            previous,
            stamp_equal(&fixture, &context, &overlays, &requested, &mut facts)
        );
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        let bindings = fresh.mirror.as_ref().unwrap().package_routes_snapshot();
        assert!(!bindings.is_empty());
        assert!(
            bindings
                .iter()
                .all(|binding| binding.context.active_conditions == ["development"])
        );

        let child = fixture.root.join("src/Child.vue");
        let overlay_source = r#"<script setup lang="ts">import Only from './OverlayOnly.vue'; defineProps<{ value: number }>()</script><template><Only />{{ value }}</template>"#;
        let overlay = FxHashMap::from_iter([(child.clone(), overlay_source)]);
        write(&fixture.root.join("src/OverlayOnly.vue"), "<template />");
        let mut overlay_facts = ModuleSpecifierFactsCache::default();
        let overlay_context = fixture.build(&overlay, &requested, &mut overlay_facts);
        let mut uncached = ModuleSpecifierFactsCache::uncached();
        let reference = fixture.build(&overlay, &requested, &mut uncached);
        assert_context_equal(&overlay_context, &reference);
        stamp_equal(
            &fixture,
            &overlay_context,
            &overlay,
            &requested,
            &mut overlay_facts,
        );
        assert_ne!(
            fixture.fingerprint(&overlays, &requested).identity(),
            fixture.fingerprint(&overlay, &requested).identity()
        );
        assert_eq!(std::fs::read_to_string(&child).unwrap(), CHILD);
        assert!(
            overlay_context
                .mirror
                .as_ref()
                .unwrap()
                .find_by_original(&fixture.root.join("src/OverlayOnly.vue"))
                .is_some()
        );
    }

    #[cfg(unix)]
    #[test]
    fn warm_stamp_facts_reobserve_a_retargeted_symlink() {
        let fixture = Fixture::new();
        let overlays = FxHashMap::default();
        let requested = fixture.requested(false);
        let mut facts = ModuleSpecifierFactsCache::default();
        let context = fixture.build(&overlays, &requested, &mut facts);
        let before = stamp_equal(&fixture, &context, &overlays, &requested, &mut facts);
        let link = fixture.root.join("src/link.ts");
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink("linked-b.ts", &link).unwrap();
        assert!(before.iter().any(|stamp| !stamp.is_current()));
        assert_ne!(
            before,
            stamp_equal(&fixture, &context, &overlays, &requested, &mut facts)
        );
        let fresh = assert_fresh_revision(&fixture, &overlays, &requested);
        let mirror = fresh.mirror.as_ref().unwrap();
        assert!(
            mirror
                .find_by_original(&fixture.root.join("src/linked-b.ts"))
                .is_some()
        );
        assert!(
            mirror
                .find_by_original(&fixture.root.join("src/linked-a.ts"))
                .is_none()
        );
    }
}
