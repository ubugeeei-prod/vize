//! Complete, ordered output of the real editor preparation for cache parity tests.

#![expect(
    clippy::disallowed_types,
    reason = "test snapshots use owned standard strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "test snapshots retain complete debug representations"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::VirtualProject;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PreparationImportFactsSnapshot {
    pub(crate) files: BTreeMap<PathBuf, (PathBuf, String, String)>,
    pub(crate) original_contents: BTreeMap<PathBuf, vize_carton::String>,
    pre_rewrite_code: BTreeMap<PathBuf, vize_carton::String>,
    diagnostics: Vec<String>,
    pub(crate) bindings: Vec<crate::PackageRouteBinding>,
    pub(crate) edges: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    inbound: BTreeMap<PathBuf, usize>,
    reverse_routes: BTreeMap<PathBuf, BTreeSet<crate::package_route::PackageRouteKey>>,
    artifacts: BTreeMap<PathBuf, (Vec<PathBuf>, Vec<PathBuf>, bool, String)>,
    shadows: BTreeMap<PathBuf, (PathBuf, String)>,
    manifests: BTreeMap<PathBuf, (PathBuf, String)>,
    passthrough: BTreeMap<PathBuf, (PathBuf, Vec<u8>)>,
    expected_files: BTreeSet<PathBuf>,
    resolution_inputs: Vec<PathBuf>,
    governing_configs: Vec<PathBuf>,
    compiler_options: serde_json::Value,
}

impl VirtualProject {
    pub(crate) fn preparation_import_facts_snapshot(&self) -> PreparationImportFactsSnapshot {
        let mut diagnostics = self
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{diagnostic:?}"))
            .collect::<Vec<_>>();
        diagnostics.sort();
        PreparationImportFactsSnapshot {
            files: self
                .virtual_files
                .iter()
                .map(|(path, file)| {
                    (
                        path.clone(),
                        (
                            file.original_path.clone(),
                            file.content.as_str().to_owned(),
                            format!("{:?}", file.source_map),
                        ),
                    )
                })
                .collect(),
            original_contents: self
                .original_contents
                .iter()
                .map(|(path, content)| (path.clone(), content.clone()))
                .collect(),
            pre_rewrite_code: self
                .pre_rewrite_code
                .iter()
                .map(|(path, content)| (path.clone(), content.clone()))
                .collect(),
            diagnostics,
            bindings: self.package_routes_snapshot(),
            edges: self
                .dependency_edges
                .iter()
                .map(|(source, targets)| (source.clone(), targets.iter().cloned().collect()))
                .collect(),
            inbound: self
                .dependency_inbound
                .iter()
                .map(|(path, count)| (path.clone(), *count))
                .collect(),
            reverse_routes: self
                .package_route_reverse
                .iter()
                .map(|(path, keys)| (path.clone(), keys.iter().cloned().collect()))
                .collect(),
            artifacts: self
                .source_artifacts
                .iter()
                .map(|(path, artifacts)| {
                    (
                        path.clone(),
                        (
                            artifacts.virtual_paths.clone(),
                            artifacts.passthrough_paths.clone(),
                            artifacts.empty_module_exports,
                            format!("{:?}", artifacts.module_exports),
                        ),
                    )
                })
                .collect(),
            shadows: self
                .package_shadow_files
                .iter()
                .map(|(shadow, source)| {
                    (
                        shadow.clone(),
                        (
                            source.clone(),
                            self.package_shadow_content(shadow, source)
                                .expect("prepared shadow content")
                                .as_str()
                                .to_owned(),
                        ),
                    )
                })
                .collect(),
            manifests: self
                .package_shadow_manifests
                .iter()
                .map(|(shadow, source)| {
                    (
                        shadow.clone(),
                        (source.clone(), std::fs::read_to_string(source).unwrap()),
                    )
                })
                .collect(),
            passthrough: self
                .passthrough_files
                .iter()
                .map(|(virtual_path, source)| {
                    (
                        virtual_path.clone(),
                        (source.clone(), std::fs::read(source).unwrap()),
                    )
                })
                .collect(),
            expected_files: self.expected_materialized_files().into_iter().collect(),
            resolution_inputs: self.editor_resolution_inputs(),
            governing_configs: self.governing_config_paths(),
            compiler_options: serde_json::Value::Object(
                self.load_compiler_options(self.resolved_tsconfig_path().as_deref())
                    .expect("fixture compiler options"),
            ),
        }
    }
}

