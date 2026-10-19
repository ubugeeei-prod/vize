//! Package-route recording and bounded dependency-specifier discovery.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String as CompactString};

use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::{
    PackageRouteReachability, package_resolution::PackageResolutionSettings,
};

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")] // Compiler-option aliases originate in serde_json maps.
type CompilerAlias = (std::string::String, std::string::String);

type ReachabilityCacheKey = (
    PathBuf,
    PathBuf,
    CompactString,
    crate::PackageResolutionContext,
    u8,
);

// Compiler-option aliases originate in serde_json maps.
pub(super) struct RouteDiscovery<'a> {
    settings: &'a PackageResolutionSettings,
    resolver: &'a mut crate::PackageRouteResolver,
    routes: &'a mut FxHashMap<(PathBuf, CompactString), crate::PackageRoute>,
    reachability: &'a mut FxHashMap<ReachabilityCacheKey, PackageRouteReachability>,
    bindings: &'a mut Vec<crate::PackageRouteBinding>,
    inputs: &'a mut Vec<PathBuf>,
    aliases: &'a [CompilerAlias],
    /// Aliases and compiler options a session-cached scan was resolved with.
    scope: u64,
}

impl<'a> RouteDiscovery<'a> {
    pub(super) fn new(
        settings: &'a PackageResolutionSettings,
        resolver: &'a mut crate::PackageRouteResolver,
        routes: &'a mut FxHashMap<(PathBuf, CompactString), crate::PackageRoute>,
        reachability: &'a mut FxHashMap<ReachabilityCacheKey, PackageRouteReachability>,
        bindings: &'a mut Vec<crate::PackageRouteBinding>,
        inputs: &'a mut Vec<PathBuf>,
        aliases: &'a [CompilerAlias],
    ) -> Self {
        let scope = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::hash::DefaultHasher::new();
            aliases.hash(&mut hasher);
            settings.hash_identity(&mut hasher);
            hasher.finish()
        };
        Self {
            settings,
            resolver,
            routes,
            reachability,
            bindings,
            inputs,
            aliases,
            scope,
        }
    }

    pub(super) fn resolve(
        &mut self,
        importer: &Path,
        specifier: &str,
        mode: crate::PackageResolutionMode,
    ) -> bool {
        if crate::batch::virtual_project::is_vue_runtime_support_specifier(specifier) {
            return false;
        }
        let importer_dir = importer.parent().unwrap_or(importer);
        let (context, context_inputs) = self.settings.context(self.resolver, importer, mode);
        let lookup = self.resolver.lookup_with_context(
            importer_dir,
            specifier,
            crate::PackageSourceOptions::new(true, true),
            context.clone(),
        );
        let watchable_negative = lookup.is_watchable_negative();
        let (route, consulted) = lookup.into_parts();
        if route.is_some() || watchable_negative {
            self.inputs.extend(consulted.iter().cloned());
            self.inputs
                .extend(self.settings.input_paths().iter().cloned());
            self.inputs.extend(context_inputs.iter().cloned());
        }
        let reachability = route
            .as_ref()
            .map_or_else(PackageRouteReachability::default, |route| {
                let key = (
                    logical_absolute(importer),
                    route.manifest_path.clone(),
                    CompactString::from(specifier),
                    context.clone(),
                    crate::batch::PACKAGE_REACHABILITY_BUDGET_REVISION,
                );
                let scope = self.scope;
                self.reachability
                    .entry(key)
                    .or_insert_with_key(|(importer, manifest, specifier, context, budget)| {
                        // Edits rebuild the alias context; the session resolver
                        // keeps scans whose consulted inputs are unchanged.
                        let shared = (
                            scope,
                            importer.clone(),
                            manifest.clone(),
                            specifier.clone(),
                            context.clone(),
                            *budget,
                        );
                        if let Some(cached) = self.resolver.cached_reachability(&shared) {
                            return cached;
                        }
                        let scanned = crate::batch::virtual_project::package_route_reaches_vue(
                            route,
                            self.aliases,
                            self.settings,
                            self.resolver,
                            crate::PackageSourceOptions::new(true, true),
                        );
                        scanned.record_work(self.resolver);
                        self.resolver.store_reachability(shared, &scanned);
                        scanned
                    })
                    .clone()
            });
        // Private imports need their authored manifest even for plain TS
        // targets: the mirror's synthetic package boundary has no imports map.
        let needs_shadow = (specifier.starts_with('#') && route.is_some())
            || reachability.requires_shadow()
            || route
                .as_ref()
                .is_some_and(crate::PackageRoute::requires_workspace_source_shadow);
        let track_reachability = reachability.requires_tracking();
        self.inputs.extend(reachability.inputs.iter().cloned());
        if needs_shadow && let Some(route) = route.as_ref() {
            self.routes.insert(
                (logical_absolute(importer_dir), specifier.into()),
                route.clone(),
            );
        }
        if needs_shadow || track_reachability || watchable_negative {
            self.bindings.push(crate::PackageRouteBinding {
                importer_path: importer.to_path_buf(),
                specifier: specifier.into(),
                occurrence_mode: mode,
                context,
                route: needs_shadow.then_some(route).flatten(),
                invalidation_paths: consulted
                    .into_iter()
                    .chain(self.settings.input_paths().iter().cloned())
                    .chain(context_inputs)
                    .chain(std::iter::once(importer.to_path_buf()))
                    .collect(),
                shared_invalidation_paths: Vec::new(),
            });
        }
        needs_shadow
    }
}

