//! Lightweight project identity shared by command and editor entry points.
//!
//! This is a snapshot of paths for one invocation or workspace folder. It
//! does not scan source files or retain a TypeScript program.

use std::path::{Path, PathBuf};

use super::TypeCheckerConfig;

/// Inputs that select a Vize configuration and a TypeScript project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModel {
    root: Option<PathBuf>,
    config_source: Option<PathBuf>,
    tsconfig: Option<PathBuf>,
}

impl ProjectModel {
    /// Resolve paths declared by the already-loaded Vize configuration.
    ///
    /// Vite-owned checker paths arrive as absolute paths projected against
    /// Vite's selected root. A remaining relative `typeChecker.tsconfig` in a
    /// dedicated file belongs to the config file's directory. Without a config
    /// file it belongs to the invocation root; without either, it remains
    /// relative for the caller to resolve later.
    pub fn new(
        root: Option<&Path>,
        config_source: Option<&Path>,
        checker: &TypeCheckerConfig,
    ) -> Self {
        let base = config_source.and_then(Path::parent).or(root);
        let tsconfig = checker.tsconfig.as_deref().map(|path| {
            let path = Path::new(path);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                base.map_or_else(|| path.to_path_buf(), |base| base.join(path))
            }
        });
        Self {
            root: root.map(Path::to_path_buf),
            config_source: config_source.map(Path::to_path_buf),
            tsconfig,
        }
    }

    /// Explicit CLI selection wins over configuration and is rooted at the
    /// invocation directory, regardless of where `vize.config.*` lives.
    #[must_use]
    pub fn with_explicit_tsconfig(mut self, explicit: Option<&Path>) -> Self {
        if let Some(path) = explicit {
            self.tsconfig = Some(if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.root
                    .as_deref()
                    .map_or_else(|| path.to_path_buf(), |root| root.join(path))
            });
        }
        self
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    pub fn config_source(&self) -> Option<&Path> {
        self.config_source.as_deref()
    }

    pub fn tsconfig(&self) -> Option<&Path> {
        self.tsconfig.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{ProjectModel, TypeCheckerConfig};

    #[test]
    fn configured_tsconfig_uses_config_directory() {
        let checker = TypeCheckerConfig {
            tsconfig: Some("tsconfig.app.json".into()),
            ..Default::default()
        };
        let project = ProjectModel::new(
            Some(Path::new("/work")),
            Some(Path::new("/work/app/vize.config.json")),
            &checker,
        );
        assert_eq!(project.root(), Some(Path::new("/work")));
        assert_eq!(
            project.config_source(),
            Some(Path::new("/work/app/vize.config.json"))
        );
        assert_eq!(
            project.tsconfig(),
            Some(Path::new("/work/app/tsconfig.app.json"))
        );
    }

    #[test]
    fn explicit_tsconfig_uses_invocation_root() {
        let checker = TypeCheckerConfig {
            tsconfig: Some("tsconfig.app.json".into()),
            ..Default::default()
        };
        let project = ProjectModel::new(
            Some(Path::new("/work")),
            Some(Path::new("/work/app/vize.config.json")),
            &checker,
        )
        .with_explicit_tsconfig(Some(Path::new("other/tsconfig.json")));
        assert_eq!(
            project.tsconfig(),
            Some(Path::new("/work/other/tsconfig.json"))
        );
    }

    #[test]
    fn absolute_and_unrooted_paths_keep_their_identity() {
        let checker = TypeCheckerConfig {
            tsconfig: Some("/external/tsconfig.json".into()),
            ..Default::default()
        };
        let project = ProjectModel::new(None, None, &checker);
        assert_eq!(
            project.tsconfig(),
            Some(Path::new("/external/tsconfig.json"))
        );
        let checker = TypeCheckerConfig {
            tsconfig: Some("tsconfig.json".into()),
            ..Default::default()
        };
        let project = ProjectModel::new(None, None, &checker);
        assert_eq!(project.tsconfig(), Some(Path::new("tsconfig.json")));
    }
}
