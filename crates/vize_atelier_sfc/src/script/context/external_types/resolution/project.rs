//! Immutable project options shared by parallel compiles and source snapshots.
#![expect(
    clippy::disallowed_types,
    reason = "project and capture Arcs share immutable options across parallel SFC workers"
)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use sha2::{Digest, Sha256};
use vize_carton::typescript_project::{
    CompilerOptionsContext, FlattenedCompilerOptions, normalize_path_lexically,
};
use vize_carton::{FxHashMap, String, ToCompactString, cstr};

use super::super::super::batch_epoch::{NO_EPOCH, current_batch_epoch};
use super::super::{FileStamp, file_stamp};

mod host;
use host::FileHost;

type Observations = Vec<(PathBuf, FileStamp)>;

#[derive(Debug)]
pub struct Project {
    pub(crate) config: PathBuf,
    pub(crate) revision: [u8; 32],
    root: PathBuf,
    base_url: Option<PathBuf>,
    aliases: Vec<(String, Vec<String>)>,
    inputs: Vec<PathBuf>,
    configuration_diagnostics: Vec<String>,
}

impl Project {
    /// Build a project from options captured by the host's existing authority.
    /// The host owns freshness; construction performs no filesystem reads.
    pub fn from_flattened(
        config: &Path,
        options: FlattenedCompilerOptions,
    ) -> Result<Arc<Self>, String> {
        let inputs = options.input_paths.clone();
        build(&normalize_path_lexically(config), options, inputs).map(Arc::new)
    }

    /// Identity of the captured options used by host-owned compile caches.
    pub fn cache_revision(&self) -> [u8; 32] {
        self.revision
    }

    /// Unresolved inherited configs do not prevent using the resolved options.
    /// Hosts can report these diagnostics while compilation continues.
    pub fn configuration_diagnostics(&self) -> &[String] {
        &self.configuration_diagnostics
    }

    pub(crate) fn inputs(&self) -> &[PathBuf] {
        &self.inputs
    }

    /// Watched discovery candidates include absent nearer configs so hosts can
    /// invalidate a previously inferred project when a user config appears.
    pub fn discovery_inputs(file: &Path) -> Result<Vec<PathBuf>, String> {
        Ok(discovery_for_file(file)?
            .observations
            .iter()
            .map(|(path, _)| path.clone())
            .collect())
    }

    /// Imported macro types are a TypeScript concern. Runtime-only SFCs do
    /// not perform project discovery or configuration IO.
    pub fn is_needed_by(descriptor: &crate::types::SfcDescriptor<'_>) -> bool {
        if descriptor.script_setup.is_none() {
            return false;
        }
        [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
            .into_iter()
            .flatten()
            .any(|script| {
                if !matches!(script.lang.as_deref(), Some("ts" | "tsx")) {
                    return false;
                }
                let mut summary = super::super::FileTypeSummary::default();
                super::super::extract_script_summary(&script.content, &mut summary, false);
                !summary.specifiers.is_empty()
            })
    }

    /// Find the nearest user config without loading its compiler options.
    /// Hosts can apply their project-reference ownership authority before
    /// capturing the selected config with `from_tsconfig`.
    pub fn discover_tsconfig(file: &Path) -> Result<Option<PathBuf>, String> {
        discover_for_file(file)
    }

    /// Capture one host-selected tsconfig. The immutable result can be shared
    /// by parallel SFC compiles without parsing that config for each file.
    pub fn from_tsconfig(path: &Path) -> Result<Arc<Self>, String> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| cstr!("cannot select TypeScript project: {error}"))?
                .join(path)
        };
        load(&path)
    }

    pub(crate) fn resolve(
        &self,
        specifier: &str,
        probe: impl Fn(PathBuf) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let mut selected = None;
        for (pattern, targets) in &self.aliases {
            if !pattern.contains('*') && pattern == specifier {
                selected = Some((targets, None, usize::MAX));
                break;
            }
            let Some((prefix, suffix)) = pattern.split_once('*') else {
                continue;
            };
            let Some(capture) = specifier
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix(suffix))
            else {
                continue;
            };
            if selected.is_none_or(|(_, _, length)| prefix.len() > length) {
                selected = Some((targets, Some(capture), prefix.len()));
            }
        }
        if let Some((targets, capture, _)) = selected {
            for target in targets {
                let target = capture.map_or_else(
                    || target.clone(),
                    |capture| target.replace('*', capture).into(),
                );
                let target = Path::new(target.as_str());
                let absolute = if target.is_absolute() {
                    target.to_path_buf()
                } else {
                    self.root.join(target)
                };
                if let Some(path) = probe(normalize_path_lexically(&absolute)) {
                    return Some(path);
                }
            }
        }
        self.base_url
            .as_ref()
            .and_then(|base| probe(base.join(specifier)))
    }
}

