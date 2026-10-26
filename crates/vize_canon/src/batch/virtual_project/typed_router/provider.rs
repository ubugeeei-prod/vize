//! Project-owned admission for the real experimental route-props declarations.
//! Both packages report `5.4.0`, so only the resolved declarations decide.

mod export_graph;
mod manifest;

use std::path::Path;

use serde_json::{Map, Value};
use vize_carton::String;

use super::super::tsconfig_gen::compiler_inputs::CompilerInputs;
use export_graph::{ExportGraph, Kind};
use manifest::Refusal;

/// Whether `vue-router/experimental`, resolved from `directory` under the
/// selected compiler options, declares the plugin's component generic and
/// route-props helper. Every read is observed so package edits revoke it.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
pub(in crate::batch::virtual_project) fn admits_route_props(
    directory: &Path,
    compiler: &Map<std::string::String, Value>,
    inputs: &CompilerInputs,
) -> bool {
    admission(directory, compiler, inputs).is_ok()
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
fn admission(
    directory: &Path,
    compiler: &Map<std::string::String, Value>,
    inputs: &CompilerInputs,
) -> Result<(), Refusal> {
    let read = |path: &Path| -> Result<Option<String>, Refusal> {
        inputs.observe(path);
        match std::fs::read_to_string(path) {
            Ok(source) => Ok(Some(source.into())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Refusal::new(
                "unreadable-provider",
                vize_carton::cstr!("cannot read {}: {error}", path.display()),
            )),
        }
    };
    let (manifest_path, _) = crate::package_route::installed_package_manifest_with_reader(
        directory,
        "vue-router",
        |path| {
            read(path)
                .ok()
                .flatten()
                .and_then(|source| serde_json::from_str(&source).ok())
        },
        |path| inputs.observe(path),
    )
    .ok_or_else(|| Refusal::new("missing-provider", "vue-router is not installed"))?;
    let package_root = manifest_path
        .parent()
        .ok_or_else(|| Refusal::new("missing-provider", "vue-router has no package root"))?;
    let source = read(&manifest_path)?
        .ok_or_else(|| Refusal::new("missing-provider", "vue-router manifest disappeared"))?;
    let mut target = None;
    for conditions in condition_sets(compiler)? {
        let conditions = conditions.iter().map(String::as_str).collect::<Vec<_>>();
        let winner = manifest::experimental_type_target(&source, &conditions)?;
        if target.as_ref().is_some_and(|previous| *previous != winner) {
            return Err(Refusal::new(
                "inconsistent-type-target",
                "import and require contexts select different declarations",
            ));
        }
        target = Some(winner);
    }
    let entry = super::super::tsconfig_paths::normalize_path_lexically(
        &package_root.join(target.unwrap_or_default().as_str()),
    );
    let mut graph = ExportGraph::new(package_root, read);
    let helper = graph.exported(&entry, "_RoutePropsForFilePath")?;
    let page = graph.exported(&entry, "definePage")?;
    match (helper, page) {
        (Some(helper), Some(page))
            if helper.kind == Kind::Alias
                && helper.parameters.len() == 1
                && helper
                    .parameters
                    .iter()
                    .all(|parameter| parameter.string_constraint)
                && page.kind == Kind::Function
                && page.parameters.len() == 2
                && page
                    .parameters
                    .first()
                    .is_some_and(|parameter| parameter.string_constraint) =>
        {
            Ok(())
        }
        _ => Err(Refusal::new(
            "unsupported-route-props",
            "the resolved declarations lack the two-parameter page macro or props helper",
        )),
    }
}

/// Every package-export condition context the selected resolver can use.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
fn condition_sets(compiler: &Map<std::string::String, Value>) -> Result<Vec<Vec<String>>, Refusal> {
    let custom = match compiler.get("customConditions") {
        None => Vec::new(),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| value.as_str().map(String::from))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| Refusal::new("invalid-custom-conditions", "non-string condition"))?,
        Some(_) => return Err(Refusal::new("invalid-custom-conditions", "not an array")),
    };
    if compiler.get("resolvePackageJsonExports") == Some(&Value::Bool(false)) {
        return Err(Refusal::new(
            "exports-disabled",
            "package exports are disabled",
        ));
    }
    let resolution = compiler
        .get("moduleResolution")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase);
    let contexts: &[&[&str]] = match resolution.as_deref() {
        Some("bundler") => &[&["types", "import"]],
        Some("node16" | "nodenext") => {
            &[&["types", "node", "import"], &["types", "node", "require"]]
        }
        // Implicit and legacy resolution need the native resolver's defaults.
        _ => {
            return Err(Refusal::new(
                "unsupported-resolution",
                "explicit exports resolution required",
            ));
        }
    };
    Ok(contexts
        .iter()
        .map(|context| {
            context
                .iter()
                .map(|condition| String::from(*condition))
                .chain(custom.iter().cloned())
                .collect()
        })
        .collect())
}

#[cfg(test)]
mod tests;
