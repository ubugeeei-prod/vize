//! Source-domain-aware module registration.

use std::path::Path;

use vize_carton::CompactString;
use vize_croquis::{Croquis, ScopeData};
use vize_module::ModuleDocument;

use super::{
    FileId, ModuleEntry, ModuleImportFact, ModuleRegistry, extract_component_name, hash_source,
    source_renders_slot,
};
use crate::rules::cross_file_reactivity::store_detection::{
    StoreFactories, collect_store_factories,
};

impl ModuleRegistry {
    /// Register a new file or update an existing one.
    ///
    /// Returns the file ID and whether this was a new entry.
    pub fn register(
        &mut self,
        path: impl AsRef<Path>,
        source: &str,
        analysis: impl Into<vize_atlas::Shared<Croquis>>,
    ) -> (FileId, bool) {
        self.register_inner(path.as_ref(), source, source, analysis.into(), None, None)
    }

    /// Register analyzed SFC script text while deriving template-only facts
    /// from the separately owned template source.
    pub(crate) fn register_with_template_source(
        &mut self,
        path: impl AsRef<Path>,
        script_source: &str,
        template_source: &str,
        analysis: impl Into<vize_atlas::Shared<Croquis>>,
    ) -> (FileId, bool) {
        self.register_inner(
            path.as_ref(),
            script_source,
            template_source,
            analysis.into(),
            None,
            None,
        )
    }

    /// Register an Atlas-backed source with module facts produced by its
    /// frontend. This is the production path; it never rediscovers imports by
    /// parsing source text or by treating Croquis scopes as a module graph.
    pub(crate) fn register_with_module_document(
        &mut self,
        path: impl AsRef<Path>,
        script_source: &str,
        template_source: &str,
        analysis: impl Into<vize_atlas::Shared<Croquis>>,
        modules: &ModuleDocument,
        stores: StoreFactories,
    ) -> (FileId, bool) {
        self.register_inner(
            path.as_ref(),
            script_source,
            template_source,
            analysis.into(),
            Some(modules),
            Some(stores),
        )
    }

    fn register_inner(
        &mut self,
        path: &Path,
        source: &str,
        template_source: &str,
        analysis: vize_atlas::Shared<Croquis>,
        modules: Option<&ModuleDocument>,
        stores: Option<StoreFactories>,
    ) -> (FileId, bool) {
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref root) = self.project_root {
            root.join(path)
        } else {
            path.to_path_buf()
        };
        let source_hash = hash_source(source);
        let module_imports = modules.map_or_else(
            || imports_from_croquis(analysis.as_ref()),
            imports_from_modules,
        );
        let module = modules.cloned();

        if let Some(&existing_id) = self.path_to_id.get(&abs_path) {
            if let Some(entry) = self.entries.get_mut(&existing_id) {
                entry.source_hash = source_hash;
                entry.analysis = analysis;
                entry.pinia_stores = stores
                    .clone()
                    .unwrap_or_else(|| collect_store_factories(source));
                entry.module_imports = module_imports;
                entry.module = module;
                entry.mtime = std::fs::metadata(&abs_path)
                    .ok()
                    .and_then(|metadata| metadata.modified().ok());
            }
            self.set_slot_outlet(existing_id, source_renders_slot(template_source));
            return (existing_id, false);
        }

        let id = FileId::new(self.next_id);
        self.next_id += 1;
        let filename = abs_path
            .file_name()
            .map(|name| CompactString::new(name.to_string_lossy().as_ref()))
            .unwrap_or_else(|| CompactString::new("unknown"));
        let is_vue_sfc = abs_path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("vue"));
        let component_name = is_vue_sfc
            .then(|| extract_component_name(&abs_path))
            .flatten();
        let entry = ModuleEntry {
            id,
            path: abs_path.clone(),
            filename,
            mtime: std::fs::metadata(&abs_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok()),
            analysis,
            source_hash,
            is_vue_sfc,
            component_name,
            pinia_stores: stores.unwrap_or_else(|| collect_store_factories(source)),
            module_imports,
            module,
        };

        self.path_to_id.insert(abs_path, id);
        self.entries.insert(id, entry);
        self.set_slot_outlet(id, source_renders_slot(template_source));
        (id, true)
    }

    #[inline]
    pub(crate) fn renders_slot(&self, id: FileId) -> bool {
        self.slot_outlets.contains(&id)
    }

    fn set_slot_outlet(&mut self, id: FileId, renders_slot: bool) {
        if renders_slot {
            self.slot_outlets.insert(id);
        } else {
            self.slot_outlets.remove(&id);
        }
    }
}

fn imports_from_modules(modules: &ModuleDocument) -> Vec<ModuleImportFact> {
    modules
        .modules
        .iter()
        .flat_map(|module| &module.imports)
        .map(|import| ModuleImportFact {
            source: CompactString::new(import.specifier.as_ref()),
            is_type_only: import.type_only
                || (!import.dynamic
                    && !import.bindings.is_empty()
                    && import.bindings.iter().all(|binding| binding.type_only)),
            local_bindings: import
                .locals
                .iter()
                .map(|local| CompactString::new(local.as_ref()))
                .collect(),
        })
        .collect()
}

fn imports_from_croquis(analysis: &Croquis) -> Vec<ModuleImportFact> {
    analysis
        .scopes
        .iter()
        .filter_map(|scope| match scope.data() {
            ScopeData::ExternalModule(data) => Some(ModuleImportFact {
                source: data.source.clone(),
                is_type_only: data.is_type_only,
                local_bindings: scope
                    .bindings()
                    .map(|(name, _)| CompactString::new(name))
                    .collect(),
            }),
            _ => None,
        })
        .collect()
}
