//! Cross-file analyzer implementation.

use super::super::analyzers;
use super::super::diagnostics::{CrossFileDiagnostic, DiagnosticSeverity};
use super::super::graph::{DependencyEdge, DependencyGraph, ModuleNode};
use super::super::registry::{FileId, ModuleRegistry};
use super::types::{CrossFileOptions, CrossFileResult, CrossFileStats};
use crate::{Analyzer, AnalyzerOptions, Croquis};
use std::path::{Component, Path, PathBuf};
use vize_carton::{CompactString, FxHashMap, String};

/// Cross-file analyzer for Vue projects.
pub struct CrossFileAnalyzer {
    /// Analysis options.
    options: CrossFileOptions,
    /// Module registry.
    registry: ModuleRegistry,
    /// Dependency graph.
    graph: DependencyGraph,
    /// Single-file analyzer options.
    single_file_options: AnalyzerOptions,
}

impl CrossFileAnalyzer {
    /// Create a new cross-file analyzer.
    pub fn new(options: CrossFileOptions) -> Self {
        Self {
            options,
            registry: ModuleRegistry::new(),
            graph: DependencyGraph::new(),
            single_file_options: AnalyzerOptions::full(),
        }
    }

    /// Create with a project root directory.
    pub fn with_project_root(options: CrossFileOptions, root: impl AsRef<Path>) -> Self {
        Self {
            options,
            registry: ModuleRegistry::with_project_root(root.as_ref()),
            graph: DependencyGraph::new(),
            single_file_options: AnalyzerOptions::full(),
        }
    }

    /// Set single-file analyzer options.
    pub fn set_single_file_options(&mut self, options: AnalyzerOptions) {
        self.single_file_options = options;
    }

    /// Add a file to be analyzed.
    pub fn add_file(&mut self, path: impl AsRef<Path>, source: &str) -> FileId {
        let path = path.as_ref();

        // Analyze the file with single-file analyzer
        let analysis = self.analyze_single_file(source, path);

        // Register in module registry (takes ownership of analysis)
        let (file_id, is_new) = self.registry.register(path, source, analysis);

        if is_new {
            // Add to dependency graph
            let mut node = ModuleNode::new(file_id, path.to_string_lossy().as_ref());

            // Extract component name
            if let Some(entry) = self.registry.get(file_id) {
                node.component_name = entry.component_name.clone();
            }

            // Mark entry points
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename == "App.vue"
                || filename == "main.ts"
                || filename == "main.js"
                || filename == "index.vue"
            {
                node.is_entry = true;
            }

            self.graph.add_node(node);
        }

        self.update_dependency_edges(file_id);

        file_id
    }

    /// Add multiple files.
    pub fn add_files(&mut self, files: &[(&Path, &str)]) {
        for (path, source) in files {
            self.add_file(path, source);
        }
    }

    /// Add a file with pre-computed analysis.
    ///
    /// This is useful when the caller has already performed analysis (e.g., WASM bindings
    /// that parse both script and template content). The analysis should include
    /// `used_components` populated from template analysis for component usage edges.
    pub fn add_file_with_analysis(
        &mut self,
        path: impl AsRef<Path>,
        source: &str,
        analysis: Croquis,
    ) -> FileId {
        let path = path.as_ref();

        // Register in module registry (takes ownership of analysis)
        let (file_id, is_new) = self.registry.register(path, source, analysis);

        if is_new {
            // Add to dependency graph
            let mut node = ModuleNode::new(file_id, path.to_string_lossy().as_ref());

            // Extract component name
            if let Some(entry) = self.registry.get(file_id) {
                node.component_name = entry.component_name.clone();
            }

            // Mark entry points
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename == "App.vue"
                || filename == "main.ts"
                || filename == "main.js"
                || filename == "index.vue"
            {
                node.is_entry = true;
            }

            self.graph.add_node(node);
        }

        self.update_dependency_edges(file_id);

        file_id
    }

    /// Rebuild import and import-backed component usage edges.
    ///
    /// This should be called after all files have been registered when callers
    /// add files in an arbitrary order. A parent may be added before the file
    /// referenced by `./Child.vue`; the first pass cannot resolve that target.
    pub fn rebuild_import_edges(&mut self) {
        let file_ids: Vec<_> = self.registry.iter().map(|entry| entry.id).collect();
        for file_id in file_ids {
            self.update_dependency_edges(file_id);
        }
    }

