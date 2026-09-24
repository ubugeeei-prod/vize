//! Registry source resolution.
//!
//! First-party sources (`ui`, `composable`): `--registry` paths, then the
//! project's installed `node_modules/@vizejs/<pkg>/registry/registry.json`,
//! then (for an explicit version, or when not installed) `npm pack` + `tar`.
//!
//! Third-party sources (`@ns`) come from `lib.registries` in vize.config: a
//! local path, `npm:<package>[@range]` (installed first, else `npm pack`), or
//! an `https://` URL of a `registry.json` (fetched with curl).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use vize_s0::{String, cstr};

use super::error::{LibError, LibResult};
use super::fetch::{Tools, installed_manifest, pack_registry, url_registry};
use super::registry::LoadedRegistry;

/// Where items come from: a first-party package or a configured namespace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Ui,
    Composable,
    /// Third-party registry namespace, including the leading `@`.
    Namespace(String),
}

impl Source {
    pub const BUILTIN: [Self; 2] = [Self::Ui, Self::Composable];

    /// Lockfile key: `ui`, `composable`, or `@ns`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ui => "ui",
            Self::Composable => "composable",
            Self::Namespace(namespace) => namespace,
        }
    }

    /// Package for first-party sources.
    pub fn builtin_package(&self) -> Option<&'static str> {
        match self {
            Self::Ui => Some("@vizejs/ui"),
            Self::Composable => Some("@vizejs/composable"),
            Self::Namespace(_) => None,
        }
    }

    /// Display form of an item: `ui:switch` or `@acme/button`.
    pub fn label(&self, name: &str) -> String {
        match self {
            Self::Namespace(namespace) => cstr!("{namespace}/{name}"),
            _ => cstr!("{}:{name}", self.as_str()),
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "ui" => Some(Self::Ui),
            "composable" => Some(Self::Composable),
            namespace if is_namespace(namespace) => Some(Self::Namespace(namespace.into())),
            _ => None,
        }
    }
}

/// `@[a-z0-9][a-z0-9._-]*`
pub fn is_namespace(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('@') else {
        return false;
    };
    let mut characters = rest.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && characters.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
}

/// Parsed `lib.registries` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Path(PathBuf),
    Npm {
        package: String,
        range: Option<String>,
    },
    Url(String),
}

impl Origin {
    /// Parse a configured source; relative paths resolve against `base`.
    pub fn parse(source: &str, base: &Path) -> LibResult<Self> {
        if source.starts_with("https://") {
            return Ok(Self::Url(source.into()));
        }
        if source.starts_with("http://") {
            return Err(LibError::new(cstr!("registry URL {source} must use https")));
        }
        if let Some(spec) = source.strip_prefix("npm:") {
            // `@scope/name@range` or `name@range`; the first `@` of a scope is not a separator.
            let (package, range) = match spec.get(1..).and_then(|rest| rest.split_once('@')) {
                Some((head, range)) => (
                    cstr!("{}{head}", spec.get(..1).unwrap_or_default()),
                    Some(range),
                ),
                None => (String::from(spec), None),
            };
            if package.is_empty() {
                return Err(LibError::new(cstr!("invalid npm registry source {source}")));
            }
            return Ok(Self::Npm {
                package,
                range: range.filter(|range| !range.is_empty()).map(String::from),
            });
        }
        Ok(Self::Path(base.join(source)))
    }
}

/// A configured namespace.
#[derive(Debug, Clone)]
pub struct Namespace {
    pub origin: Origin,
    pub dir: Option<String>,
}

/// Loads registries lazily and memoizes them per source and version.
pub struct Resolver {
    project_root: PathBuf,
    explicit: Vec<LoadedRegistry>,
    namespaces: BTreeMap<String, Namespace>,
    offline: bool,
    tools: Tools,
    loaded: Vec<(Source, LoadedRegistry)>,
}

impl Resolver {
    /// Build a resolver; `explicit` paths may name `registry.json`, a registry
    /// directory, or a package directory.
    pub fn new(
        project_root: &Path,
        explicit: &[PathBuf],
        namespaces: BTreeMap<String, Namespace>,
        offline: bool,
    ) -> LibResult<Self> {
        let mut registries: Vec<LoadedRegistry> = Vec::new();
        for path in explicit {
            let registry = LoadedRegistry::load(&manifest_path_for(path)?, "--registry")?;
            if registries
                .iter()
                .any(|seen| seen.manifest.kind == registry.manifest.kind)
            {
                return Err(LibError::new(cstr!(
                    "more than one --registry provides {} items",
                    registry.manifest.kind
                )));
            }
            registries.push(registry);
        }
        Ok(Self {
            project_root: project_root.to_path_buf(),
            explicit: registries,
            namespaces,
            offline,
            tools: Tools::default(),
            loaded: Vec::new(),
        })
    }

