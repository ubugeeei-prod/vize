//! One-snapshot source scan for Nuxt fallback module imports.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcDescriptorArtifact, SfcDescriptorProduct};
use vize_atlas::{Compilation, CompilationSnapshot, QueryOutcome, SourceId};
use vize_carton::{FxHashMap, String, ToCompactString};

use super::ModuleImports;
use crate::commands::check::nuxt::parsing::{
    is_ts_identifier, source_type_for_path, source_type_for_script_lang,
};
use crate::commands::check::nuxt::stubs::tracked_read_to_string;

struct ScanSource {
    path: PathBuf,
    text: String,
    sfc: Option<SourceId>,
}

struct SourceScanGraph {
    snapshot: CompilationSnapshot,
    sources: Vec<ScanSource>,
}

impl SourceScanGraph {
    fn discover(cwd: &Path) -> Self {
        let entries = nuxt_source_roots(cwd)
            .into_iter()
            .flat_map(|root| {
                WalkBuilder::new(root)
                    .hidden(false)
                    .standard_filters(true)
                    .build()
                    .flatten()
                    .filter_map(|entry| {
                        let path = entry.into_path();
                        if !path.is_file() || !is_import_scan_source(&path) {
                            return None;
                        }
                        tracked_read_to_string(&path)
                            .ok()
                            .map(|text| (path, text.to_compact_string()))
                    })
            })
            .collect();
        Self::new(entries)
    }

    fn new(entries: Vec<(PathBuf, String)>) -> Self {
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .expect("Nuxt scan registers SFC providers once");
        let mut sources = Vec::with_capacity(entries.len());
        for (path, text) in entries {
            let sfc = is_vue_source(&path).then(|| {
                compilation
                    .add_source(path.to_string_lossy().as_ref(), text.as_str())
                    .expect("Nuxt scan source identity is valid")
            });
            sources.push(ScanSource { path, text, sfc });
        }
        Self {
            snapshot: compilation.snapshot(),
            sources,
        }
    }

    fn descriptor(
        &self,
        source: SourceId,
    ) -> Result<QueryOutcome<SfcDescriptorProduct>, vize_atlas::QueryError> {
        self.snapshot
            .query_session()
            .query::<SfcDescriptorProduct>(source)
    }

    fn collect(&self) -> FxHashMap<String, ModuleImports> {
        let mut imports = FxHashMap::default();
        for source in &self.sources {
            if let Some(source_id) = source.sfc {
                let Ok(descriptor) = self.descriptor(source_id) else {
                    continue;
                };
                collect_descriptor(descriptor.value(), &mut imports);
            } else {
                collect_script(
                    source.text.as_str(),
                    source_type_for_path(&source.path),
                    &mut imports,
                );
            }
        }
        imports
    }
}

pub(super) fn collect_nuxt_virtual_module_imports(cwd: &Path) -> FxHashMap<String, ModuleImports> {
    SourceScanGraph::discover(cwd).collect()
}

fn collect_descriptor(
    artifact: &SfcDescriptorArtifact,
    imports: &mut FxHashMap<String, ModuleImports>,
) {
    let Some(descriptor) = artifact.descriptor() else {
        return;
    };
    for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        collect_script(
            script.content.as_ref(),
            source_type_for_script_lang(script.lang.as_deref()),
            imports,
        );
    }
}

fn collect_script(
    source: &str,
    source_type: SourceType,
    imports: &mut FxHashMap<String, ModuleImports>,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    for statement in &ret.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let module_name = import.source.value.as_str();
        if !is_nuxt_fallback_module(module_name) {
            continue;
        }
        let entry = imports.entry(module_name.into()).or_default();
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let imported = specifier.imported.name().as_str();
                    if is_ts_identifier(imported) {
                        entry.named.insert(imported.into());
                    }
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                    entry.has_default = true;
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
    }
}

fn nuxt_source_roots(cwd: &Path) -> Vec<PathBuf> {
    [
        "app",
        "pages",
        "components",
        "composables",
        "layouts",
        "middleware",
        "plugins",
        "server",
        "shared",
        "utils",
        "modules",
        "i18n",
    ]
    .into_iter()
    .map(|directory| cwd.join(directory))
    .filter(|path| path.is_dir())
    .collect()
}

fn is_import_scan_source(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        name.rsplit_once('.').map(|(_, extension)| extension),
        Some("vue" | "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
    )
}

fn is_vue_source(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("vue")
}

fn is_nuxt_fallback_module(module_name: &str) -> bool {
    matches!(
        module_name,
        "#imports" | "#components" | "#app" | "@typed-router"
    )
}

#[cfg(test)]
#[path = "source_scan/tests.rs"]
mod tests;