    /// Rebuild component usage edges.
    ///
    /// This should be called after all files have been added to ensure
    /// that ComponentUsage edges are correctly established. When files
    /// are added one by one, component references might not resolve
    /// if the target component hasn't been added yet.
    pub fn rebuild_component_edges(&mut self) {
        // Collect all used_components from all files
        let component_data: Vec<_> = self
            .registry
            .iter()
            .map(|entry| {
                let components: Vec<_> = entry.analysis.used_components.iter().cloned().collect();
                (entry.id, components)
            })
            .collect();

        // Add ComponentUsage edges for any that were missed
        for (file_id, used_components) in component_data {
            for component in used_components {
                if let Some(target_id) = self.find_component_by_name(component.as_str()) {
                    // add_edge checks for duplicates internally
                    self.graph
                        .add_edge(file_id, target_id, DependencyEdge::ComponentUsage);
                }
            }
        }
    }

    /// Run cross-file analysis.
    pub fn analyze(&mut self) -> CrossFileResult {
        // Note: std::time::Instant is not available in WASM, so we conditionally
        // compile time measurement only for non-WASM targets
        #[cfg(not(target_arch = "wasm32"))]
        let start_time = std::time::Instant::now();

        let mut result = CrossFileResult::default();

        // Detect circular dependencies first
        if self.options.circular_dependencies {
            self.graph.detect_circular_dependencies();
            result.circular_deps = self.graph.circular_dependencies().to_vec();
        }

        let provide_inject_index = self
            .options
            .provide_inject
            .then(|| analyzers::ProvideInjectIndex::new(&self.registry, &self.graph));

        // Run enabled analyzers
        if self.options.fallthrough_attrs {
            let (info, diags) = analyzers::analyze_fallthrough(&self.registry, &self.graph);
            result.fallthrough_info = info;
            result.diagnostics.extend(diags);
        }

        if self.options.component_emits {
            let (flows, diags) = analyzers::analyze_emits(&self.registry, &self.graph);
            result.emit_flows = flows;
            result.diagnostics.extend(diags);
        }

        if self.options.event_bubbling {
            let (bubbles, diags) = analyzers::analyze_event_bubbling(&self.registry, &self.graph);
            result.event_bubbles = bubbles;
            result.diagnostics.extend(diags);
        }

        if self.options.provide_inject {
            let index = provide_inject_index
                .as_ref()
                .expect("provide/inject index should be initialized when enabled");
            let (matches, diags) = analyzers::analyze_provide_inject_with_index(index);
            result.provide_inject_tree = Some(analyzers::build_provide_inject_tree_with_index(
                &self.registry,
                index,
                &matches,
            ));
            result.provide_inject_matches = matches;
            result.diagnostics.extend(diags);
        }

        if self.options.unique_ids {
            let (issues, diags) = analyzers::analyze_element_ids(&self.registry);
            result.unique_id_issues = issues;
            result.diagnostics.extend(diags);
        }

        if self.options.server_client_boundary || self.options.error_suspense_boundary {
            let (boundaries, diags) = analyzers::analyze_boundaries(&self.registry, &self.graph);
            result.boundaries = boundaries;
            result.diagnostics.extend(diags);
        }

        if self.options.reactivity_tracking {
            // Single-file reactivity analysis
            let (issues, diags) = analyzers::analyze_reactivity(&self.registry, &self.graph);
            result.reactivity_issues = issues;
            result.diagnostics.extend(diags);

            // Cross-file reactivity analysis
            let (cross_issues, cross_diags) =
                analyzers::analyze_cross_file_reactivity(&self.registry, &self.graph);
            result.cross_file_reactivity_issues = cross_issues;
            result.diagnostics.extend(cross_diags);
        }

        if self.options.race_conditions {
            let (issues, diags) = analyzers::analyze_race_conditions_with_index(
                &self.registry,
                &self.graph,
                provide_inject_index.as_ref(),
            );
            result.race_condition_issues = issues;
            result.diagnostics.extend(diags);
        }

        if self.options.setup_context {
            // Setup context violation analysis (CSRP/memory leaks)
            let (issues, diags) = analyzers::analyze_setup_context(&self.registry, &self.graph);
            result.setup_context_issues = issues;
            result.diagnostics.extend(diags);
        }

        // Static validation analyzers
        if self.options.component_resolution {
            let (issues, diags) =
                analyzers::analyze_component_resolution(&self.registry, &self.graph);
            result.component_resolution_issues = issues;
            result.diagnostics.extend(diags);
        }

        if self.options.props_validation {
            let (issues, diags) = analyzers::analyze_props_validation(&self.registry, &self.graph);
            result.props_validation_issues = issues;
            result.diagnostics.extend(diags);
        }

        dedupe_diagnostics(&mut result.diagnostics);
        sort_diagnostics(&mut result.diagnostics);

        // Calculate statistics
        let error_count = result.diagnostics.iter().filter(|d| d.is_error()).count();
        let warning_count = result.diagnostics.iter().filter(|d| d.is_warning()).count();

        #[cfg(not(target_arch = "wasm32"))]
        let analysis_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        #[cfg(target_arch = "wasm32")]
        let analysis_time_ms = 0.0; // Time measurement not available in WASM

        result.stats = CrossFileStats {
            files_analyzed: self.registry.len(),
            vue_components: self.registry.vue_components().count(),
            dependency_edges: self.count_edges(),
            error_count,
            warning_count,
            info_count: result.diagnostics.len() - error_count - warning_count,
            analysis_time_ms,
        };

        result
    }

