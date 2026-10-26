//! Native support for Vue Router's explicitly configured typed-page plugin.
//! The generated declaration owns file-to-route identity; page names are never guessed.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::{VirtualProject, tsconfig_paths::normalize_path_lexically};
use crate::virtual_ts::{TemplateGlobal, VirtualTsOptions};

#[cfg(test)]
mod config_tests;
pub(super) mod provider;
#[cfg(test)]
mod route_props_tests;
mod transform;

const PLUGIN: &str = "vue-router/volar/sfc-typed-router";

#[derive(Default)]
pub(super) struct Context {
    /// Explicit configuration for the known plugin; arbitrary plugins are not executed.
    pub(super) root: Option<PathBuf>,
    /// The project's resolved router declarations admit the route-props projection.
    pub(super) route_props: bool,
    /// Registered files whose generated code actually imports the route helper.
    pub(super) import_files: FxHashSet<PathBuf>,
}

impl Context {
    pub(super) fn config(&self) -> Option<Config<'_>> {
        self.root.as_deref().map(|root| Config {
            root,
            route_props: self.route_props,
        })
    }
}

/// Project-owned typed-page authority shared by every generated SFC.
#[derive(Clone, Copy)]
pub(super) struct Config<'a> {
    pub(super) root: &'a Path,
    pub(super) route_props: bool,
}

impl<'a> Config<'a> {
    /// Hosts without a route-props admission keep the file-only projection.
    pub(super) fn files_only(root: &'a Path) -> Self {
        Self {
            root,
            route_props: false,
        }
    }
}

pub(super) struct Generation {
    literal: Option<String>,
    /// The self-import literal when the project admits route props.
    component: Option<String>,
    template_options: Option<VirtualTsOptions>,
    plan: Option<transform::Plan>,
}

impl Generation {
    pub(super) fn new(
        config: Option<Config<'_>>,
        path: &Path,
        options: &VirtualTsOptions,
        uses_template_route: bool,
    ) -> Self {
        let literal = config.and_then(|config| file_literal(config.root, path));
        let template_options = literal
            .as_deref()
            .filter(|_| uses_template_route)
            .map(|literal| template_options(options, literal));
        let component = config
            .filter(|config| config.route_props && literal.is_some())
            .and_then(|_| path.file_name()?.to_str())
            .and_then(|name| string_literal(&vize_carton::cstr!("./{name}")));
        Self {
            literal,
            component,
            template_options,
            plan: None,
        }
    }

