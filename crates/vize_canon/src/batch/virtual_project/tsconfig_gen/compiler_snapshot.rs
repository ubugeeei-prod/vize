//! Immutable compiler authority admitted by one editor project owner.

use std::path::{Path, PathBuf};
#[expect(
    clippy::disallowed_types,
    reason = "immutable snapshots are shared by editor owners"
)]
use std::sync::Arc;

use serde_json::{Map, Value};

use super::super::VirtualProject;
pub use super::compiler_inputs::ConfigurationObservations;
use super::compiler_options::FlattenedCompilerOptions;
use crate::batch::error::{CorsaError, CorsaResult};

/// Effective options and their original path anchors survive a rejected editor
/// reload. Source buffers and package files remain live; only the admitted
/// compiler configuration is retained until its owner is replaced.
#[derive(Clone)]
#[expect(
    clippy::disallowed_types,
    reason = "serde_json object keys use std String"
)]
pub struct CompilerConfigurationSnapshot {
    project_root: PathBuf,
    tsconfig_path: Option<PathBuf>,
    pub(super) flattened: FlattenedCompilerOptions,
    pub(super) verbatim: Map<std::string::String, Value>,
    pub(super) vue_options: Map<std::string::String, Value>,
    dependencies: Vec<PathBuf>,
    vue_target: Result<Option<vize_carton::config::VueVersion>, vize_carton::String>,
    solution_shell: bool,
    observations: ConfigurationObservations,
}

/// One capture result plus the exact paths observed while attempting it.
/// Failed attempts expose watch-only repair inputs, never admitted options.
pub struct CompilerConfigurationAttempt {
    /// Only a successful capture contains a snapshot eligible for admission.
    pub outcome: CorsaResult<CompilerConfigurationSnapshot>,
    /// Attempted reads and resolution candidates for watching, not compiler authority.
    pub dependencies: Vec<PathBuf>,
    /// Original read/resolution observations, even when admission failed.
    pub observations: ConfigurationObservations,
}

#[cfg(test)]
#[path = "compiler_snapshot_tests.rs"]
mod tests;

impl CompilerConfigurationSnapshot {
    /// `extends` entries this admitted chain skipped, as TypeScript does.
    pub fn unresolved_extends(&self) -> &[super::UnresolvedExtends] {
        &self.flattened.unresolved_extends
    }

    /// Compare complete compiler authority, excluding only observation metadata.
    /// Exhaustive destructuring makes added authority fields require a decision.
    pub fn same_authority(&self, other: &Self) -> bool {
        let Self {
            project_root,
            tsconfig_path,
            flattened,
            verbatim,
            vue_options,
            dependencies,
            vue_target,
            solution_shell,
            observations: _,
        } = self;
        let Self {
            project_root: other_root,
            tsconfig_path: other_path,
            flattened: other_flattened,
            verbatim: other_verbatim,
            vue_options: other_vue,
            dependencies: other_dependencies,
            vue_target: other_target,
            solution_shell: other_shell,
            observations: _,
        } = other;
        (
            project_root,
            tsconfig_path,
            flattened,
            verbatim,
            vue_options,
            dependencies,
            vue_target,
            solution_shell,
        ) == (
            other_root,
            other_path,
            other_flattened,
            other_verbatim,
            other_vue,
            other_dependencies,
            other_target,
            other_shell,
        )
    }

    /// Capture with consistent per-file reads and resolution outcomes through
    /// the existing checked resolver, then validate every observed input once.
    /// Changes reject admission; this is not a multi-file filesystem transaction.
    pub fn capture(project_root: &Path, tsconfig_path: Option<&Path>) -> CorsaResult<Self> {
        Self::capture_with_observer(project_root, tsconfig_path, || {})
    }

    /// Retain watch-only dependency observations even when strict capture fails.
    pub fn capture_attempt(
        project_root: &Path,
        tsconfig_path: Option<&Path>,
    ) -> CompilerConfigurationAttempt {
        Self::capture_attempt_with_observer(project_root, tsconfig_path, || {})
    }

    fn capture_with_observer(
        project_root: &Path,
        tsconfig_path: Option<&Path>,
        after_flattened: impl FnOnce(),
    ) -> CorsaResult<Self> {
        Self::capture_attempt_with_observer(project_root, tsconfig_path, after_flattened).outcome
    }