    /// Get the module registry.
    #[inline]
    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    /// Get the dependency graph.
    #[inline]
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Get analysis for a specific file.
    pub fn get_analysis(&self, file_id: FileId) -> Option<&Croquis> {
        self.registry.get(file_id).map(|e| &e.analysis)
    }

    /// Get file path by ID.
    pub fn get_file_path(&self, file_id: FileId) -> Option<&Path> {
        self.registry.get(file_id).map(|e| e.path.as_path())
    }

    /// Clear all data and reset.
    pub fn clear(&mut self) {
        self.registry.clear();
        self.graph = DependencyGraph::new();
    }

    // === Private methods ===

    fn analyze_single_file(&self, source: &str, path: &Path) -> Croquis {
        let mut analyzer = Analyzer::with_options(self.single_file_options);

        // Detect if it's a Vue SFC
        let is_vue = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("vue"));

        if is_vue {
            // For Vue SFC, we need the script content extracted.
            // The caller should pass just the script content, or use
            // the WASM bindings which properly parse SFC.
            // For cross-file analysis, we treat Vue SFC source as script setup.
            analyzer.analyze_script_setup(source);
        } else {
            analyzer.analyze_script_plain(source);
        }

        analyzer.finish()
    }

    fn update_dependency_edges(&mut self, file_id: FileId) {
        let Some(entry) = self.registry.get(file_id) else {
            return;
        };

        let current_dir = entry.path.parent().map(Path::to_path_buf);
        let imports_data: Vec<_> = entry
            .analysis
            .scopes
            .iter()
            .filter_map(|scope| {
                if let crate::scope::ScopeData::ExternalModule(data) = scope.data() {
                    Some((
                        data.source.clone(),
                        data.is_type_only,
                        scope
                            .bindings()
                            .map(|(name, _)| CompactString::new(name))
                            .collect::<Vec<_>>(),
                    ))
                } else {
                    None
                }
            })
            .collect();
        let used_components: Vec<_> = entry.analysis.used_components.iter().cloned().collect();

        let mut import_bindings = Vec::new();
        for (source, is_type_only, local_bindings) in imports_data {
            let Some(target_id) = self.resolve_import(source.as_str(), current_dir.as_deref())
            else {
                continue;
            };

            let edge_type = if is_type_only {
                DependencyEdge::TypeImport
            } else {
                DependencyEdge::Import
            };
            self.graph.add_edge(file_id, target_id, edge_type);

            if !is_type_only {
                for local in local_bindings {
                    import_bindings.push((local, target_id));
                }
            }
        }

        for component in used_components {
            let target_id = import_bindings
                .iter()
                .find(|(local, _)| component_names_match(component.as_str(), local.as_str()))
                .map(|(_, target_id)| *target_id)
                .or_else(|| self.find_component_by_name(component.as_str()));

            if let Some(target_id) = target_id {
                self.graph
                    .add_edge(file_id, target_id, DependencyEdge::ComponentUsage);
            }
        }
    }

    fn resolve_import(&self, specifier: &str, from_dir: Option<&Path>) -> Option<FileId> {
        for candidate in import_candidates(specifier, from_dir) {
            if let Some(entry) = self.registry.get_by_path(&candidate) {
                return Some(entry.id);
            }
        }

        // Fallback for flat in-memory playground projects and non-relative
        // imports where only the filename is meaningful.
        let basename = specifier
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or(specifier);

        let mut vue_basename = String::from(basename);
        vue_basename.push_str(".vue");
        let mut ts_basename = String::from(basename);
        ts_basename.push_str(".ts");

        for entry in self.registry.iter() {
            if entry.filename.as_str() == basename
                || entry.filename.as_str() == vue_basename
                || entry.filename.as_str() == ts_basename
            {
                return Some(entry.id);
            }
        }

        None
    }

    fn find_component_by_name(&self, name: &str) -> Option<FileId> {
        self.graph.find_by_component(name)
    }

    fn count_edges(&self) -> usize {
        self.graph.nodes().map(|n| n.imports.len()).sum()
    }
}

