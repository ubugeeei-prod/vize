//! Project lifecycle state for the TypeScript content-mapper server.
//!
//! The merged upstream protocol (microsoft/typescript-go#4712) opens one
//! mapper project per TypeScript project before any transform: `openProject`
//! carries the tsconfig entry's mapper options and the project's effective
//! compiler options, and every `transform` then references the opened project
//! by its host-assigned handle. Invalid mapper options are reported as
//! `optionDiagnostics` addressed into the entry's options object instead of
//! failing the request, so TypeScript can surface them in the tsconfig.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use vize_s0::{FxHashMap, String as CompactString, cstr};

/// Mapper diagnostic code for an options value that is not an object.
const OPTION_CODE_NOT_AN_OBJECT: i32 = 1;
/// Mapper diagnostic code for an option Vize does not recognize.
const OPTION_CODE_UNKNOWN_OPTION: i32 = 2;
/// Mapper diagnostic code for a recognized option holding the wrong type.
const OPTION_CODE_INVALID_TYPE: i32 = 3;

/// Mapper settings resolved when TypeScript opens a project.
#[derive(Clone, Copy, Debug)]
pub(super) struct ProjectSettings {
    /// Resolve Vue Options API instance bindings in templates.
    pub options_api: bool,
    /// Preserve diagnostics for user-authored unused locals.
    pub no_unused_locals: bool,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            options_api: true,
            no_unused_locals: false,
        }
    }
}

/// Open mapper projects, keyed by the host-assigned opaque handle.
#[derive(Default)]
pub(super) struct ProjectRegistry {
    projects: FxHashMap<CompactString, ProjectSettings>,
}

impl ProjectRegistry {
    pub fn open(&mut self, handle: CompactString, settings: ProjectSettings) {
        self.projects.insert(handle, settings);
    }

    pub fn close(&mut self, handle: &str) {
        self.projects.remove(handle);
    }

    pub fn settings(&self, handle: &str) -> Option<ProjectSettings> {
        self.projects.get(handle).copied()
    }
}

/// The `openProject` request parameters.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OpenProjectParams {
    /// The absolute project configuration file name, or empty when none exists.
    #[allow(dead_code)]
    #[serde(default)]
    config_file_name: CompactString,
    pub project_handle: CompactString,
    /// The mapper entry's options from the project's `contentMappers` configuration.
    #[serde(default)]
    options: Value,
    /// The project's effective compiler options.
    #[serde(default)]
    compiler_options: Value,
}

/// The `closeProject` request parameters.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CloseProjectParams {
    pub project_handle: CompactString,
}

/// An invalid mapper option, addressed relative to the entry's options object.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OptionDiagnostic {
    path: Vec<Value>,
    message_text: CompactString,
    code: i32,
}

/// The `openProject` response. Vize does not declare `dynamicConfig`, so it
/// must return an empty `configIdentity` and no `watchedFiles`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OpenProjectResult {
    config_identity: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    option_diagnostics: Vec<OptionDiagnostic>,
}

/// Resolve one `openProject` request into settings and option diagnostics.
///
/// Invalid options degrade to their defaults so the project stays usable while
/// TypeScript reports the diagnostics against the tsconfig entry.
pub(super) fn resolve_open_project(
    params: &OpenProjectParams,
) -> (ProjectSettings, OpenProjectResult) {
    let mut settings = ProjectSettings::default();
    let mut diagnostics = Vec::new();
    match &params.options {
        Value::Null => {}
        Value::Object(options) => {
            for (name, value) in options {
                match name.as_str() {
                    "optionsApi" => match value {
                        Value::Bool(enabled) => settings.options_api = *enabled,
                        _ => diagnostics.push(OptionDiagnostic {
                            path: vec![json!(name)],
                            message_text: cstr!("Option '{name}' requires a value of type boolean"),
                            code: OPTION_CODE_INVALID_TYPE,
                        }),
                    },
                    _ => diagnostics.push(OptionDiagnostic {
                        path: vec![json!(name)],
                        message_text: cstr!("Unknown option '{name}'"),
                        code: OPTION_CODE_UNKNOWN_OPTION,
                    }),
                }
            }
        }
        _ => diagnostics.push(OptionDiagnostic {
            path: Vec::new(),
            message_text: CompactString::from("Content mapper options must be an object"),
            code: OPTION_CODE_NOT_AN_OBJECT,
        }),
    }
    if let Value::Object(compiler_options) = &params.compiler_options
        && compiler_options.get("noUnusedLocals") == Some(&Value::Bool(true))
    {
        settings.no_unused_locals = true;
    }
    (
        settings,
        OpenProjectResult {
            config_identity: "",
            option_diagnostics: diagnostics,
        },
    )
}
