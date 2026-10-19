//! Consistent inputs for one compiler-configuration capture.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, profile};

use crate::batch::error::{CorsaError, CorsaResult};
use crate::package_route::stamp::{InputStamp, InputStampReadStats, InputStampValidation};

use super::super::tsconfig_paths::{normalize_path_lexically, parse_jsonc_value};

/// Owned filesystem observations for configuration admission and repair.
/// These observations carry no compiler options and are safe to share across workers.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ConfigurationObservations {
    stamps: Vec<InputStamp>,
    changed_during_capture: bool,
}

impl ConfigurationObservations {
    /// Observe known paths before an evaluation, including link ancestors.
    /// This does not prove which bytes an external evaluator consumed.
    pub fn capture(paths: &[PathBuf]) -> Self {
        let phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "configuration_observation_capture",
            paths.len(),
        );
        let inputs = CompilerInputs::default();
        for path in paths {
            inputs.observe(path);
        }
        let observations = inputs.observations();
        phase.finish();
        observations
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
        if !crate::corsa_bridge::preparation_trace::enabled() {
            return self.validate_current_inputs(None).0;
        }
        let started = std::time::Instant::now();
        let (paths, stats, unique) = self.observed_changed_paths();
        tracing::info!(target: "vize_editor_preparation",
            phase = "configuration_observation_validation", event = "aggregate",
            entries = self.stamps.len(), unique, reads = stats.reads, bytes = stats.bytes,
            read_ms = stats.read_time.as_secs_f64() * 1000.0,
            digest_ms = stats.digest_time.as_secs_f64() * 1000.0,
            elapsed_ms = started.elapsed().as_secs_f64() * 1000.0);
        paths
    }

    fn observed_changed_paths(&self) -> (Vec<PathBuf>, InputStampReadStats, usize) {
        let mut stats = InputStampReadStats::default();
        let (paths, unique) = self.validate_current_inputs(Some(&mut stats));
        (paths, stats, unique)
    }

    fn validate_current_inputs(
        &self,
        mut stats: Option<&mut InputStampReadStats>,
    ) -> (Vec<PathBuf>, usize) {
        let mut current = InputStampValidation::default();
        let paths =
            self.collect_changed_paths(|stamp| current.is_current(stamp, stats.as_deref_mut()));
        (paths, current.len())
    }

    fn collect_changed_paths(
        &self,
        mut is_current: impl FnMut(&InputStamp) -> bool,
    ) -> Vec<PathBuf> {
        let mut paths = self
            .stamps
            .iter()
            .filter(|stamp| !is_current(stamp))
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
        let parsed = parse_jsonc_at_path(&path, &content)?;
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
    parse_jsonc_at_path(path, &content)
}