    fn capture_attempt_with_observer(
        project_root: &Path,
        tsconfig_path: Option<&Path>,
        after_flattened: impl FnOnce(),
    ) -> CompilerConfigurationAttempt {
        let phase = crate::corsa_bridge::preparation_trace::Phase::start(
            "compiler_configuration_capture",
            1,
        );
        let inputs = super::compiler_inputs::CompilerInputs::default();
        let normalized = tsconfig_path
            .map(|path| vize_carton::path::normalize_windows_verbatim_path(path.to_path_buf()));
        let tsconfig_path = normalized.as_deref();
        let implicit_root = vize_carton::path::canonicalize_non_verbatim(project_root);
        let implicit_candidates = [
            implicit_root.join("tsconfig.json"),
            implicit_root.join("jsconfig.json"),
        ];
        if let Some(path) = tsconfig_path {
            inputs.observe(path);
        } else {
            for path in &implicit_candidates {
                inputs.observe(path);
            }
        }
        let outcome = (|| {
            let project = VirtualProject::new(&implicit_root)?;
            if tsconfig_path.is_none() {
                for path in &implicit_candidates {
                    inputs.observe(path);
                    if path.is_file() {
                        inputs.mark_changed();
                        return Err(CorsaError::Configuration(vize_carton::cstr!(
                            "TypeScript configuration {} appeared while admitting configuration absence",
                            path.display()
                        )));
                    }
                }
            }
            let flattened = project
                .load_compiler_options_flattened_with_inputs(tsconfig_path, Some(&inputs))?;
            after_flattened();
            let verbatim =
                project.load_compiler_options_verbatim_with_inputs(tsconfig_path, Some(&inputs))?;
            let vue_options = super::vue_compiler_options::load_vue_compiler_options_with_inputs(
                tsconfig_path,
                Some(&inputs),
            )?;
            let vue_target = tsconfig_path
                .map_or(Ok(None), |path| {
                    super::vue_compiler_options::vue_target_from_options(
                        project_root,
                        path,
                        &vue_options,
                        Some(&inputs),
                    )
                })
                .map_err(|error| vize_carton::cstr!("{error}"));
            let solution_shell = tsconfig_path.is_some_and(|path| {
                super::references::graph::collect_project_paths_with_inputs(path, Some(&inputs))
                    .len()
                    > 1
            });
            let mut configured = project;
            // Dependency traversal uses the same input cache; avoid the live-policy
            // refresh performed by the public setter during ordinary source loading.
            configured.tsconfig_path = tsconfig_path.map(Path::to_path_buf);
            let mut dependencies = if tsconfig_path.is_some() {
                configured.governing_config_paths_with_inputs(Some(&inputs))
            } else {
                implicit_candidates.to_vec()
            };
            dependencies.extend(inputs.paths());
            dependencies.sort();
            dependencies.dedup();
            inputs.validate()?;
            Ok(Self {
                project_root: configured.project_root().to_path_buf(),
                tsconfig_path: tsconfig_path.map(Path::to_path_buf),
                flattened,
                verbatim,
                vue_options,
                dependencies,
                vue_target,
                solution_shell,
                observations: inputs.observations(),
            })
        })();
        let mut dependencies = inputs.paths();
        if let Ok(snapshot) = &outcome {
            dependencies.extend(snapshot.dependencies().iter().cloned());
        }
        dependencies.sort();
        dependencies.dedup();
        let attempt = CompilerConfigurationAttempt {
            outcome,
            dependencies,
            observations: inputs.observations(),
        };
        phase.finish();
        attempt
    }

    /// The actual inputs checked during this capture, not a later reread.
    pub fn observations(&self) -> &ConfigurationObservations {
        &self.observations
    }

    /// The dialect resolved from these same captured options and Vue manifest.
    /// Callers apply an explicitly authored Vite dialect before consulting it.
    pub fn vue_target(
        &self,
    ) -> Result<Option<vize_carton::config::VueVersion>, vize_carton::String> {
        self.vue_target.clone()
    }

    /// Whether the captured selected config is a project-reference shell.
    pub fn is_solution_shell(&self) -> bool {
        self.solution_shell
    }

    pub(crate) fn tsconfig_path(&self) -> Option<&Path> {
        self.tsconfig_path.as_deref()
    }

    /// Config and Vue-manifest inputs retained for watcher repair.
    pub fn dependencies(&self) -> &[PathBuf] {
        &self.dependencies
    }
}

impl VirtualProject {
    pub(crate) fn has_admitted_compiler_configuration(&self) -> bool {
        self.compiler_configuration.is_some()
    }

    pub(crate) fn is_admitted_compiler_input(&self, path: &Path) -> bool {
        self.compiler_configuration
            .as_ref()
            .is_some_and(|snapshot| {
                let inputs = if snapshot.tsconfig_path.is_some() {
                    &snapshot.flattened.input_paths
                } else {
                    &snapshot.dependencies
                };
                inputs.contains(&super::super::tsconfig_paths::normalize_path_lexically(
                    path,
                ))
            })
    }

    /// The embedding editor already selected this complete project owner.
    /// Do not select another reference from mutable disk during a rejected reload.
    #[expect(
        clippy::disallowed_types,
        reason = "the admitted immutable owner shares compiler authority across native rebuilds"
    )]
    pub(crate) fn set_compiler_configuration_snapshot(
        &mut self,
        snapshot: Arc<CompilerConfigurationSnapshot>,
    ) -> CorsaResult<()> {
        if self.project_root != snapshot.project_root {
            return Err(CorsaError::Configuration(vize_carton::cstr!(
                "admitted compiler configuration belongs to {}, not {}",
                snapshot.project_root.display(),
                self.project_root.display()
            )));
        }
        let tsconfig = snapshot.tsconfig_path.clone();
        self.compiler_configuration = Some(snapshot);
        self.set_tsconfig_path(tsconfig);
        Ok(())
    }
}
