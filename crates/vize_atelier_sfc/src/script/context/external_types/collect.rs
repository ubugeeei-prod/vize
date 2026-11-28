//! Collect the types a script imports into its compile context.

#![expect(
    clippy::disallowed_types,
    reason = "the components of a batch share one walk's declarations"
)]

use std::path::Path;
use std::sync::Arc;

use vize_carton::String;

use super::super::batch_epoch::{NO_EPOCH, current_batch_epoch};
use super::bindings::{self, Bound, ImportBinding};
use super::declarations::Declarations;
use super::modules::{self, ModuleGraph, ModuleId};
use super::resolution::{canonical_base_file, path_key, project::Project};
use super::walk::{RequestedExports, TypeDependency, Walk};
use super::work::{Step, count};
use super::{FileTypeSummary, ImportedTypes, ScriptCompileContext, extract_script_summary_binding};

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
        if let Ok(imported) =
            self.try_collect_imported_types_from_path(source, filename, is_ts, None)
        {
            self.merge_bindings(&imported.bindings);
            self.merge_declarations(&imported.declarations);
        }
    }

    /// Resolve the script's type-bearing imports. What a walk of the batch's
    /// shared graph declares is returned in `declarations`, unmerged: the
    /// caller merges it once every script of the component is walked.
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
        let mut bindings = Vec::new();
        extract_script_summary_binding(source, &mut root, false, &mut bindings);
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

        let walked = self.walk_imported_declarations(
            &root.dependencies,
            &bindings,
            base_file,
            project,
            modules::shared(project, current_batch_epoch()).as_deref(),
        );
        let mut dependencies = walked.files;
        let mut imports = root.imports;
        imports.extend(walked.imports);
        // The walked names are sorted and distinct; the configuration inputs
        // are few. Sort those alone and merge them in.
        let mut inputs: Vec<String> = Project::discovery_inputs(base_file)?
            .iter()
            .map(|path| path_key(path))
            .collect();
        if discovered.is_some() {
            inputs.extend(
                Project::reference_inputs(base_file)?
                    .iter()
                    .map(|path| path_key(path)),
            );
        }
        if let Some(project) = project {
            inputs.extend(project.inputs().iter().map(|path| path_key(path)));
        }
        inputs.sort();
        inputs.dedup();
        dependencies = sorted_union(dependencies, inputs);
        Ok(ImportedTypes {
            imports,
            dependencies,
            declarations: walked.declarations,
            bindings: walked.bound,
        })
    }

    /// [`Self::walk_imported_declarations`], merging every declaration.
    #[cfg(test)]
    pub(super) fn walk_imported_modules(
        &mut self,
        roots: &[TypeDependency],
        base_file: &Path,
        project: Option<&Project>,
        shared: Option<&ModuleGraph>,
    ) -> (Vec<(String, String)>, Vec<String>) {
        let walked = self.walk_imported_declarations(roots, &[], base_file, project, shared);
        self.merge_declarations(&walked.declarations);
        (walked.imports, walked.files)
    }

    /// Walk the declarations reachable through `roots`, the type-bearing
    /// imports of the script in `base_file`, and return the imports and the
    /// sorted dependency names of the walked modules. Each of `bindings`, the
    /// script's named imports, is resolved to the export it names.
    ///
    /// In a batch, `shared` is the graph of the batch's stability window. Each
    /// root import then contributes the closure of the module it resolves to,
    /// which the graph keeps for all components, and the closure's
    /// declarations by name are returned for the caller to merge in root
    /// order, after the bound ones. One walk over all roots visits the same
    /// modules: a module an earlier root already walked is skipped together
    /// with everything it reaches, and what is left keeps its order, so every
    /// name keeps the same first declaration.
    ///
    /// Outside a batch nothing may be reused across compiles, so one walk over
    /// a graph of its own merges the bound declarations and then every walked
    /// module, and returns no declarations.
    pub(super) fn walk_imported_declarations(
        &mut self,
        roots: &[TypeDependency],
        bindings: &[ImportBinding],
        base_file: &Path,
        project: Option<&Project>,
        shared: Option<&ModuleGraph>,
    ) -> Walked {
        let all = RequestedExports::All;
        let requests = roots.iter().enumerate().filter_map(|(index, dependency)| {
            count(Step::Edge, 1);
            let requested = dependency.request(&all)?.into_owned();
            Some((index, dependency.specifier(), requested))
        });
        let name = |graph: &ModuleGraph, id: &ModuleId| Some(graph.module(*id)?.key.clone());
        let bound_by = |index: usize| {
            bindings
                .iter()
                .filter(move |binding| binding.dependency == index)
        };
        let Some(graph) = shared else {
            let graph = ModuleGraph::new(NO_EPOCH);
            let mut modules = Vec::new();
            let mut entries = Vec::new();
            let mut walk = Walk::new(&graph, project, &mut modules);
            for (index, specifier, requested) in requests {
                if let Some(module) = graph.resolve(base_file, specifier, project) {
                    walk.enter(module, &requested);
                    entries.push((index, module));
                }
            }
            let (files, _) = walk.finish();
            for (index, module) in entries {
                for binding in bound_by(index) {
                    let declared = bindings::resolve(&graph, module, &binding.imported, project)
                        .and_then(|export| Some((export.interface, export.declaration(&graph)?)));
                    if let Some((interface, (_, body))) = declared {
                        self.merge_body(&binding.local, interface, body);
                    }
                }
            }
            let mut imports = Vec::new();
            for summary in modules.iter().filter_map(|id| graph.summary(*id)) {
                self.merge_file_summary(summary);
                imports.extend(summary.imports.iter().cloned());
            }
            count(Step::Dependency, files.len());
            let files = files.iter().filter_map(|id| name(&graph, id)).collect();
            return Walked {
                imports,
                files,
                ..Walked::default()
            };
        };
        let mut walked = Walked::default();
        for (index, specifier, requested) in requests {
            let Some(closure) = graph
                .entry(base_file, specifier, project)
                .and_then(|module| graph.closure(module, &requested, project))
            else {
                continue;
            };
            let declarations = closure.declarations(graph);
            walked.imports.extend(declarations.imports.iter().cloned());
            for binding in bound_by(index) {
                if let Some(export) = declarations.export(&binding.imported) {
                    walked.bound.push(Bound {
                        local: binding.local.clone(),
                        walk: Arc::clone(declarations),
                        export,
                    });
                }
            }
            walked.declarations.push(Arc::clone(declarations));
            count(Step::Dependency, closure.files.len());
            // One closure's names are already sorted and distinct.
            let files = closure.files.iter().filter_map(|id| name(graph, id));
            walked.files = sorted_union(std::mem::take(&mut walked.files), files.collect());
        }
        walked
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

/// What one script's walks found and did not merge yet.
#[derive(Default)]
pub(super) struct Walked {
    /// The imports of the walked modules, in merge order.
    pub(super) imports: Vec<(String, String)>,
    /// The sorted dependency names.
    pub(super) files: Vec<String>,
    /// The declarations of each walk, by name.
    pub(super) declarations: Vec<Arc<Declarations>>,
    /// The declarations the script's named imports bind.
    pub(super) bound: Vec<Bound>,
}

/// The sorted, distinct names of two sorted lists, moved rather than copied.
fn sorted_union(left: Vec<String>, right: Vec<String>) -> Vec<String> {
    if right.is_empty() {
        return left;
    }
    if left.is_empty() {
        return right;
    }
    let mut union: Vec<String> = Vec::with_capacity(left.len() + right.len());
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();
    while let (Some(l), Some(r)) = (left.peek(), right.peek()) {
        match l.cmp(r) {
            std::cmp::Ordering::Less => union.extend(left.next()),
            std::cmp::Ordering::Greater => union.extend(right.next()),
            std::cmp::Ordering::Equal => {
                union.extend(left.next());
                right.next();
            }
        }
    }
    // Each input is already distinct. Once one is exhausted, the remaining
    // names need no comparisons or deduplication (the usual case when a few
    // config inputs precede a wide barrel's source paths).
    union.extend(left);
    union.extend(right);
    union
}

#[cfg(test)]
mod tests {
    use super::sorted_union;
    use vize_carton::String;

    #[test]
    fn dependency_union_preserves_order_and_removes_overlapping_names() {
        let names = |names: &[&str]| names.iter().copied().map(String::from).collect();
        for (left, right, expected) in [
            (&[][..], &[][..], &[][..]),
            (&["a", "b"][..], &[][..], &["a", "b"][..]),
            (&[][..], &["a", "b"][..], &["a", "b"][..]),
            (&["a", "c"][..], &["b", "d"][..], &["a", "b", "c", "d"][..]),
            (&["a", "c"][..], &["a", "b", "c"][..], &["a", "b", "c"][..]),
            (&["b", "c", "d"][..], &["a"][..], &["a", "b", "c", "d"][..]),
        ] {
            assert_eq!(sorted_union(names(left), names(right)), names(expected));
        }
    }
}
