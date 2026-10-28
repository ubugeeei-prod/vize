//! Cross-file analyzer state and module wiring.

mod accessors;
mod constructors;
mod deps;
mod diagnostics;
mod files;
mod nuxt_render;
mod run;
mod single_file;

use super::types::{CrossFileOptions, NuxtRenderDirs};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::FxHashMap;
use vize_croquis::{AnalyzerOptions, EffectGraphSummary};

/// Cross-file analyzer for Vue projects.
pub struct CrossFileAnalyzer {
    /// Analysis options.
    options: CrossFileOptions,
    /// Host project capability, separate from enabling the boundary rule group.
    ssr_enabled: bool,
    /// Nuxt directories from the host config; `None` infers them beside `app.vue`.
    nuxt_dirs: Option<NuxtRenderDirs>,
    /// Host authority for exactly one completed source/inventory invocation.
    nuxt_scope_once: bool,
    /// Module registry.
    registry: ModuleRegistry,
    /// Dependency graph.
    graph: DependencyGraph,
    /// Parser-built local reactive effect summaries by file.
    effect_graph_summaries: FxHashMap<FileId, EffectGraphSummary>,
    /// Own template complexity facts of each SFC (L2 `template-complexity`).
    pub(crate) template_complexity: crate::rules::TemplateFacts,
    /// Single-file analyzer options.
    single_file_options: AnalyzerOptions,
}

impl Default for CrossFileAnalyzer {
    fn default() -> Self {
        Self::new(CrossFileOptions::default())
    }
}
