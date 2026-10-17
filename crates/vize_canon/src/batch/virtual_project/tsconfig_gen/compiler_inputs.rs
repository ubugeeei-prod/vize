//! Consistent inputs for one compiler-configuration capture.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, profile};

use crate::batch::error::{CorsaError, CorsaResult};
use crate::package_route::stamp::InputStamp;

use super::super::tsconfig_paths::{normalize_path_lexically, parse_jsonc_value};

/// Owned filesystem observations for configuration admission and repair.
/// These observations carry no compiler options and are safe to share across workers.
#[derive(Clone, Default)]
pub struct ConfigurationObservations {
    stamps: Vec<InputStamp>,
    changed_during_capture: bool,
}

impl ConfigurationObservations {
    /// Observe known paths before an evaluation, including link ancestors.
    /// This does not prove which bytes an external evaluator consumed.
    pub fn capture(paths: &[PathBuf]) -> Self {
        let inputs = CompilerInputs::default();
        for path in paths {
            inputs.observe(path);
        }
        inputs.observations()
    }

    /// Preserve both consumed-source observations and known-before observations.
    pub fn extend(&mut self, other: &Self) {
        self.stamps.extend(other.stamps.iter().cloned());
        self.changed_during_capture |= other.changed_during_capture;
    }

    /// A rejected read must confirm the next stable revision, even if inputs
    /// subsequently return to the observed bytes before repair is recorded.
    pub fn needs_confirmation(&self) -> bool {
        self.changed_during_capture || !self.changed_paths().is_empty()
    }

    /// Known read and resolution paths, including absent candidates.
    pub fn paths(&self) -> Vec<PathBuf> {
        let mut paths = self
            .stamps
            .iter()
            .map(|stamp| stamp.path().to_path_buf())
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        paths
    }

    /// Inputs whose current physical identity differs from this observation.
    pub fn changed_paths(&self) -> Vec<PathBuf> {
        let mut paths = self
            .stamps
            .iter()
            .filter(|stamp| !stamp.is_current())
            .map(|stamp| stamp.path().to_path_buf())
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        paths
    }
}

#[derive(Eq, Hash, PartialEq)]
struct ResolutionKey {
    kind: &'static str,
    origin: PathBuf,
    specifier: vize_carton::String,
}

#[derive(Default)]
pub(in super::super) struct CompilerInputs {
    parsed: RefCell<FxHashMap<PathBuf, Value>>,
    content: RefCell<FxHashMap<PathBuf, vize_carton::String>>,
    observations: RefCell<FxHashMap<PathBuf, InputStamp>>,
    parsed_observations: RefCell<Vec<InputStamp>>,
    resolutions: RefCell<FxHashMap<ResolutionKey, Option<PathBuf>>>,
    changed_during_capture: Cell<bool>,
}

