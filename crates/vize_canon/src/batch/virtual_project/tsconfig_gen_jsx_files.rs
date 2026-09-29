use serde_json::{Map, Value};

use super::VirtualProject;

pub(super) fn needs_vue_jsx_compiler_options(project: &VirtualProject) -> bool {
    project.virtual_files.values().any(|file| {
        file.virtual_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with(".vue.tsx") || name.ends_with(".tsx.ts") || name.ends_with(".jsx.ts")
            })
    })
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(super) fn compiler_option_enabled(
    options: &Map<std::string::String, Value>,
    name: &str,
) -> bool {
    options.get(name).and_then(Value::as_bool).unwrap_or(false)
}
