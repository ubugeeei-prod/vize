//! Which projection a content-mapper host receives.
//!
//! A command-line type-check reads only diagnostics, so it gets the
//! projection `vize check` builds, without editor navigation scaffolding.
//! Any other host may ask for hover, definition or rename, so the editor
//! projection stays the default. TypeScript names no host kind: `initialize`
//! carries the locale and position encodings, and `openProject` the
//! effective compiler options. Those include command-line flags, but the
//! editor host also forwards whatever a tsconfig sets (`checkers`,
//! `listFiles`, `pretty`, even `project`), so only values a tsconfig cannot
//! produce prove a command line.

use std::path::Path;

use serde_json::Value;

/// Overrides detection with `check` or `editor`; other values are ignored.
pub(super) const MODE_ENV: &str = "VIZE_CONTENT_MAPPER_MODE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProjectionMode {
    Editor,
    Check,
}

impl ProjectionMode {
    /// The launcher's explicit choice, read once per server.
    pub(super) fn from_env() -> Option<Self> {
        Self::parse(std::env::var(MODE_ENV).ok()?.as_str())
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "check" => Some(Self::Check),
            "editor" => Some(Self::Editor),
            _ => None,
        }
    }

    pub(super) fn resolve(
        explicit: Option<Self>,
        config_file_name: &str,
        compiler_options: &Value,
    ) -> Self {
        explicit.unwrap_or_else(|| {
            if command_line_type_check(config_file_name, compiler_options) {
                Self::Check
            } else {
                Self::Editor
            }
        })
    }

    /// Cached transforms of one mode never serve the other.
    pub(super) fn config_identity(self, identity: &str) -> vize_l0::String {
        match self {
            Self::Editor => identity.into(),
            Self::Check => vize_l0::cstr!("{identity}:check"),
        }
    }
}

/// `--watch` and `--build` are refused in a tsconfig (TS6266, TS5023). `-p`
/// sets `project` to the selected config or its directory. The editor host
/// forwards a tsconfig `project` too, resolved against the config directory;
/// only one pointing back at its own config looks like `-p`.
fn command_line_type_check(config_file_name: &str, compiler_options: &Value) -> bool {
    let Some(options) = compiler_options.as_object() else {
        return false;
    };
    if ["watch", "build"]
        .iter()
        .any(|flag| options.get(*flag) == Some(&Value::Bool(true)))
    {
        return true;
    }
    let config = Path::new(config_file_name);
    !config_file_name.is_empty()
        && options
            .get("project")
            .and_then(Value::as_str)
            .map(Path::new)
            .is_some_and(|project| project == config || config.parent() == Some(project))
}