fn dedupe_diagnostics(diagnostics: &mut Vec<CrossFileDiagnostic>) {
    let mut seen: FxHashMap<(&'static str, FileId, u32), usize> = FxHashMap::default();
    let mut deduped: Vec<CrossFileDiagnostic> = Vec::with_capacity(diagnostics.len());

    for diagnostic in diagnostics.drain(..) {
        let key = (
            diagnostic.code(),
            diagnostic.primary_file,
            diagnostic.primary_offset,
        );

        if let Some(index) = seen.get(&key).copied() {
            merge_duplicate_diagnostic(&mut deduped[index], diagnostic);
        } else {
            seen.insert(key, deduped.len());
            deduped.push(diagnostic);
        }
    }

    *diagnostics = deduped;
}

fn merge_duplicate_diagnostic(existing: &mut CrossFileDiagnostic, incoming: CrossFileDiagnostic) {
    if is_more_severe(incoming.severity, existing.severity) {
        existing.severity = incoming.severity;
    }

    if existing.suggestion.is_none() {
        existing.suggestion = incoming.suggestion.clone();
    }

    for related in incoming.related_files {
        if !existing.related_files.iter().any(|entry| entry == &related) {
            existing.related_files.push(related);
        }
    }
}

fn sort_diagnostics(diagnostics: &mut [CrossFileDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.primary_file
            .as_u32()
            .cmp(&right.primary_file.as_u32())
            .then_with(|| left.primary_offset.cmp(&right.primary_offset))
            .then_with(|| severity_order(left.severity).cmp(&severity_order(right.severity)))
            .then_with(|| left.code().cmp(right.code()))
    });
}

fn is_more_severe(candidate: DiagnosticSeverity, current: DiagnosticSeverity) -> bool {
    severity_order(candidate) < severity_order(current)
}

fn severity_order(severity: DiagnosticSeverity) -> u8 {
    severity as u8
}

fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Some(relative) = specifier.strip_prefix("@/") {
        bases.push(PathBuf::from("src").join(relative));
    } else if specifier.starts_with('.') {
        let base = from_dir
            .filter(|dir| !dir.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from(specifier), |dir| dir.join(specifier));
        bases.push(base);
    } else if let Some(stripped) = specifier.strip_prefix('/') {
        bases.push(PathBuf::from(stripped));
        bases.push(PathBuf::from(specifier));
    } else {
        bases.push(PathBuf::from(specifier));
    }

    let mut candidates = Vec::new();
    for base in bases {
        let has_extension = base.extension().is_some();
        candidates.push(normalize_logical_path(base.clone()));

        if !has_extension {
            for suffix in [
                ".vue",
                ".ts",
                ".tsx",
                ".js",
                ".jsx",
                "/index.vue",
                "/index.ts",
                "/index.tsx",
                "/index.js",
                "/index.jsx",
            ] {
                candidates.push(normalize_logical_path(path_with_suffix(&base, suffix)));
            }
        }
    }

    candidates
}

fn path_with_suffix(base: &Path, suffix: &str) -> PathBuf {
    if let Some(index_file) = suffix.strip_prefix('/') {
        base.join(index_file)
    } else {
        let mut value = base.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    }
}

fn normalize_logical_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
        }
    }

    normalized
}

fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

fn to_pascal_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::default(),
            }
        })
        .collect()
}

impl Default for CrossFileAnalyzer {
    fn default() -> Self {
        Self::new(CrossFileOptions::default())
    }
}