pub(super) fn package_specifiers_from_frontier(
    project: &VirtualProject,
    scanned: &mut vize_carton::FxHashSet<PathBuf>,
    facts: &mut crate::batch::ModuleSpecifierFactsCache,
) -> Vec<CompactString> {
    let mut specifiers = project
        .virtual_files_sorted()
        .into_iter()
        .filter(|file| scanned.insert(file.original_path.clone()))
        .flat_map(|file| {
            let source_type = if file
                .virtual_path
                .extension()
                .is_some_and(|extension| extension == "tsx")
            {
                oxc_span::SourceType::tsx()
            } else {
                oxc_span::SourceType::ts()
            };
            facts.collect_all_specifiers(&file.content, source_type)
        })
        .filter(|specifier| {
            !specifier.starts_with('.') && !Path::new(specifier.as_str()).is_absolute()
        })
        .collect::<Vec<_>>();
    specifiers.sort();
    specifiers.dedup();
    specifiers
}

pub(super) fn logical_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(test)]
#[path = "routes_budget_tests.rs"]
mod budget_tests;

#[cfg(test)]
mod private_import_tests;

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    reason = "insta snapshots expand through std::format! (CONTRIBUTING: snapshot assertions)"
)]
mod tests {
    //! Native package route and declaration ownership regressions.

    use super::RouteDiscovery;
    use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
    use vize_carton::FxHashMap;

    const HOST_SOURCE: &str = "<template><div /></template>\n";
    const CLASS_HOST_SOURCE: &str = r#"<script lang="ts">
import { Vue } from "vue-property-decorator";
export const DecoratorBase = Vue;
</script>
"#;

    #[test]
    fn generated_runtime_support_never_creates_editor_route_state() {
        let fixture = runtime_package_fixture();
        let settings =
            crate::batch::virtual_project::package_resolution::PackageResolutionSettings::default();
        let mut resolver = crate::PackageRouteResolver::default();
        let mut routes = FxHashMap::default();
        let mut reachability = FxHashMap::default();
        let mut bindings = Vec::new();
        let mut inputs = Vec::new();
        let aliases = Vec::new();
        let mut discovery = RouteDiscovery::new(
            &settings,
            &mut resolver,
            &mut routes,
            &mut reachability,
            &mut bindings,
            &mut inputs,
            &aliases,
        );

        for specifier in ["vue", "@vue/runtime-dom", "vite/client"] {
            assert!(!discovery.resolve(
                &fixture.host,
                specifier,
                crate::PackageResolutionMode::Import,
            ));
        }
        assert!(routes.is_empty());
        assert!(reachability.is_empty());
        assert!(bindings.is_empty());
        assert!(inputs.is_empty());
    }

    #[test]
    fn generated_vue_helpers_do_not_expand_the_editor_project() {
        let fixture = runtime_package_fixture();
        let context = AliasContext::for_host(&fixture.host, HOST_SOURCE, &FxHashMap::default());

        assert!(context.aliases.is_empty());
        assert!(context.package_routes.is_empty());
        assert!(context.route_inputs.is_empty());
        assert_eq!(
            context
                .mirror
                .as_ref()
                .unwrap()
                .registered_original_paths_sorted(),
            vec![vize_carton::path::canonicalize_non_verbatim(&fixture.host)]
        );
    }

    #[test]
    fn runtime_support_dependency_chain_stays_on_the_native_package_path() {
        let fixture = runtime_package_fixture();
        let context =
            AliasContext::for_host(&fixture.host, CLASS_HOST_SOURCE, &FxHashMap::default());

        assert!(context.aliases.is_empty());
        assert!(context.package_routes.is_empty());
        assert_eq!(
            context
                .mirror
                .as_ref()
                .unwrap()
                .registered_original_paths_sorted(),
            vec![vize_carton::path::canonicalize_non_verbatim(&fixture.host)]
        );
        // Every probe inside the fixture project, relative to it: both decorator
        // packages are routed and the runtime-only Vue build never is.
        let project = vize_carton::path::canonicalize_non_verbatim(
            fixture.host.parent().unwrap().parent().unwrap(),
        );
        let routed = context
            .route_inputs
            .iter()
            .filter_map(|path| path.strip_prefix(&project).ok())
            .map(|path| path.to_str().unwrap())
            .collect::<Vec<_>>();
        insta::assert_snapshot!("class_component_route_inputs", routed.join("\n"));
    }

