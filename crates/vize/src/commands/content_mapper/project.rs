//! Project lifecycle state for the TypeScript content-mapper server.
//!
//! The merged upstream protocol (microsoft/typescript-go#4712) opens one
//! mapper project per TypeScript project before any transform: `openProject`
//! carries the tsconfig entry's mapper options and the project's effective
//! compiler options, and every `transform` then references the opened project
//! by its host-assigned handle. Invalid mapper options are reported as
//! `optionDiagnostics` addressed into the entry's options object instead of
//! failing the request, so TypeScript can surface them in the tsconfig.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use vize_canon::{
    ContentMapperProjectConfiguration, ContentMapperProjectContext, ContentMapperTransformOptions,
    CorsaError,
};
use vize_l0::{FxHashMap, String as CompactString, cstr};

/// Mapper diagnostic code for an options value that is not an object.
const OPTION_CODE_NOT_AN_OBJECT: i32 = 1;
/// Mapper diagnostic code for an option Vize does not recognize.
const OPTION_CODE_UNKNOWN_OPTION: i32 = 2;
/// Mapper diagnostic code for a recognized option holding the wrong type.
const OPTION_CODE_INVALID_TYPE: i32 = 3;

/// Mapper settings resolved when TypeScript opens a project.
#[derive(Clone, Debug)]
pub(super) struct ProjectSettings {
    pub transform_options: ContentMapperTransformOptions,
    pub project_context: ContentMapperProjectContext,
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

    pub fn settings(&self, handle: &str) -> Option<&ProjectSettings> {
        self.projects.get(handle)
    }
}

/// The `openProject` request parameters.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OpenProjectParams {
    /// The absolute project configuration file name, or empty when none exists.
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

/// Dynamic config identity and its complete tsconfig resolution watch closure.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OpenProjectResult {
    config_identity: CompactString,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    watched_files: Vec<CompactString>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    option_diagnostics: Vec<OptionDiagnostic>,
}

/// Resolve one `openProject` request into settings and option diagnostics.
///
/// Invalid options degrade to their defaults so the project stays usable while
/// TypeScript reports the diagnostics against the tsconfig entry.
pub(super) fn resolve_open_project(
    params: &OpenProjectParams,
) -> Result<(ProjectSettings, OpenProjectResult), CorsaError> {
    let empty_compiler_options = serde_json::Map::new();
    let (configuration, _) = ContentMapperProjectConfiguration::load_with_compiler_options(
        Path::new(params.config_file_name.as_str()),
        params
            .compiler_options
            .as_object()
            .unwrap_or(&empty_compiler_options),
    )?;
    let mut options_api = true;
    let mut no_unused_locals = false;
    let mut check_javascript = false;
    let mut diagnostics = Vec::new();
    match &params.options {
        Value::Null => {}
        Value::Object(options) => {
            for (name, value) in options {
                match name.as_str() {
                    "optionsApi" => match value {
                        Value::Bool(enabled) => options_api = *enabled,
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
    if let Value::Object(compiler_options) = &params.compiler_options {
        no_unused_locals = compiler_options.get("noUnusedLocals") == Some(&Value::Bool(true));
        check_javascript = compiler_options.get("checkJs") == Some(&Value::Bool(true));
    }
    let shared_helpers = super::helper_abi::installed_helpers(
        Path::new(params.config_file_name.as_str()),
        &params.compiler_options,
    );
    let mut watched_files = configuration.watched_files;
    watched_files.extend(shared_helpers.watched_files);
    watched_files.sort();
    watched_files.dedup();
    let config_identity = if shared_helpers.fingerprint.is_empty() {
        configuration.config_identity
    } else {
        cstr!(
            "{}:sdk:{}",
            configuration.config_identity,
            shared_helpers.fingerprint
        )
    };
    Ok((
        ProjectSettings {
            project_context: configuration.project_context,
            transform_options: configuration
                .transform_options
                .with_options_api(options_api)
                .with_preserve_unused_diagnostics(no_unused_locals)
                .with_check_javascript(check_javascript)
                .with_external_component_attrs_module(shared_helpers.core_types)
                .with_external_component_type_helpers(shared_helpers.core_types)
                .with_shared_virtual_module(shared_helpers.virtual_module),
        },
        OpenProjectResult {
            config_identity,
            watched_files,
            option_diagnostics: diagnostics,
        },
    ))
}
