use super::CrossFileAnalyzer;
use super::diagnostics::{dedupe_diagnostics, sort_diagnostics};
use crate::analyzer::types::{CrossFileResult, CrossFileStats};
use crate::rules;

impl CrossFileAnalyzer {
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
            .then(|| rules::ProvideInjectIndex::new(&self.registry, &self.graph));

        // Run enabled rules
        if self.options.fallthrough_attrs {
            let (info, diags) = rules::analyze_fallthrough(&self.registry, &self.graph);
            result.fallthrough_info = info;
            result.diagnostics.extend(diags);
        }

        if self.options.component_emits {
            let (flows, diags) = rules::analyze_emits(&self.registry, &self.graph);
            result.emit_flows = flows;
            result.diagnostics.extend(diags);
        }

        if self.options.event_bubbling {
            let (bubbles, diags) = rules::analyze_event_bubbling(&self.registry, &self.graph);
            result.event_bubbles = bubbles;
            result.diagnostics.extend(diags);
        }

        if self.options.provide_inject
            && let Some(index) = provide_inject_index.as_ref()
        {
            let (matches, diags) = rules::analyze_provide_inject_with_index(index);
            result.provide_inject_tree = Some(rules::build_provide_inject_tree_with_index(
                &self.registry,
                index,
                &matches,
            ));
            result.provide_inject_matches = matches;
            result.diagnostics.extend(diags);
        }

        if self.options.unique_ids {
            let (issues, diags) = rules::analyze_element_ids(&self.registry);
            result.unique_id_issues = issues;
            result.diagnostics.extend(diags);
        }

        if self.options.server_client_boundary || self.options.error_suspense_boundary {
            let (boundaries, diags) = rules::analyze_boundaries(&self.registry, &self.graph);
            result.boundaries = boundaries;
            result.diagnostics.extend(diags);
        }

        if self.options.reactivity_tracking {
            // Single-file reactivity analysis
            let (issues, diags) = rules::analyze_reactivity(&self.registry, &self.graph);
            result.reactivity_issues = issues;
            result.diagnostics.extend(diags);

            // Cross-file reactivity analysis
            let (cross_issues, cross_diags) =
                rules::analyze_cross_file_reactivity(&self.registry, &self.graph);
            result.cross_file_reactivity_issues = cross_issues;
            result.diagnostics.extend(cross_diags);
        }

        if self.options.race_conditions {
            let (issues, diags) = rules::analyze_race_conditions_with_index(
                &self.registry,
                &self.graph,
                provide_inject_index.as_ref(),
            );
            result.race_condition_issues = issues;
            result.diagnostics.extend(diags);
        }

        if self.options.setup_context {
            // Setup context violation analysis (CSRP/memory leaks)
            let (issues, diags) = rules::analyze_setup_context(&self.registry, &self.graph);
            result.setup_context_issues = issues;
            result.diagnostics.extend(diags);
        }

        // Static validation rules
        if self.options.component_resolution {
            let (issues, diags) = rules::analyze_component_resolution(&self.registry, &self.graph);
            result.component_resolution_issues = issues;
            result.diagnostics.extend(diags);
        }

        if self.options.props_validation {
            let (issues, diags) = rules::analyze_props_validation(&self.registry, &self.graph);
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
}
