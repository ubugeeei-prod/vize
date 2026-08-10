//! Cached, importer-scoped routes for Vue editor virtual documents.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String as CompactString};

use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::dependency_scan::resolve_dependency;

#[path = "context/cache.rs"]
mod cache;
use cache::{ContextFingerprint, lock_session_cache};

/// The alias map for the host document's package, resolved once per open,
/// plus the materialized mirror the resolutions point into.
///
/// The checker resolves modules from the disk only — open in-memory documents
/// are never resolution targets — so first-party Vue imports must land on real
/// generated files. Package routes retain the importer in their key instead of
/// leaking into a global exact-specifier map.
#[allow(clippy::disallowed_types)]
pub(in crate::corsa_bridge) struct AliasContext {
    pub(in crate::corsa_bridge) project_root: PathBuf,
    pub(in crate::corsa_bridge) aliases: Vec<(std::string::String, std::string::String)>,
    pub(in crate::corsa_bridge) package_routes:
        FxHashMap<(PathBuf, CompactString), crate::PackageRoute>,
    route_inputs: Vec<PathBuf>,
    mirror: Option<VirtualProject>,
}

impl AliasContext {
    /// Build or reuse a context while every route input remains unchanged.
    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn for_host_cached(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> std::sync::Arc<Self> {
        let fingerprint = ContextFingerprint::capture(source_path, content, overlays);
        if let Some(context) = lock_session_cache().get(source_path, &fingerprint) {
            return context;
        }
        let context = std::sync::Arc::new(Self::for_host(source_path, content, overlays));
        let mut fingerprint = fingerprint;
        fingerprint.stamp(context.as_ref());
        lock_session_cache().insert(
            source_path.to_path_buf(),
            fingerprint,
            std::sync::Arc::clone(&context),
        );
        context
    }

    pub(in crate::corsa_bridge) fn for_host(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Self {
        let root = source_path
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("tsconfig.json").is_file())
            .map(Path::to_path_buf);
        let (project_root, aliases, package_routes, route_inputs, mirror) = match root {
            Some(root) => match VirtualProject::new(&root) {
                Ok(mut project) => {
                    project.set_session_script_registration(true);
                    let aliases = project.dependency_alias_map();
                    let mut resolver = crate::PackageRouteResolver::default();
                    let mut package_routes = FxHashMap::default();
                    let mut route_inputs = Vec::new();
                    let host_registered = project
                        .register_path_with_content(source_path, content)
                        .is_ok();
                    let host_specifiers = host_registered
                        .then(|| {
                            let virtual_file = project.find_by_original(source_path)?;
                            let source_type = if virtual_file
                                .virtual_path
                                .extension()
                                .is_some_and(|extension| extension == "tsx")
                            {
                                oxc_span::SourceType::tsx()
                            } else {
                                oxc_span::SourceType::ts()
                            };
                            Some(
                                crate::batch::ImportRewriter::new()
                                    .collect_all_specifiers(&virtual_file.content, source_type),
                            )
                        })
                        .flatten()
                        .unwrap_or_default();
                    let importer_dir = source_path.parent().unwrap_or(source_path);
                    let registered = {
                        let mut resolve_package = |importer_dir: &Path, specifier: &str| {
                            let lookup = resolver.lookup(
                                importer_dir,
                                specifier,
                                crate::PackageSourceOptions::new(true, true),
                            );
                            let (route, inputs) = lookup.into_parts();
                            route_inputs.extend(inputs);
                            let route = route?;
                            if !route.workspace_source {
                                return None;
                            }
                            let source_path = route.source_path.clone();
                            package_routes
                                .insert((logical_absolute(importer_dir), specifier.into()), route);
                            Some(source_path)
                        };
                        let mut workspace_package_specifiers = host_specifiers
                            .iter()
                            .filter(|specifier| {
                                resolve_package(importer_dir, specifier.as_str()).is_some()
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        workspace_package_specifiers.sort();
                        workspace_package_specifiers.dedup();
                        host_registered
                            && project
                                .register_reachable_dependencies_with_package_resolver(
                                    overlays,
                                    &workspace_package_specifiers,
                                    &mut resolve_package,
                                )
                                .is_ok()
                    };
                    route_inputs.sort();
                    route_inputs.dedup();
                    let mirror = (registered
                        && (!aliases.is_empty() || !package_routes.is_empty())
                        && project.materialize().is_ok())
                    .then_some(project);
                    (root, aliases, package_routes, route_inputs, mirror)
                }
                Err(_) => (root, Vec::new(), FxHashMap::default(), Vec::new(), None),
            },
            None => (
                source_path.parent().unwrap_or(source_path).to_path_buf(),
                Vec::new(),
                FxHashMap::default(),
                Vec::new(),
                None,
            ),
        };
        Self {
            project_root,
            aliases,
            package_routes,
            route_inputs,
            mirror,
        }
    }

    /// Resolve a Vue route to the materialized companion used by Corsa.
    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn resolve_specifier_to_mirror_path(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<std::string::String> {
        let package_source = self
            .package_route(specifier, importer_dir)
            .map(|route| route.source_path.clone());
        let path = package_source.or_else(|| {
            resolve_dependency(specifier, importer_dir, &self.project_root, &self.aliases)
        })?;
        let key = std::fs::canonicalize(&path).unwrap_or(path);
        if super::inside_node_modules(&key) || super::is_declaration(&key) {
            return None;
        }
        self.mirror_specifier_for_source(&key)
    }

    #[allow(clippy::disallowed_types)]
    pub(in crate::corsa_bridge) fn resolve_relative_vue_to_mirror_path(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<std::string::String> {
        if !specifier.starts_with("./") && !specifier.starts_with("../") {
            return None;
        }
        let source = std::fs::canonicalize(importer_dir.join(specifier)).ok()?;
        if source
            .extension()
            .is_none_or(|extension| extension != "vue")
        {
            return None;
        }
        self.mirror_specifier_for_source(&source)
    }

    #[allow(clippy::disallowed_types)]
    fn mirror_specifier_for_source(&self, source: &Path) -> Option<std::string::String> {
        let target = self
            .mirror
            .as_ref()?
            .find_by_original(source)?
            .virtual_path
            .clone();
        let spelled = target.to_string_lossy().replace('\\', "/");
        Some(
            spelled
                .strip_suffix(".tsx")
                .or_else(|| spelled.strip_suffix(".ts"))
                .unwrap_or(&spelled)
                .to_owned(),
        )
    }

    pub(in crate::corsa_bridge) fn resolve_first_party_source(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<PathBuf> {
        self.package_route(specifier, importer_dir)
            .map(|route| route.source_path.clone())
            .or_else(|| {
                resolve_dependency(specifier, importer_dir, &self.project_root, &self.aliases)
            })
    }

    pub(in crate::corsa_bridge) fn package_route(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<&crate::PackageRoute> {
        let package_key = (
            logical_absolute(importer_dir),
            CompactString::from(specifier),
        );
        self.package_routes.get(&package_key)
    }
}

fn logical_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}