struct Discovery {
    config: Option<PathBuf>,
    observations: Observations,
    epoch: AtomicU64,
}

impl Discovery {
    fn fresh(&self, epoch: u64) -> bool {
        fresh(&self.observations, &self.epoch, epoch)
    }
}

struct Capture {
    result: Result<Arc<Project>, String>,
    observations: Observations,
    epoch: AtomicU64,
}

static DISCOVERIES: LazyLock<RwLock<FxHashMap<PathBuf, Arc<Discovery>>>> =
    LazyLock::new(|| RwLock::new(FxHashMap::default()));
type ProjectCell = Arc<Mutex<Option<Capture>>>;
static PROJECTS: LazyLock<Mutex<FxHashMap<PathBuf, ProjectCell>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

fn fresh(observations: &Observations, validated: &AtomicU64, epoch: u64) -> bool {
    if epoch != NO_EPOCH && validated.load(Ordering::Relaxed) == epoch {
        return true;
    }
    if observations
        .iter()
        .all(|(path, stamp)| file_stamp(path) == *stamp)
    {
        validated.store(epoch, Ordering::Relaxed);
        true
    } else {
        false
    }
}

pub(crate) fn for_file(file: &Path) -> Result<Option<Arc<Project>>, String> {
    discover_for_file(file)?.as_deref().map(load).transpose()
}

fn discover_for_file(file: &Path) -> Result<Option<PathBuf>, String> {
    Ok(discovery_for_file(file)?.config.clone())
}

fn discovery_for_file(file: &Path) -> Result<Arc<Discovery>, String> {
    let absolute = if file.is_absolute() {
        file.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| cstr!("cannot select TypeScript project: {error}"))?
            .join(file)
    };
    let Some(directory) = absolute.parent() else {
        return Ok(Arc::new(Discovery {
            config: None,
            observations: Vec::new(),
            epoch: AtomicU64::new(NO_EPOCH),
        }));
    };
    let directory = normalize_path_lexically(directory);
    let epoch = current_batch_epoch();
    let cached = DISCOVERIES
        .read()
        .map_err(|_| String::from("TypeScript project discovery cache unavailable"))?
        .get(&directory)
        .cloned();
    let discovery = match cached.filter(|entry| entry.fresh(epoch)) {
        Some(entry) => entry,
        None => {
            let entry = Arc::new(discover(&directory, epoch)?);
            DISCOVERIES
                .write()
                .map_err(|_| String::from("TypeScript project discovery cache unavailable"))?
                .insert(directory, Arc::clone(&entry));
            entry
        }
    };
    Ok(discovery)
}

