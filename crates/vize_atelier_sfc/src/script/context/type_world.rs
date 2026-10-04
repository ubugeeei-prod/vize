//! Produce the scoped declaration world used by public Alpha summaries.

#![expect(
    clippy::disallowed_types,
    reason = "source snapshots lend shared immutable Arc module text to the parser"
)]

#[cfg(test)]
mod overlay_tests;
mod parse;
#[cfg(test)]
mod scoped_props_tests;
#[cfg(test)]
mod snapshot_cache_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod with_defaults_tests;

use oxc_span::SourceType;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vize_carton::{FxHashSet, cstr};
use vize_croquis::types::ResolvedTypeWorld;
use vize_croquis::types::world::{TypeExportBinding, TypeModule, TypeModuleReference};

use super::external_types::resolution::path_key;
use super::source_snapshot::source_path;
use super::{ScriptCompileContext, TypeSourceSnapshot};

const MAX_MODULES: usize = 512;
type ModuleSource = (Arc<str>, bool);
type PendingModule = (PathBuf, Option<ModuleSource>);
pub(super) type ParsedTypeModule = (TypeModule, FxHashSet<vize_carton::CompactString>);

impl ScriptCompileContext {
    /// Retain real module/export identities separately from the compatibility
    /// resolver's flat name maps. The root is the current in-memory script.
    pub fn resolve_type_world(
        &self,
        filename: &str,
        normal_script: Option<&str>,
    ) -> ResolvedTypeWorld {
        self.resolve_type_world_with_syntax(filename, normal_script, filename.ends_with(".tsx"))
    }

    pub fn resolve_type_world_with_syntax(
        &self,
        filename: &str,
        normal_script: Option<&str>,
        is_tsx: bool,
    ) -> ResolvedTypeWorld {
        self.resolve_type_world_with_sources(
            filename,
            normal_script,
            is_tsx,
            &TypeSourceSnapshot::default(),
        )
    }

    pub fn resolve_type_world_with_sources(
        &self,
        filename: &str,
        normal_script: Option<&str>,
        is_tsx: bool,
        sources: &TypeSourceSnapshot,
    ) -> ResolvedTypeWorld {
        let path = source_path(Path::new(filename));
        let root_module = path_key(&path);
        let source = normal_script.map_or_else(
            || self.source.clone(),
            |normal| cstr!("{normal}\n{}", self.source),
        );
        let mut world = ResolvedTypeWorld {
            root_module: root_module.clone(),
            ..ResolvedTypeWorld::default()
        };
        let mut pending = vec![(path, Some((Arc::<str>::from(source.as_str()), is_tsx)))];
        let mut visited = FxHashSet::default();
        while let Some((path, source)) = pending.pop() {
            let identity = path_key(&path);
            if !visited.insert(identity.clone()) {
                continue;
            }
            if world.modules.len() >= MAX_MODULES {
                world.modules.insert(identity, TypeModule::default());
                continue;
            }
            // Root text belongs to this caller, including unsaved buffers and
            // combined normal/setup scripts. Never reuse it by path. Only
            // immutable snapshot dependencies share their unresolved parse.
            let parsed = source.map_or_else(
                || read_dependency_module(&path, sources),
                |(source, is_tsx)| Some(parse_source(&source, is_tsx)),
            );
            let Some((mut module, needed_imports)) = parsed else {
                world.modules.insert(identity, TypeModule::default());
                continue;
            };
            for (local, import) in &mut module.imports {
                resolve_target(
                    &mut import.target,
                    &path,
                    &mut pending,
                    needed_imports.contains(local),
                    sources,
                );
            }
            for export in module.exports.values_mut() {
                match export {
                    TypeExportBinding::Forward { target, .. }
                    | TypeExportBinding::Namespace(target) => {
                        resolve_target(target, &path, &mut pending, true, sources);
                    }
                    TypeExportBinding::Local(local) => {
                        if let Some(import) = module.imports.get(local)
                            && let Some(target) = &import.target.module
                        {
                            pending.push((PathBuf::from(target.as_str()), None));
                        }
                    }
                }
            }
            for target in &mut module.star_exports {
                resolve_target(target, &path, &mut pending, true, sources);
            }
            for target in module.direct_imports.values_mut() {
                resolve_target(target, &path, &mut pending, true, sources);
            }
            world.modules.insert(identity, module);
        }
        world
    }
}

fn parse_source(source: &str, is_tsx: bool) -> ParsedTypeModule {
    let source_type = if is_tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    parse::parse_module(source, source_type)
}

fn read_dependency_module(path: &Path, sources: &TypeSourceSnapshot) -> Option<ParsedTypeModule> {
    // Single-flight per dependency, without holding the map lock while an
    // unrelated dependency is read or parsed by another publication.
    let entry = sources
        .modules
        .lock()
        .ok()?
        .entry(path.to_path_buf())
        .or_default()
        .clone();
    entry
        .get_or_init(|| {
            read_module_source(path, sources).map(|(source, is_tsx)| parse_source(&source, is_tsx))
        })
        .clone()
}

fn resolve_target(
    target: &mut TypeModuleReference,
    current: &Path,
    pending: &mut Vec<PendingModule>,
    follow: bool,
    sources: &TypeSourceSnapshot,
) {
    if let Some(path) = sources.resolve_import_normalized(current, &target.specifier) {
        target.module = Some(path_key(&path));
        if follow {
            pending.push((path, None));
        }
    }
}

fn read_module_source(path: &Path, sources: &TypeSourceSnapshot) -> Option<ModuleSource> {
    let source = sources.read(path)?;
    if path.extension().is_some_and(|ext| ext == "vue") {
        let descriptor = crate::parse_sfc(&source, crate::SfcParseOptions::default()).ok()?;
        let is_tsx = descriptor
            .script
            .as_ref()
            .is_some_and(|script| script.lang.as_deref() == Some("tsx"))
            || descriptor
                .script_setup
                .as_ref()
                .is_some_and(|script| script.lang.as_deref() == Some("tsx"));
        return Some((
            Arc::from(
                cstr!(
                    "{}\n{}",
                    descriptor
                        .script
                        .as_ref()
                        .map_or("", |script| script.content.as_ref()),
                    descriptor
                        .script_setup
                        .as_ref()
                        .map_or("", |script| script.content.as_ref()),
                )
                .as_str(),
            ),
            is_tsx,
        ));
    }
    Some((
        source,
        path.extension()
            .is_some_and(|ext| ext == "tsx" || ext == "jsx"),
    ))
}