    /// Parse setup once before generation. A `props: true` page's
    /// `defineProps` contract also becomes the public and template contract.
    pub(super) fn prepare(
        &mut self,
        croquis: &mut vize_croquis::Croquis,
        descriptor: &vize_atelier_sfc::SfcDescriptor,
        script: Option<&str>,
        script_offset: u32,
        split: Option<(usize, usize)>,
    ) {
        let (Some(literal), Some(setup), Some(script)) = (
            self.literal.as_deref(),
            descriptor.script_setup.as_ref(),
            script,
        ) else {
            return;
        };
        let Some(source_type) = source_type(setup.lang.as_deref()) else {
            return;
        };
        let setup_range = setup.loc.start..setup.loc.start + setup.content.len();
        self.plan = transform::plan(
            script,
            |offset| source_offset(split, script_offset, offset),
            setup_range,
            source_type,
            self.component.is_some(),
        );
        let route_props = transform::route_props_type(literal);
        for (call, type_argument) in self.plan.iter().flat_map(|plan| &plan.props_calls) {
            let projected = match type_argument {
                Some(span) => script
                    .get(span.start as usize..span.end as usize)
                    .map(|authored| cstr!("<{route_props} & ({authored})>")),
                None => Some(cstr!("<{route_props}>")),
            };
            if let Some(projected) = projected {
                croquis
                    .macros
                    .project_define_props_type_args((call.start, call.end), projected);
            }
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
        diagnostic_ownership: Option<
            &crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector,
        >,
    ) -> bool {
        let mut imported = self.template_options.is_some();
        if let Some(literal) = self.literal.as_deref()
            && let Some(setup) = descriptor.script_setup.as_ref()
            && let Some(script) = script
            && let Some(plan) = self.plan.as_ref()
        {
            imported |= transform::apply(
                output,
                plan,
                script,
                |offset| source_offset(split, script_offset, offset),
                setup.loc.start..setup.loc.start + setup.content.len(),
                (literal, self.component.as_deref()),
                diagnostic_ownership,
            );
        }
        imported
    }
}

fn source_type(lang: Option<&str>) -> Option<oxc_span::SourceType> {
    match lang.unwrap_or("js") {
        "ts" => Some(oxc_span::SourceType::ts()),
        "tsx" => Some(oxc_span::SourceType::tsx()),
        "jsx" => Some(oxc_span::SourceType::jsx()),
        "js" => Some(oxc_span::SourceType::mjs()),
        _ => None,
    }
}

fn source_offset(split: Option<(usize, usize)>, script_offset: u32, offset: usize) -> usize {
    if let Some((synthetic_start, authored_start)) = split
        && offset >= synthetic_start
    {
        authored_start + offset - synthetic_start
    } else {
        script_offset as usize + offset
    }
}

/// Refresh the opt-in and its route-props admission. An admitted editor
/// snapshot owns both; otherwise the live compiler options decide.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
pub(super) fn refresh(
    project: &mut VirtualProject,
    options: Option<&Map<std::string::String, Value>>,
    tsconfig: Option<&Path>,
) {
    project.typed_router.root = configured_root(project, options, tsconfig);
    project.typed_router.route_props = project.typed_router.root.is_some()
        && project.compiler_configuration.as_ref().map_or_else(
            || {
                let Ok(compiler) = project.load_compiler_options(tsconfig) else {
                    return false;
                };
                let directory = tsconfig
                    .and_then(Path::parent)
                    .unwrap_or(&project.project_root);
                provider::admits_route_props(directory, &compiler, &Default::default())
            },
            |snapshot| snapshot.typed_router_route_props(),
        );
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
pub(super) fn configured_root(
    project: &VirtualProject,
    options: Option<&Map<std::string::String, Value>>,
    tsconfig: Option<&Path>,
) -> Option<PathBuf> {
    configured_root_with(&project.project_root, options, || {
        let compiler = project.load_compiler_options(tsconfig).ok()?;
        root_from_compiler_options(&project.project_root, &compiler)
    })
}

/// Resolve the same opt-in from already admitted immutable compiler options.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
pub(in crate::batch::virtual_project) fn configured_root_from_options(
    project_root: &Path,
    options: Option<&Map<std::string::String, Value>>,
    compiler: &Map<std::string::String, Value>,
) -> Option<PathBuf> {
    configured_root_with(project_root, options, || {
        root_from_compiler_options(project_root, compiler)
    })
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
fn root_from_compiler_options(
    project_root: &Path,
    compiler: &Map<std::string::String, Value>,
) -> Option<PathBuf> {
    compiler
        .get("rootDir")
        .map(|value| value.as_str().map(PathBuf::from))
        .unwrap_or_else(|| Some(project_root.to_path_buf()))
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std strings")]
fn configured_root_with(
    project_root: &Path,
    options: Option<&Map<std::string::String, Value>>,
    compiler_root: impl FnOnce() -> Option<PathBuf>,
) -> Option<PathBuf> {
    let plugin = options?.get("plugins")?.as_array()?.iter().find(|entry| {
        entry.as_str() == Some(PLUGIN) || entry.get("name").and_then(Value::as_str) == Some(PLUGIN)
    })?;
    let explicit_root = plugin.get("options").and_then(|value| value.get("rootDir"));
    let root = if let Some(value) = explicit_root {
        // A malformed path is unsupported; it never enables a guessed transform.
        PathBuf::from(value.as_str()?)
    } else {
        compiler_root()?
    };
    Some(normalize_path_lexically(&if root.is_absolute() {
        root
    } else {
        project_root.join(root)
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
    string_literal(&filename)
}

fn string_literal(value: &str) -> Option<String> {
    Some(
        serde_json::to_string(value)
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
