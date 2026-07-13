//! Import extraction from the cached production-neutral module product.

use vize_carton::{FxHashSet, String, ToCompactString};
use vize_croquis::CroquisSemanticSnapshot;
use vize_module::{ModuleDocument, ModuleImport};

#[derive(Debug, Clone)]
pub(super) struct ImportEdge {
    pub specifier: String,
    pub kind: &'static str,
    pub locals: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct FileAnalysis {
    pub imports: Vec<ImportEdge>,
    pub template_used_ids: FxHashSet<String>,
}

pub(super) fn analyze_script_file(modules: &ModuleDocument) -> FileAnalysis {
    FileAnalysis {
        imports: collect_imports(modules),
        template_used_ids: FxHashSet::default(),
    }
}

pub(super) fn analyze_component_file(
    modules: &ModuleDocument,
    semantics: &CroquisSemanticSnapshot,
) -> FileAnalysis {
    FileAnalysis {
        imports: collect_imports(modules),
        template_used_ids: semantics
            .component_usages
            .iter()
            .map(|usage| usage.name.as_str().to_compact_string())
            .collect(),
    }
}

fn collect_imports(modules: &ModuleDocument) -> Vec<ImportEdge> {
    modules
        .modules
        .iter()
        .flat_map(|module| module.imports.iter())
        .filter(|import| !import.type_only)
        .filter(|import| {
            import.bindings.is_empty() || import.bindings.iter().any(|binding| !binding.type_only)
        })
        .map(import_edge)
        .collect()
}

fn import_edge(import: &ModuleImport) -> ImportEdge {
    ImportEdge {
        specifier: import.specifier.as_ref().to_compact_string(),
        kind: if import.dynamic {
            "dynamic-import"
        } else {
            "import"
        },
        locals: if import.bindings.is_empty() {
            import
                .locals
                .iter()
                .map(|local| local.as_ref().to_compact_string())
                .collect()
        } else {
            import
                .bindings
                .iter()
                .filter(|binding| !binding.type_only)
                .map(|binding| binding.local.as_ref().to_compact_string())
                .collect()
        },
    }
}