fn discover(directory: &Path, epoch: u64) -> Result<Discovery, String> {
    let mut observations = Vec::new();
    let mut config = None;
    for parent in directory.ancestors() {
        let candidate = parent.join("tsconfig.json");
        let stamp = file_stamp(&candidate);
        observations.push((candidate.clone(), stamp));
        match std::fs::symlink_metadata(&candidate) {
            Ok(_) => {
                if !candidate.is_file() {
                    return Err(cstr!(
                        "cannot read TypeScript project {}",
                        candidate.display()
                    ));
                }
                config = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(cstr!(
                    "cannot inspect TypeScript project {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Ok(Discovery {
        config,
        observations,
        epoch: AtomicU64::new(epoch),
    })
}

pub(crate) fn load(config: &Path) -> Result<Arc<Project>, String> {
    let config = normalize_path_lexically(config);
    let cell = PROJECTS
        .lock()
        .map_err(|_| String::from("TypeScript project cache unavailable"))?
        .entry(config.clone())
        .or_default()
        .clone();
    let mut cached = cell
        .lock()
        .map_err(|_| String::from("TypeScript project capture unavailable"))?;
    let epoch = current_batch_epoch();
    if let Some(entry) = cached.as_ref()
        && fresh(&entry.observations, &entry.epoch, epoch)
    {
        return entry.result.clone();
    }
    let host = FileHost::default();
    let result = capture(&config, &host).map(Arc::new);
    let observations = host.observations();
    let result = if observations
        .iter()
        .all(|(path, stamp)| file_stamp(path) == *stamp)
    {
        result
    } else {
        Err(cstr!(
            "TypeScript project changed while loading {}",
            config.display()
        ))
    };
    *cached = Some(Capture {
        result: result.clone(),
        observations,
        epoch: AtomicU64::new(epoch),
    });
    result
}

fn capture(config: &Path, host: &FileHost) -> Result<Project, String> {
    let root = config
        .parent()
        .ok_or_else(|| String::from("TypeScript project has no parent"))?;
    let flattened = CompilerOptionsContext::observed(root).load_flattened(Some(config), host)?;
    build(
        config,
        flattened,
        host.observations()
            .into_iter()
            .map(|(path, _)| path)
            .collect(),
    )
}

fn build(
    config: &Path,
    flattened: FlattenedCompilerOptions,
    inputs: Vec<PathBuf>,
) -> Result<Project, String> {
    let root = config
        .parent()
        .ok_or_else(|| String::from("TypeScript project has no parent"))?;
    let configuration_diagnostics = flattened
        .unresolved_extends
        .iter()
        .map(|unresolved| unresolved.configuration_message())
        .collect();
    let mut aliases = Vec::new();
    if let Some(value) = flattened.options.get("paths") {
        let paths = value
            .as_object()
            .ok_or_else(|| String::from("compilerOptions.paths must be an object"))?;
        for (pattern, targets) in paths {
            if pattern.matches('*').count() > 1 {
                return Err(cstr!("invalid TypeScript paths pattern {pattern}"));
            }
            let targets = targets
                .as_array()
                .ok_or_else(|| cstr!("TypeScript paths {pattern} must contain targets"))?;
            let targets = targets
                .iter()
                .map(|target| {
                    let target = target.as_str().ok_or_else(|| {
                        cstr!("TypeScript paths {pattern} target must be a string")
                    })?;
                    if target.matches('*').count() > 1 {
                        return Err(cstr!("invalid TypeScript paths target {target}"));
                    }
                    Ok(target.to_compact_string())
                })
                .collect::<Result<Vec<_>, String>>()?;
            aliases.push((pattern.to_compact_string(), targets));
        }
    }
    let base_url = flattened.base_url.as_ref().map(|base| {
        let base = Path::new(base.as_str());
        if base.is_absolute() {
            base.to_path_buf()
        } else {
            root.join(base)
        }
    });
    let bytes =
        serde_json::to_vec(&(&flattened.options, &flattened.base_url)).map_err(|error| {
            cstr!(
                "cannot identify TypeScript project {}: {error}",
                config.display()
            )
        })?;
    Ok(Project {
        config: config.to_path_buf(),
        revision: Sha256::digest(bytes).into(),
        root: root.to_path_buf(),
        base_url,
        aliases,
        inputs,
        configuration_diagnostics,
    })
}
