//! Cached, importer-scoped routes for Vue editor virtual documents.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String as CompactString};

use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::dependency_scan::resolve_dependency;

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
        let cache = session_cache();
        if let Ok(mut slots) = cache.lock() {
            if let Some(cached) = slots.get(source_path)
                && cached.fingerprint == fingerprint
                && cached.fingerprint.stamps_still_valid()
            {
                return std::sync::Arc::clone(&cached.context);
            }
            slots.remove(source_path);
        }
        let context = std::sync::Arc::new(Self::for_host(source_path, content, overlays));
        let mut fingerprint = fingerprint;
        fingerprint.stamp(context.as_ref());
        if let Ok(mut slots) = cache.lock() {
            if slots.len() >= 8 {
                slots.clear();
            }
            slots.insert(
                source_path.to_path_buf(),
                CachedContext {
                    fingerprint,
                    context: std::sync::Arc::clone(&context),
                },
            );
        }
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
                            package_routes.insert(
                                (
                                    vize_carton::path::canonicalize_non_verbatim(importer_dir),
                                    specifier.into(),
                                ),
                                route,
                            );
                            Some(source_path)
                        };
                        project
                            .register_path_with_content(source_path, content)
                            .is_ok()
                            && project
                                .register_reachable_dependencies_with_package_resolver(
                                    overlays,
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
            vize_carton::path::canonicalize_non_verbatim(importer_dir),
            CompactString::from(specifier),
        );
        self.package_routes.get(&package_key)
    }
}

#[allow(clippy::disallowed_types)]
struct CachedContext {
    fingerprint: ContextFingerprint,
    context: std::sync::Arc<AliasContext>,
}

#[allow(clippy::disallowed_types)]
fn session_cache() -> &'static std::sync::Mutex<FxHashMap<PathBuf, CachedContext>> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<FxHashMap<PathBuf, CachedContext>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(FxHashMap::default()))
}

/// The import closure and disk inputs a cached editor route depends on.
#[derive(PartialEq)]
#[allow(clippy::disallowed_types)]
struct ContextFingerprint {
    host_content: u64,
    overlays: u64,
    stamps: Vec<DiskInputStamp>,
}

impl ContextFingerprint {
    #[allow(clippy::disallowed_methods)]
    fn capture(source_path: &Path, content: &str, overlays: &FxHashMap<PathBuf, &str>) -> Self {
        use std::hash::{Hash, Hasher};
        let mut host = std::hash::DefaultHasher::new();
        source_path.hash(&mut host);
        content.hash(&mut host);
        let mut overlay_entries: Vec<_> = overlays.iter().collect();
        overlay_entries.sort_by(|left, right| left.0.cmp(right.0));
        let mut overlay_hash = std::hash::DefaultHasher::new();
        for (path, text) in overlay_entries {
            path.hash(&mut overlay_hash);
            text.hash(&mut overlay_hash);
        }
        Self {
            host_content: host.finish(),
            overlays: overlay_hash.finish(),
            stamps: Vec::new(),
        }
    }

    fn stamp(&mut self, context: &AliasContext) {
        let mut paths = vec![context.project_root.join("tsconfig.json")];
        if let Some(mirror) = context.mirror.as_ref() {
            paths.extend(mirror.governing_config_paths());
            paths.extend(mirror.registered_original_paths_sorted());
        }
        paths.extend(context.route_inputs.iter().cloned());
        paths.sort();
        paths.dedup();
        self.stamps = paths.into_iter().map(DiskInputStamp::capture).collect();
    }

    fn stamps_still_valid(&self) -> bool {
        self.stamps
            .iter()
            .all(|stamp| *stamp == DiskInputStamp::capture(stamp.path.clone()))
    }
}

/// Disk identity strong enough for same-mtime edits and workspace-link retargets.
#[derive(PartialEq)]
struct DiskInputStamp {
    path: PathBuf,
    modified: Option<std::time::SystemTime>,
    len: Option<u64>,
    kind: Option<DiskInputKind>,
    content_digest: Option<u64>,
    symlink_target: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq)]
enum DiskInputKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl DiskInputStamp {
    fn capture(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let metadata = std::fs::symlink_metadata(&path).ok();
        let modified = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        let kind = metadata.as_ref().map(|metadata| {
            let file_type = metadata.file_type();
            if file_type.is_file() {
                DiskInputKind::File
            } else if file_type.is_dir() {
                DiskInputKind::Directory
            } else if file_type.is_symlink() {
                DiskInputKind::Symlink
            } else {
                DiskInputKind::Other
            }
        });
        let content_digest = matches!(kind, Some(DiskInputKind::File))
            .then(|| std::fs::read(&path).ok().map(|content| digest(&content)))
            .flatten();
        let symlink_target = matches!(kind, Some(DiskInputKind::Symlink))
            .then(|| std::fs::read_link(&path).ok())
            .flatten();
        Self {
            path,
            modified,
            len,
            kind,
            content_digest,
            symlink_target,
        }
    }
}

fn digest(content: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