fn parse_jsonc_at_path(path: &Path, content: &str) -> CorsaResult<Value> {
    profile!("canon.tsconfig.parse", parse_jsonc_value(content)).map_err(|error| {
        CorsaError::Configuration(vize_carton::cstr!(
            "cannot parse tsconfig {}: {error}",
            path.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{CompilerInputs, ConfigurationObservations, read_jsonc};
    use crate::package_route::stamp::InputStamp;
    use crate::{CompilerConfigurationSnapshot, CorsaError, snapshot_tsconfig_compiler_options};

    #[test]
    fn owned_observations_preserve_original_read_and_absent_candidate_identity() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConfigurationObservations>();
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let base = root.join("base.json");
        let absent = root.join("candidate.json");
        std::fs::write(&base, "{}").unwrap();
        let inputs = CompilerInputs::default();
        inputs.read_jsonc(&base).unwrap();
        inputs.observe(&absent);
        let observed = inputs.observations();
        drop(inputs);
        assert!(observed.changed_paths().is_empty());
        let mut expected = vec![base.clone(), absent.clone()];
        expected.sort();
        assert_eq!(observed.paths(), expected);
        std::fs::write(&base, r#"{"strict":true}"#).unwrap();
        std::fs::write(&absent, "{}").unwrap();
        assert_eq!(observed.changed_paths(), expected);
        let repair_only = ConfigurationObservations::capture(&observed.paths());
        assert!(repair_only.changed_paths().is_empty());
        assert_eq!(observed.changed_paths(), expected);
        let inputs = CompilerInputs::default();
        inputs.observe(&root.join("base.json"));
        inputs.mark_changed();
        assert!(inputs.observations().changed_paths().is_empty());
        assert!(inputs.observations().needs_confirmation());
    }

    #[test]
    fn validation_aggregates_keep_duplicate_roles_conflicts_and_fresh_reads() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("base.json");
        let absent = directory.path().join("absent.json");
        let content = b"alpha";
        std::fs::write(&file, content).unwrap();
        let physical = InputStamp::capture(&file);
        let source = InputStamp::capture_source(&file, content);
        let mut observed = ConfigurationObservations {
            stamps: vec![
                physical.clone(),
                physical,
                source.clone(),
                source,
                InputStamp::capture(&absent),
            ],
            changed_during_capture: false,
        };
        let (paths, stats, unique) = observed.observed_changed_paths();
        assert!(paths.is_empty());
        assert_eq!(observed.stamps.len(), 5);
        assert_eq!(unique, 3);
        assert_eq!(stats.reads, 2);
        assert_eq!(stats.bytes, 10);
        assert_eq!(
            paths,
            observed.collect_changed_paths(InputStamp::is_current)
        );

        // A later validation must read disk again, even for the same paths/roles.
        std::fs::write(&file, b"omega").unwrap();
        assert_eq!(observed.changed_paths(), vec![file.clone()]);
        let conflicting = ConfigurationObservations::capture(std::slice::from_ref(&file));
        observed.extend(&conflicting);
        let (paths, stats, unique) = observed.observed_changed_paths();
        assert_eq!(paths, vec![file.clone()]);
        assert_eq!(observed.stamps.len(), 6);
        assert_eq!(unique, 3);
        assert_eq!(stats.reads, 2);
        assert_eq!(stats.bytes, 10);
        assert_eq!(
            paths,
            observed.collect_changed_paths(InputStamp::is_current)
        );
        observed.stamps.reverse();
        assert_eq!(observed.changed_paths(), paths);
        observed.stamps.reverse();
        assert!(conflicting.observed_changed_paths().0.is_empty());
        std::fs::write(&absent, content).unwrap();
        let (paths, stats, unique) = observed.observed_changed_paths();
        let mut expected = vec![file, absent];
        expected.sort();
        assert_eq!(paths, expected);
        assert_eq!(unique, 3);
        assert_eq!(stats.reads, 3);
        assert_eq!(stats.bytes, 15);
    }

    #[cfg(unix)]
    #[test]
    fn validation_aggregates_preserve_symlink_source_and_filesystem_roles() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target.json");
        let link = directory.path().join("base.json");
        let content = b"alpha";
        std::fs::write(&target, content).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let observed = ConfigurationObservations {
            stamps: vec![
                InputStamp::capture(&link),
                InputStamp::capture_source(&link, content),
            ],
            changed_during_capture: false,
        };
        let (paths, stats, unique) = observed.observed_changed_paths();
        assert!(paths.is_empty());
        assert_eq!(unique, 2);
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.bytes, 5);
        std::fs::write(&target, b"omega").unwrap();
        assert!(observed.stamps[0].is_current());
        assert!(!observed.stamps[1].is_current());
        let (paths, stats, _) = observed.observed_changed_paths();
        assert_eq!(paths, vec![link]);
        assert_eq!(
            paths,
            observed.collect_changed_paths(InputStamp::is_current)
        );
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.bytes, 5);
    }

    #[test]
    fn cached_and_live_parse_errors_keep_the_consumed_file_and_complete_parser_reason() {
        let directory = tempfile::tempdir().unwrap();
        let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
        let path = root.join("tsconfig.base.json");
        std::fs::write(&path, "{\n").unwrap();
        let expected = vize_carton::cstr!(
            "Configuration error: cannot parse tsconfig {}: JSON parse error: EOF while parsing an object at line 2 column 0",
            path.display()
        );
        let inputs = CompilerInputs::default();
        for error in [
            read_jsonc(&path, Some(&inputs)).unwrap_err(),
            read_jsonc(&path, None).unwrap_err(),
        ] {
            assert!(matches!(error, CorsaError::Configuration(_)));
            assert_eq!(vize_carton::cstr!("{error}"), expected);
        }
        std::fs::write(
            &path,
            "{/* repaired */ \"compilerOptions\": {\"strict\": true,},}\n",
        )
        .unwrap();
        // A capture keeps its original consumed revision even after physical repair.
        let error = read_jsonc(&path, Some(&inputs)).unwrap_err();
        assert_eq!(vize_carton::cstr!("{error}"), expected);
        assert!(inputs.observations().needs_confirmation());
        let repaired = serde_json::json!({"compilerOptions": {"strict": true}});
        assert_eq!(read_jsonc(&path, None).unwrap(), repaired);
        assert_eq!(
            CompilerInputs::default().read_jsonc(&path).unwrap(),
            repaired
        );
    }

    #[test]
    fn malformed_inherited_file_is_named_by_both_admitted_and_live_compiler_consumers() {
        let directory = tempfile::tempdir().unwrap();
        let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
        let app = root.join("app");
        std::fs::create_dir(&app).unwrap();
        let config = app.join("tsconfig.json");
        let base = root.join("tsconfig.base.json");
        let source =
            r#"{"extends":"../tsconfig.base.json","compilerOptions":{"strictNullChecks":false}}"#;
        std::fs::write(&config, source).unwrap();
        std::fs::write(&base, "{invalid").unwrap();
        let expected = vize_carton::cstr!(
            "Configuration error: cannot parse tsconfig {}: JSON parse error: key must be a string at line 1 column 2",
            base.display()
        );
        let attempt = CompilerConfigurationSnapshot::capture_attempt(&app, Some(&config));
        let expected_dependencies = vec![config.clone(), base.clone()];
        assert_eq!(attempt.dependencies, expected_dependencies);
        for error in [
            attempt.outcome.err().unwrap(),
            snapshot_tsconfig_compiler_options(&app, &config).unwrap_err(),
        ] {
            assert!(matches!(error, CorsaError::Configuration(_)));
            assert_eq!(vize_carton::cstr!("{error}"), expected);
        }
        std::fs::write(
            &base,
            "{/* inherited */ \"compilerOptions\": {\"strict\": true,},}\n",
        )
        .unwrap();
        let admitted = CompilerConfigurationSnapshot::capture(&app, Some(&config)).unwrap();
        assert_eq!(admitted.dependencies(), expected_dependencies);
        assert_eq!(
            snapshot_tsconfig_compiler_options(&app, &config).unwrap(),
            serde_json::json!({"strict":true,"strictNullChecks":false})
                .as_object()
                .unwrap()
                .clone()
        );
        assert_eq!(std::fs::read_to_string(&config).unwrap(), source);
    }
}