    #[test]
    fn editor_paths_declaration_is_inferred_inside_the_mirror() {
        let root = tempfile::tempdir().unwrap();
        let host = root.path().join("src/App.vue");
        let declaration = root.path().join("src/api/remote-search.d.ts");
        write(
            &root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
        );
        write(
            &host,
            "<script>import { transactionList } from '@/api/remote-search';\nexport default { methods: { load() { return transactionList() } } }\n</script>\n",
        );
        write(
            &declaration,
            "export declare function transactionList(): Promise<unknown>;\n",
        );

        let source = std::fs::read_to_string(&host).unwrap();
        let context = AliasContext::for_host(&host, &source, &FxHashMap::default());
        let mirror = context.mirror.as_ref().expect("paths require a mirror");
        let materialized = mirror
            .preferred_materialized_path_for_original(&declaration)
            .expect("reachable declaration must be mirrored");

        assert_eq!(materialized.file_name().unwrap(), "remote-search.d.ts");
        let project = materialized.ancestors().nth(3).unwrap();
        let files = mirror.expected_materialized_files();
        let mut mirrored = files
            .iter()
            .map(|path| path.strip_prefix(project).unwrap().to_str().unwrap())
            .collect::<Vec<_>>();
        mirrored.sort_unstable();
        assert_eq!(
            mirrored,
            [
                "__vize_vue_modules.d.ts",
                "package.json",
                "src/App.vue.ts",
                "src/api/remote-search.d.ts",
                "tsconfig.json",
            ]
        );
        assert!(!mirror.is_declaration_root(&declaration));
    }

    /// Every editor edit rebuilds its alias context with fresh local maps. The
    /// session resolver must keep a route scan while its consulted inputs hold,
    /// and scan again once one of them changes.
    #[test]
    fn rebuilt_discovery_reuses_session_reachability_until_an_input_changes() {
        let fixture = runtime_package_fixture();
        let settings =
            crate::batch::virtual_project::package_resolution::PackageResolutionSettings::default();
        let mut resolver = crate::PackageRouteResolver::default();
        let aliases = Vec::new();
        let mut discover = |resolver: &mut crate::PackageRouteResolver| {
            let (mut routes, mut reachability) = (FxHashMap::default(), FxHashMap::default());
            let (mut bindings, mut inputs) = (Vec::new(), Vec::new());
            let mut discovery = RouteDiscovery::new(
                &settings,
                resolver,
                &mut routes,
                &mut reachability,
                &mut bindings,
                &mut inputs,
                &aliases,
            );
            discovery.resolve(
                &fixture.host,
                "vue-property-decorator",
                crate::PackageResolutionMode::Import,
            );
            resolver.metrics().reachability_checks
        };

        assert_eq!(discover(&mut resolver), 1);
        assert_eq!(
            discover(&mut resolver),
            1,
            "an unchanged rebuild must not rescan"
        );
        let package = fixture.host.parent().unwrap().parent().unwrap();
        write(
            &package.join("node_modules/vue-property-decorator/index.d.ts"),
            "export { Vue } from 'vue-class-component';\nexport declare const changed: true;\n",
        );
        assert_eq!(discover(&mut resolver), 2, "a changed input must rescan");
    }

    struct RuntimePackageFixture {
        _root: tempfile::TempDir,
        host: std::path::PathBuf,
    }

    fn runtime_package_fixture() -> RuntimePackageFixture {
        let root = tempfile::tempdir().unwrap();
        let host = root.path().join("src/App.vue");
        let vue = root.path().join("node_modules/vue");
        write(&host, HOST_SOURCE);
        write(
            &root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
        );
        write(
            &vue.join("package.json"),
            r#"{"name":"vue","exports":"./index.ts"}"#,
        );
        write(
            &vue.join("index.ts"),
            "export { default as RuntimeOnly } from './RuntimeOnly.vue';\n",
        );
        write(&vue.join("RuntimeOnly.vue"), "<template />\n");
        let class_component = root.path().join("node_modules/vue-class-component");
        write(
            &class_component.join("package.json"),
            r#"{"name":"vue-class-component","types":"./index.d.ts"}"#,
        );
        write(
            &class_component.join("index.d.ts"),
            "import type { Component } from 'vue';\nexport declare const Vue: Component;\n",
        );
        let property_decorator = root.path().join("node_modules/vue-property-decorator");
        write(
            &property_decorator.join("package.json"),
            r#"{"name":"vue-property-decorator","types":"./index.d.ts"}"#,
        );
        write(
            &property_decorator.join("index.d.ts"),
            "export { Vue } from 'vue-class-component';\n",
        );
        RuntimePackageFixture { _root: root, host }
    }

    fn write(path: &std::path::Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
}
