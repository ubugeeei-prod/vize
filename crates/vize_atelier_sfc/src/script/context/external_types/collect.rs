//! Collect the types a script imports into its compile context.

use std::path::Path;

use vize_carton::String;

use super::super::batch_epoch::{NO_EPOCH, current_batch_epoch};
use super::modules::{self, ModuleGraph, ModuleId};
use super::resolution::{canonical_base_file, path_key, project::Project};
use super::walk::{RequestedExports, TypeDependency, Visit, Walk};
use super::work::{Step, count};
use super::{FileTypeSummary, ImportedTypes, ScriptCompileContext, extract_script_summary};

impl ScriptCompileContext {
    /// Walk the script's type-bearing imports/re-exports on disk and merge the
    /// interfaces/type aliases they declare into this context.
    ///
    /// `is_ts` must reflect whether the script block is TypeScript
    /// (`lang="ts"`/`"tsx"`, computed once per compile at the call site) — it
    /// is the real signal, derived from the parsed SFC, that replaced the old
    /// `source.contains("type")` substring pre-check. Imported *types* can only
    /// be referenced from TypeScript (`defineProps<Props>()`), so for plain JS
    /// the walk would only burn stat/realpath syscalls; the substring heuristic
    /// misfired on JS object keys like `type: 'text'` next to any `import`,
    /// which is exactly what the `is_ts` gate cuts off.
    pub fn collect_imported_types_from_path(&mut self, source: &str, filename: &str, is_ts: bool) {
        let _ = self.try_collect_imported_types_from_path(source, filename, is_ts, None);
    }

    pub(crate) fn try_collect_imported_types_from_path(
        &mut self,
        source: &str,
        filename: &str,
        is_ts: bool,
        selected: Option<&Project>,
    ) -> Result<ImportedTypes, String> {
        if !is_ts {
            return Ok(ImportedTypes::default());
        }

        // The root source lives in memory (possibly unsaved editor state), so
        // parse it directly; only files read from disk go through the cache.
        // The parsed specifier list is the precise "is there anything to
        // follow?" signal — strictly tighter than the old substring guard, so
        // no separate text pre-check is needed.
        let mut root = FileTypeSummary::default();
        extract_script_summary(source, &mut root, false);
        if root.specifiers.is_empty() {
            // Nothing to resolve — skip base-file canonicalization entirely
            // (the common case: scripts with only runtime imports).
            return Ok(ImportedTypes::default());
        }

        let owned_base = canonical_base_file(filename);
        let base_file = owned_base.as_path();
        let Some(base_dir) = base_file.parent() else {
            return Ok(ImportedTypes::default());
        };
        if base_dir.as_os_str().is_empty() {
            return Ok(ImportedTypes::default());
        }

        let discovered = if selected.is_none() {
            super::resolution::project::for_file(base_file)?
        } else {
            None
        };
        let project = selected
            .filter(|project| project.is_configured())
            .or(discovered.as_deref());

        let (walked, mut dependencies) = self.walk_imported_modules(
            &root.dependencies,
            base_file,
            project,
            modules::shared(project, current_batch_epoch()).as_deref(),
        );
        let mut imports = root.imports;
        imports.extend(walked);
        dependencies.extend(
            Project::discovery_inputs(base_file)?
                .iter()
                .map(|path| path_key(path)),
        );
        if discovered.is_some() {
            dependencies.extend(
                Project::reference_inputs(base_file)?
                    .iter()
                    .map(|path| path_key(path)),
            );
        }
        if let Some(project) = project {
            dependencies.extend(project.inputs().iter().map(|path| path_key(path)));
        }
        dependencies.sort();
        dependencies.dedup();
        Ok(ImportedTypes {
            imports,
            dependencies,
        })
    }

    /// Merge the declarations reachable through `roots`, the type-bearing
    /// imports of the script in `base_file`, and return the imports and the
    /// sorted dependency names of the walked modules.
    ///
    /// In a batch, `shared` is the graph of the batch's stability window. Each
    /// root import then contributes the closure of the module it resolves to,
    /// which the graph computes once for all components, and this component
    /// merges the closure's modules in their walk order. One walk over all
    /// roots visits the same modules: a module an earlier root already walked
    /// is skipped together with everything it reaches, and what is left keeps
    /// its order, so every name keeps the same first declaration.
    ///
    /// Outside a batch nothing may be reused across compiles, so one walk over
    /// a graph of its own merges as it goes.
    pub(super) fn walk_imported_modules(
        &mut self,
        roots: &[TypeDependency],
        base_file: &Path,
        project: Option<&Project>,
        shared: Option<&ModuleGraph>,
    ) -> (Vec<(String, String)>, Vec<String>) {
        let all = RequestedExports::All;
        let requests = roots.iter().filter_map(|dependency| {
            count(Step::Edge, 1);
            let requested = dependency.request(&all)?.into_owned();
            Some((dependency.specifier(), requested))
        });
        let mut merge = Merge {
            context: self,
            imports: Vec::new(),
        };
        let name = |graph: &ModuleGraph, id: &ModuleId| Some(graph.module(*id)?.key.clone());
        let Some(graph) = shared else {
            let graph = ModuleGraph::new(NO_EPOCH);
            let mut walk = Walk::new(&graph, project, &mut merge);
            for (specifier, requested) in requests {
                if let Some(module) = graph.resolve(base_file, specifier, project) {
                    walk.enter(module, &requested);
                }
            }
            let (files, _) = walk.finish();
            count(Step::Dependency, files.len());
            let files = files.iter().filter_map(|id| name(&graph, id)).collect();
            return (merge.imports, files);
        };
        let mut files: Vec<String> = Vec::new();
        for (specifier, requested) in requests {
            let Some(closure) = graph
                .entry(base_file, specifier, project)
                .and_then(|module| graph.closure(module, &requested, project))
            else {
                continue;
            };
            for id in &closure.modules {
                if let Some(summary) = graph.summary(*id) {
                    merge.module(*id, summary);
                }
            }
            count(Step::Dependency, closure.files.len());
            // One closure's names are already sorted and distinct.
            let joins = !files.is_empty();
            files.extend(closure.files.iter().filter_map(|id| name(graph, id)));
            if joins {
                files.sort();
                files.dedup();
            }
        }
        (merge.imports, files)
    }

    pub(super) fn merge_file_summary(&mut self, summary: &FileTypeSummary) {
        count(
            Step::Declaration,
            summary.interfaces.len() + summary.type_aliases.len(),
        );
        // A name already declared in this SFC, in either map, is that
        // binding. An imported `interface Props` must not shadow a local
        // `type Props` (#12942).
        for (name, body) in &summary.interfaces {
            if self.type_aliases.contains_key(name) {
                continue;
            }
            self.interfaces
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
        for (name, body) in &summary.type_aliases {
            if self.interfaces.contains_key(name) {
                continue;
            }
            self.type_aliases
                .entry(name.clone())
                .or_insert_with(|| body.clone());
        }
    }
}

/// Merges each visited module into a component's context.
struct Merge<'a> {
    context: &'a mut ScriptCompileContext,
    imports: Vec<(String, String)>,
}

impl Visit for Merge<'_> {
    fn module(&mut self, _id: ModuleId, summary: &FileTypeSummary) {
        self.context.merge_file_summary(summary);
        self.imports.extend(summary.imports.iter().cloned());
    }
}