    pub fn tools_mut(&mut self) -> &mut Tools {
        &mut self.tools
    }

    pub fn tools(&self) -> &Tools {
        &self.tools
    }

    pub fn offline(&self) -> bool {
        self.offline
    }

    /// Whether first-party registries were passed explicitly (auto-discovery is then off).
    pub fn has_explicit(&self) -> bool {
        !self.explicit.is_empty()
    }

    pub fn namespace(&self, name: &str) -> Option<&Namespace> {
        self.namespaces.get(name)
    }

    /// Every source to search when a spec names none: first-party, then namespaces.
    pub fn all_sources(&self) -> Vec<Source> {
        Source::BUILTIN
            .into_iter()
            .chain(
                self.namespaces
                    .keys()
                    .map(|name| Source::Namespace(name.clone())),
            )
            .collect()
    }

    /// Registry for `source`, optionally pinned to an exact package version.
    pub fn registry(
        &mut self,
        source: &Source,
        version: Option<&str>,
    ) -> LibResult<&LoadedRegistry> {
        let version_matches = |registry: &LoadedRegistry| {
            version.is_none_or(|version| registry.manifest.package.version == version)
        };
        if let Some(package) = source.builtin_package()
            && self.has_explicit()
        {
            let registry = self
                .explicit
                .iter()
                .find(|registry| registry.manifest.kind == source.as_str())
                .ok_or_else(|| {
                    LibError::new(cstr!("no --registry provides {} items", source.as_str()))
                })?;
            if !version_matches(registry) {
                return Err(LibError::new(cstr!(
                    "--registry provides {}, not {package}@{}",
                    registry.package_label(),
                    version.unwrap_or("?")
                )));
            }
            return Ok(registry);
        }
        if let Some(index) = self
            .loaded
            .iter()
            .position(|(loaded, registry)| loaded == source && version_matches(registry))
        {
            return self
                .loaded
                .get(index)
                .map(|(_, registry)| registry)
                .ok_or_else(|| LibError::new("registry cache out of range"));
        }
        let registry = self.load(source, version)?;
        if !version_matches(&registry) {
            return Err(LibError::new(cstr!(
                "{} provides {}, not version {}",
                source.as_str(),
                registry.package_label(),
                version.unwrap_or("?")
            )));
        }
        self.loaded.push((source.clone(), registry));
        self.loaded
            .last()
            .map(|(_, registry)| registry)
            .ok_or_else(|| LibError::new("registry cache is empty"))
    }

    fn load(&self, source: &Source, version: Option<&str>) -> LibResult<LoadedRegistry> {
        let (package, range) = match source {
            Source::Namespace(name) => {
                let namespace = self.namespaces.get(name.as_str()).ok_or_else(|| {
                    LibError::new(cstr!(
                        "unknown registry namespace {name}; add it to lib.registries in vize.config"
                    ))
                })?;
                match &namespace.origin {
                    Origin::Path(path) => {
                        return LoadedRegistry::load(
                            &manifest_path_for(path)?,
                            cstr!("{name} (path)"),
                        );
                    }
                    Origin::Url(url) => {
                        if self.offline {
                            return Err(LibError::new(cstr!(
                                "{name} is fetched from {url} and --offline forbids it"
                            )));
                        }
                        return url_registry(&self.tools.curl, url);
                    }
                    Origin::Npm { package, range } => (package.as_str(), range.as_deref()),
                }
            }
            _ => (source.builtin_package().unwrap_or_default(), None),
        };
        let installed = match installed_manifest(&self.project_root, package) {
            Some(path) => Some(LoadedRegistry::load(&path, "node_modules")?),
            None => None,
        };
        match installed {
            Some(installed)
                if version.is_none_or(|version| installed.manifest.package.version == version) =>
            {
                Ok(installed)
            }
            installed if self.offline => Err(LibError::new(match installed {
                Some(found) => cstr!(
                    "installed {} does not match {package}@{} and --offline forbids npm pack",
                    found.package_label(),
                    version.unwrap_or("latest")
                ),
                None => cstr!("{package} is not installed and --offline forbids npm pack"),
            })),
            _ => pack_registry(&self.tools.npm, package, version.or(range)),
        }
    }

    /// Registry for `source` if one is available without touching the network.
    pub fn local_registry(&mut self, source: &Source) -> Option<&LoadedRegistry> {
        let previous = self.offline;
        self.offline = true;
        let found = self.registry(source, None).is_ok();
        self.offline = previous;
        if found {
            self.registry(source, None).ok()
        } else {
            None
        }
    }
}

/// Map a `--registry` argument or configured path to its `registry.json`.
pub fn manifest_path_for(path: &Path) -> LibResult<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    for candidate in [
        path.join("registry.json"),
        path.join("registry").join("registry.json"),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(LibError::new(cstr!(
        "no registry.json at {} (expected a file, a registry directory, or a package directory)",
        path.display()
    )))
}
