//! Native support for Vue Router's explicitly configured typed-page plugin.
//! The generated declaration owns file-to-route identity; page names are never guessed.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::{VirtualProject, tsconfig_paths::normalize_path_lexically};
use crate::virtual_ts::{TemplateGlobal, VirtualTsOptions};

mod transform;
use transform::apply;
#[cfg(test)]
mod config_tests;

const PLUGIN: &str = "vue-router/volar/sfc-typed-router";

#[derive(Default)]
pub(super) struct Context {
    /// Explicit configuration for the known plugin; arbitrary plugins are not executed.
    pub(super) root: Option<PathBuf>,
    /// Registered files whose generated code actually imports the route helper.
    pub(super) import_files: FxHashSet<PathBuf>,
}

pub(super) struct Generation {
    literal: Option<String>,
    template_options: Option<VirtualTsOptions>,
}

impl Generation {
    pub(super) fn new(
        root: Option<&Path>,
        path: &Path,
        options: &VirtualTsOptions,
        uses_template_route: bool,
    ) -> Self {
        let literal = root.and_then(|root| file_literal(root, path));
        let template_options = literal
            .as_deref()
            .filter(|_| uses_template_route)
            .map(|literal| template_options(options, literal));
        Self {
            literal,
            template_options,
        }
    }

    pub(super) fn options<'a>(&'a self, fallback: &'a VirtualTsOptions) -> &'a VirtualTsOptions {
        self.template_options.as_ref().unwrap_or(fallback)
    }

    pub(super) fn apply(
        &self,
        output: &mut crate::virtual_ts::VirtualTsOutput,
        descriptor: &vize_atelier_sfc::SfcDescriptor,
        script: Option<&str>,
        script_offset: u32,
        split: Option<(usize, usize)>,
    ) -> bool {
        let mut imported = self.template_options.is_some();
        if let Some(literal) = self.literal.as_deref()
            && let Some(setup) = descriptor.script_setup.as_ref()
            && let Some(script) = script
        {
            let source_type = match setup.lang.as_deref().unwrap_or("js") {
                "ts" => oxc_span::SourceType::ts(),
                "tsx" => oxc_span::SourceType::tsx(),
                "jsx" => oxc_span::SourceType::jsx(),
                "js" => oxc_span::SourceType::mjs(),
                _ => return imported,
            };
            let source_offset = |offset: usize| {
                if let Some((synthetic_start, authored_start)) = split
                    && offset >= synthetic_start
                {
                    authored_start + offset - synthetic_start
                } else {
                    script_offset as usize + offset
                }
            };
            imported |= apply(
                output,
                script,
                source_offset,
                setup.loc.start..setup.loc.start + setup.content.len(),
                source_type,
                literal,
            );
        }
        imported
    }
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
pub(super) fn configured_root(
    project: &VirtualProject,
    options: Option<&Map<std::string::String, Value>>,
    tsconfig: Option<&Path>,
) -> Option<PathBuf> {
    let plugin = options?.get("plugins")?.as_array()?.iter().find(|entry| {
        entry.as_str() == Some(PLUGIN) || entry.get("name").and_then(Value::as_str) == Some(PLUGIN)
    })?;
    let explicit_root = plugin.get("options").and_then(|value| value.get("rootDir"));
    let root = if let Some(value) = explicit_root {
        // A malformed path is unsupported; it never enables a guessed transform.
        PathBuf::from(value.as_str()?)
    } else {
        let compiler = project.load_compiler_options(tsconfig).ok()?;
        compiler
            .get("rootDir")
            .map(|value| value.as_str().map(PathBuf::from))
            .unwrap_or_else(|| Some(project.project_root.clone()))?
    };
    Some(normalize_path_lexically(&if root.is_absolute() {
        root
    } else {
        project.project_root.join(root)
    }))
}

pub(super) fn file_literal(root: &Path, path: &Path) -> Option<String> {
    if !root.is_absolute() || !path.is_absolute() {
        return None;
    }
    let root = normalize_path_lexically(root);
    let path = normalize_path_lexically(path);
    let root_parts = root.components().collect::<Vec<_>>();
    let file_parts = path.components().collect::<Vec<_>>();
    if root_parts.first() != file_parts.first() {
        return None;
    }
    let shared = root_parts
        .iter()
        .zip(&file_parts)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in shared..root_parts.len() {
        relative.push("..");
    }
    for component in file_parts.iter().skip(shared) {
        relative.push(component.as_os_str());
    }
    let filename = relative.to_str()?.replace('\\', "/");
    Some(
        serde_json::to_string(&filename)
            .ok()?
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029")
            .to_compact_string(),
    )
}

/// Override the template instance's broad route union only when the actual
/// template references `$route`; the configured provider still owns every name.
pub(super) fn template_options(options: &VirtualTsOptions, literal: &str) -> VirtualTsOptions {
    let mut options = options.clone();
    options
        .template_globals
        .retain(|global| global.name != "$route");
    options.template_globals.push(TemplateGlobal {
        name: "$route".into(),
        type_annotation: cstr!(
            "ReturnType<typeof import('vue-router').useRoute<import('vue-router/auto-routes')._RouteNamesForFilePath<{literal}>>>"
        ),
        default_value: crate::virtual_ts::TYPED_ROUTE_GLOBAL_MARKER.into(),
    });
    options
}

impl VirtualProject {
    /// An implicit generated import can expose globals absent from the authored
    /// dependency scanner. Keep unrelated components in the same native program.
    pub(crate) fn has_typed_router_imports(&self) -> bool {
        !self.typed_router.import_files.is_empty()
    }
}
