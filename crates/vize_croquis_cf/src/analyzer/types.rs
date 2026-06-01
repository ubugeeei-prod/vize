//! Types for cross-file analysis.

use super::super::analyzers;
use super::super::diagnostics::CrossFileDiagnostic;
use super::super::registry::FileId;

/// Options for cross-file analysis (opt-in features).
#[derive(Debug, Clone, Default)]
pub struct CrossFileOptions {
    /// Analyze fallthrough attributes.
    pub fallthrough_attrs: bool,
    /// Analyze component emits.
    pub component_emits: bool,
    /// Analyze event bubbling.
    pub event_bubbling: bool,
    /// Analyze provide/inject.
    pub provide_inject: bool,
    /// Analyze unique element IDs.
    pub unique_ids: bool,
    /// Analyze server/client boundaries.
    pub server_client_boundary: bool,
    /// Analyze error and suspense boundaries.
    pub error_suspense_boundary: bool,
    /// Analyze reactivity loss.
    pub reactivity_tracking: bool,
    /// Analyze async reactive race-condition risks.
    pub race_conditions: bool,
    /// Analyze setup context violations (CSRP/memory leaks).
    pub setup_context: bool,
    /// Detect circular dependencies.
    pub circular_dependencies: bool,
    /// Maximum depth for dependency chain warnings.
    pub max_import_depth: Option<usize>,

    // === Static validation (strict mode) ===
    /// Check for unregistered components in templates.
    pub component_resolution: bool,
    /// Validate props passed to child components.
    pub props_validation: bool,
}

impl CrossFileOptions {
    /// Create options with all features enabled.
    pub fn all() -> Self {
        Self {
            fallthrough_attrs: true,
            component_emits: true,
            event_bubbling: true,
            provide_inject: true,
            unique_ids: true,
            server_client_boundary: true,
            error_suspense_boundary: true,
            reactivity_tracking: true,
            race_conditions: true,
            setup_context: true,
            circular_dependencies: true,
            max_import_depth: Some(10),
            component_resolution: true,
            props_validation: true,
        }
    }

    /// Create options for strict static validation (compile errors for invalid Vue).
    pub fn strict() -> Self {
        Self {
            component_resolution: true,
            props_validation: true,
            circular_dependencies: true,
            ..Default::default()
        }
    }

    /// Create minimal options (fastest).
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Enable fallthrough attribute analysis.
    pub fn with_fallthrough_attrs(mut self, enabled: bool) -> Self {
        self.fallthrough_attrs = enabled;
        self
    }

    /// Enable component emit analysis.
    pub fn with_component_emits(mut self, enabled: bool) -> Self {
        self.component_emits = enabled;
        self
    }

    /// Enable event bubbling analysis.
    pub fn with_event_bubbling(mut self, enabled: bool) -> Self {
        self.event_bubbling = enabled;
        self
    }

    /// Enable provide/inject analysis.
    pub fn with_provide_inject(mut self, enabled: bool) -> Self {
        self.provide_inject = enabled;
        self
    }

    /// Enable unique ID analysis.
    pub fn with_unique_ids(mut self, enabled: bool) -> Self {
        self.unique_ids = enabled;
        self
    }

    /// Enable server/client boundary analysis.
    pub fn with_server_client_boundary(mut self, enabled: bool) -> Self {
        self.server_client_boundary = enabled;
        self
    }

    /// Enable error/suspense boundary analysis.
    pub fn with_error_suspense_boundary(mut self, enabled: bool) -> Self {
        self.error_suspense_boundary = enabled;
        self
    }

    /// Enable reactivity tracking.
    pub fn with_reactivity_tracking(mut self, enabled: bool) -> Self {
        self.reactivity_tracking = enabled;
        self
    }

    /// Enable async reactive race-condition analysis.
    pub fn with_race_conditions(mut self, enabled: bool) -> Self {
        self.race_conditions = enabled;
        self
    }

    /// Enable circular dependency detection.
    pub fn with_circular_dependencies(mut self, enabled: bool) -> Self {
        self.circular_dependencies = enabled;
        self
    }

    /// Set maximum import depth for warnings.
    pub fn with_max_import_depth(mut self, depth: Option<usize>) -> Self {
        self.max_import_depth = depth;
        self
    }

    /// Enable component resolution checking.
    pub fn with_component_resolution(mut self, enabled: bool) -> Self {
        self.component_resolution = enabled;
        self
    }

    /// Enable props validation.
    pub fn with_props_validation(mut self, enabled: bool) -> Self {
        self.props_validation = enabled;
        self
    }

    /// Check if any analysis is enabled.
    pub fn any_enabled(&self) -> bool {
        self.fallthrough_attrs
            || self.component_emits
            || self.event_bubbling
            || self.provide_inject
            || self.unique_ids
            || self.server_client_boundary
            || self.error_suspense_boundary
            || self.reactivity_tracking
            || self.race_conditions
            || self.setup_context
            || self.circular_dependencies
            || self.component_resolution
            || self.props_validation
    }

    /// Enable setup context violation analysis.
    pub fn with_setup_context(mut self, enabled: bool) -> Self {
        self.setup_context = enabled;
        self
    }
}

/// Result of cross-file analysis.
#[derive(Debug, Default)]
pub struct CrossFileResult {
    /// All diagnostics from cross-file analysis.
    pub diagnostics: Vec<CrossFileDiagnostic>,

    /// Fallthrough attribute information per component.
    pub fallthrough_info: Vec<analyzers::FallthroughInfo>,

    /// Emit flow information.
    pub emit_flows: Vec<analyzers::EmitFlow>,

    /// Event bubbling information.
    pub event_bubbles: Vec<analyzers::EventBubble>,

    /// Provide/inject matches.
    pub provide_inject_matches: Vec<analyzers::ProvideInjectMatch>,

    /// Provide/inject tree, populated when provide/inject analysis is enabled.
    pub provide_inject_tree: Option<analyzers::ProvideInjectTree>,

    /// Unique ID issues.
    pub unique_id_issues: Vec<analyzers::UniqueIdIssue>,

    /// Boundary information.
    pub boundaries: Vec<analyzers::BoundaryInfo>,

    /// Reactivity issues.
    pub reactivity_issues: Vec<analyzers::ReactivityIssue>,

    /// Cross-file reactivity issues.
    pub cross_file_reactivity_issues: Vec<analyzers::CrossFileReactivityIssue>,

    /// Async race-condition issues.
    pub race_condition_issues: Vec<analyzers::RaceConditionIssue>,

    /// Setup context violations (CSRP/memory leaks).
    pub setup_context_issues: Vec<analyzers::SetupContextIssue>,

    /// Circular dependencies (as paths of file IDs).
    pub circular_deps: Vec<Vec<FileId>>,

    /// Component resolution issues.
    pub component_resolution_issues: Vec<analyzers::ComponentResolutionIssue>,

    /// Props validation issues.
    pub props_validation_issues: Vec<analyzers::PropsValidationIssue>,

    /// Statistics.
    pub stats: CrossFileStats,
}

/// Statistics from cross-file analysis.
#[derive(Debug, Default, Clone)]
pub struct CrossFileStats {
    /// Number of files analyzed.
    pub files_analyzed: usize,
    /// Number of Vue components.
    pub vue_components: usize,
    /// Number of edges in dependency graph.
    pub dependency_edges: usize,
    /// Number of diagnostics by severity.
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    /// Analysis time in milliseconds.
    pub analysis_time_ms: f64,
}