#[test]
fn warm_syntax_facts_do_not_freeze_missing_targets_routes_or_dependency_ownership() {
    use crate::batch::ModuleSpecifierFactsCache;

    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("src/host.ts");
    let late = root.join("src/late.ts");
    let widget = root.join("node_modules/future-widget/Widget.vue");
    let nested = root.join("node_modules/future-widget/nested.ts");
    let write = |path: &std::path::Path, source: &str| {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    };
    let source = "import { value } from './late'; import Widget from 'future-widget'; export { value, Widget };";
    write(&host, source);
    write(&root.join("tsconfig.json"), "{}");
    write(
        &root.join("node_modules/future-widget/package.json"),
        r#"{"name":"future-widget","exports":"./Widget.vue"}"#,
    );
    let mut cached = VirtualProject::new(&root).unwrap();
    let mut uncached = VirtualProject::new(&root).unwrap();
    for project in [&mut cached, &mut uncached] {
        project.set_session_script_registration(true);
        project.set_declaration_roots(std::slice::from_ref(&host));
        project.register_path_with_content(&host, source).unwrap();
    }
    let mut facts = ModuleSpecifierFactsCache::default();
    let mut reference_facts = ModuleSpecifierFactsCache::uncached();
    let mut resolver = crate::PackageRouteResolver::default();
    let mut reference_resolver = crate::PackageRouteResolver::default();
    let mut run = |cached: &mut VirtualProject, uncached: &mut VirtualProject| {
        let observed = walk_with_resolver(cached, &mut resolver, &mut facts);
        let reference = walk_with_resolver(uncached, &mut reference_resolver, &mut reference_facts);
        assert_eq!(observed, reference);
        assert_eq!(
            cached.preparation_import_facts_snapshot(),
            uncached.preparation_import_facts_snapshot()
        );
        observed
    };

    let before = run(&mut cached, &mut uncached);
    assert!(before.contains(&(vize_carton::String::from("future-widget"), false)));
    assert!(cached.find_by_original(&late).is_none());
    assert!(cached.find_by_original(&widget).is_none());

    write(&late, "export const value = 1;");
    write(
        &widget,
        r#"<script setup lang="ts">import { nested } from './nested';</script><template>{{ nested }}</template>"#,
    );
    let appeared = run(&mut cached, &mut uncached);
    assert!(appeared.contains(&(vize_carton::String::from("future-widget"), true)));
    assert!(cached.find_by_original(&late).is_some());
    assert!(cached.find_by_original(&widget).is_some());
    assert!(cached.dependency_edges[&host].contains(&late));

    // Warm the package importer's exact bytes while its relative target is
    // absent, then make that same occurrence resolvable without editing it.
    run(&mut cached, &mut uncached);
    assert!(cached.find_by_original(&nested).is_none());
    write(&nested, "export const nested = 2;");
    run(&mut cached, &mut uncached);
    assert!(cached.find_by_original(&nested).is_some());
    assert!(cached.dependency_edges[&widget].contains(&nested));

    std::fs::remove_file(&late).unwrap();
    run(&mut cached, &mut uncached);
    assert!(cached.find_by_original(&late).is_none());
    assert!(!cached.dependency_edges[&host].contains(&late));
    assert!(!cached.dependency_inbound.contains_key(&late));
    assert!(cached.editor_resolution_inputs().contains(&late));

    write(&late, "export const value = 3;");
    run(&mut cached, &mut uncached);
    assert!(cached.dependency_edges[&host].contains(&late));
    assert!(
        cached
            .find_by_original(&late)
            .unwrap()
            .content
            .contains("3")
    );
    assert_eq!(
        cached.original_contents[&cached.original_index[&host]],
        source
    );
    let work = facts.stats();
    let reference_work = reference_facts.stats();
    assert!(work.hits > 0, "{work:?}");
    assert_eq!(work.parses, work.retained_entries);
    assert_eq!(reference_work.parses, work.parses + work.hits);
    assert_eq!(reference_work.retained_entries, 0);
}

fn walk_with_resolver(
    project: &mut VirtualProject,
    resolver: &mut crate::PackageRouteResolver,
    facts: &mut crate::batch::ModuleSpecifierFactsCache,
) -> Vec<(vize_carton::String, bool)> {
    let settings = project.package_resolution_settings();
    let mut observed = Vec::new();
    let mut bindings = Vec::new();
    let mut resolve =
        |importer: &std::path::Path, specifier: &str, mode: crate::PackageResolutionMode| {
            let (context, context_inputs) = settings.context(resolver, importer, mode);
            let lookup = resolver.lookup_with_context(
                importer.parent().unwrap(),
                specifier,
                crate::PackageSourceOptions::new(true, true),
                context.clone(),
            );
            let watchable_negative = lookup.is_watchable_negative();
            let (route, inputs) = lookup.into_parts();
            let found = route.is_some();
            observed.push((specifier.into(), found));
            if found || watchable_negative {
                bindings.push(crate::PackageRouteBinding {
                    importer_path: importer.to_path_buf(),
                    specifier: specifier.into(),
                    occurrence_mode: mode,
                    context,
                    route,
                    invalidation_paths: inputs
                        .into_iter()
                        .chain(settings.input_paths().iter().cloned())
                        .chain(context_inputs)
                        .chain(std::iter::once(importer.to_path_buf()))
                        .collect(),
                });
            }
            found
        };
    project
        .register_reachable_dependencies_with_import_facts(
            &Default::default(),
            &["future-widget".into()],
            &mut resolve,
            facts,
        )
        .unwrap();
    project.set_package_routes(bindings);
    project.register_package_route_targets().unwrap();
    project.finalize_package_routes().unwrap();
    observed
}