impl CompilerInputs {
    pub(in super::super) fn observe(&self, path: &Path) {
        let path = normalize_path_lexically(path);
        self.observations
            .borrow_mut()
            .entry(path.clone())
            .or_insert_with(|| InputStamp::capture(&path));
        // Package resolution follows links in ancestors, not just the final file.
        for ancestor in path.ancestors().skip(1) {
            if std::fs::symlink_metadata(ancestor)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                self.observations
                    .borrow_mut()
                    .entry(ancestor.to_path_buf())
                    .or_insert_with(|| InputStamp::capture(ancestor));
            }
        }
    }

    pub(in super::super) fn read_jsonc(&self, path: &Path) -> CorsaResult<Value> {
        let path = normalize_path_lexically(path);
        if let Some(parsed) = self.parsed.borrow().get(&path) {
            return Ok(parsed.clone());
        }
        let content = self.read_content(&path)?;
        let parsed = profile!("canon.tsconfig.parse", parse_jsonc_value(&content))?;
        self.parsed.borrow_mut().insert(path, parsed.clone());
        Ok(parsed)
    }

    fn read_content(&self, path: &Path) -> CorsaResult<vize_carton::String> {
        if let Some(content) = self.content.borrow().get(path) {
            return Ok(content.clone());
        }
        self.observe(path);
        let content =
            profile!("canon.tsconfig.read", std::fs::read_to_string(path)).map_err(|error| {
                CorsaError::Configuration(vize_carton::cstr!(
                    "cannot read tsconfig {}: {error}",
                    path.display()
                ))
            })?;
        self.parsed_observations
            .borrow_mut()
            .push(InputStamp::capture_source(path, content.as_bytes()));
        let content = vize_carton::String::from(content);
        self.content
            .borrow_mut()
            .insert(path.to_path_buf(), content.clone());
        Ok(content)
    }

    pub(in super::super) fn resolve(
        &self,
        kind: &'static str,
        origin: &Path,
        specifier: &str,
        resolve: impl FnOnce() -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let key = ResolutionKey {
            kind,
            origin: normalize_path_lexically(origin),
            specifier: vize_carton::String::from(specifier),
        };
        if let Some(resolved) = self.resolutions.borrow().get(&key) {
            return resolved.clone();
        }
        let resolved = resolve();
        self.resolutions.borrow_mut().insert(key, resolved.clone());
        resolved
    }

    pub(in super::super) fn installed_vue_manifest(
        &self,
        directory: &Path,
    ) -> Option<(PathBuf, Value)> {
        let path = self.resolve("package", directory, "vue", || {
            crate::package_route::installed_package_manifest_with_reader(
                directory,
                "vue",
                |path| {
                    self.read_content(path)
                        .ok()
                        .and_then(|content| serde_json::from_str(&content).ok())
                },
                |path| self.observe(path),
            )
            .map(|(path, _)| path)
        })?;
        let manifest = serde_json::from_str(&self.read_content(&path).ok()?).ok()?;
        Some((path, manifest))
    }

    pub(in super::super) fn observations(&self) -> ConfigurationObservations {
        let mut stamps = self
            .observations
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        stamps.extend(self.parsed_observations.borrow().iter().cloned());
        ConfigurationObservations {
            stamps,
            changed_during_capture: self.changed_during_capture.get(),
        }
    }

    pub(in super::super) fn mark_changed(&self) {
        self.changed_during_capture.set(true);
    }

    pub(in super::super) fn validate(&self) -> CorsaResult<()> {
        if self
            .observations
            .borrow()
            .values()
            .all(InputStamp::is_current)
            && self
                .parsed_observations
                .borrow()
                .iter()
                .all(InputStamp::is_current)
        {
            return Ok(());
        }
        self.mark_changed();
        Err(CorsaError::Configuration(vize_carton::cstr!(
            "compiler configuration changed while it was being read; retry after the configuration is stable"
        )))
    }

    pub(in super::super) fn paths(&self) -> Vec<PathBuf> {
        self.observations.borrow().keys().cloned().collect()
    }
}

pub(in super::super) fn read_jsonc(
    path: &Path,
    inputs: Option<&CompilerInputs>,
) -> CorsaResult<Value> {
    if let Some(inputs) = inputs {
        return inputs.read_jsonc(path);
    }
    let content =
        profile!("canon.tsconfig.read", std::fs::read_to_string(path)).map_err(|error| {
            CorsaError::Configuration(vize_carton::cstr!(
                "cannot read tsconfig {}: {error}",
                path.display()
            ))
        })?;
    profile!("canon.tsconfig.parse", parse_jsonc_value(&content))
}

#[cfg(test)]
mod tests {
    use super::{CompilerInputs, ConfigurationObservations};

    #[test]
    fn owned_observations_preserve_original_read_and_absent_candidate_identity() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConfigurationObservations>();
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("base.json");
        let absent = root.path().join("candidate.json");
        std::fs::write(&base, "{}").unwrap();
        let inputs = CompilerInputs::default();
        inputs.read_jsonc(&base).unwrap();
        inputs.observe(&absent);
        let observed = inputs.observations();
        drop(inputs);
        assert!(observed.changed_paths().is_empty());
        assert!(observed.paths().contains(&absent));
        std::fs::write(&base, r#"{"strict":true}"#).unwrap();
        std::fs::write(&absent, "{}").unwrap();
        let mut expected = vec![base, absent];
        expected.sort();
        assert_eq!(observed.changed_paths(), expected);
        let repair_only = ConfigurationObservations::capture(&observed.paths());
        assert!(repair_only.changed_paths().is_empty());
        assert_eq!(observed.changed_paths(), expected);
        let inputs = CompilerInputs::default();
        inputs.observe(&root.path().join("base.json"));
        inputs.mark_changed();
        assert!(inputs.observations().changed_paths().is_empty());
        assert!(inputs.observations().needs_confirmation());
    }
}
