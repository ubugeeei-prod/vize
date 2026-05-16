//! Setup context propagation, queries, and markdown output.
//!
//! Provides analysis methods for the [`CallGraph`]: propagating setup context
//! through call edges, querying setup context status, and generating
//! markdown visualizations.

use super::{
    CallEdge, CallGraph, ComposableCallInfo, FunctionDef, FunctionId, SetupContextKind, SmallVec,
    VueApiCall, VueApiCategory,
};
use vize_carton::append;
use vize_carton::String;

impl CallGraph {
    /// Check if a function (or None for top-level) is in setup context.
    #[inline]
    pub fn is_in_setup_context(&self, func_id: Option<FunctionId>) -> bool {
        match func_id {
            Some(id) => self.setup_context_functions.contains(&id),
            None => {
                // Top-level in script setup is setup context
                true
            }
        }
    }

    /// Propagate setup context through call edges.
    /// Call this after all functions and edges are added.
    pub fn propagate_setup_context(&mut self) {
        if self.setup_function.is_none() {
            return;
        }

        // BFS from setup function
        let mut queue: SmallVec<[FunctionId; 16]> = SmallVec::new();
        queue.extend(self.setup_context_functions.iter().copied());

        while let Some(func_id) = queue.pop() {
            // Find all functions called by this function
            for edge in &self.call_edges {
                if edge.caller == func_id && !self.setup_context_functions.contains(&edge.callee) {
                    self.setup_context_functions.insert(edge.callee);
                    queue.push(edge.callee);

                    // Update function def
                    if let Some(func) = self.functions.get_mut(edge.callee.as_u32() as usize) {
                        func.called_in_setup = true;
                    }
                }
            }
        }

        // Update vue_api_calls in_setup_context
        // Collect updates first to avoid borrow conflict
        let vue_updates: Vec<_> = self
            .vue_api_calls
            .iter()
            .enumerate()
            .map(|(i, call)| {
                (
                    i,
                    self.setup_context_functions.contains(
                        &call
                            .containing_function
                            .unwrap_or(FunctionId::new(u32::MAX)),
                    ),
                )
            })
            .collect();
        for (i, in_setup) in vue_updates {
            // Top-level is always in setup context for script setup
            let containing = self.vue_api_calls[i].containing_function;
            self.vue_api_calls[i].in_setup_context = containing.is_none() || in_setup;
        }

        // Update composable_calls in_setup_context
        let composable_updates: Vec<_> = self
            .composable_calls
            .iter()
            .enumerate()
            .map(|(i, call)| {
                (
                    i,
                    self.setup_context_functions.contains(
                        &call
                            .containing_function
                            .unwrap_or(FunctionId::new(u32::MAX)),
                    ),
                )
            })
            .collect();
        for (i, in_setup) in composable_updates {
            let containing = self.composable_calls[i].containing_function;
            self.composable_calls[i].in_setup_context = containing.is_none() || in_setup;
        }
    }

    /// Get all Vue API calls.
    #[inline]
    pub fn vue_api_calls(&self) -> &[VueApiCall] {
        &self.vue_api_calls
    }

    /// Get Vue API calls outside setup context (potential issues).
    pub fn vue_api_calls_outside_setup(&self) -> impl Iterator<Item = &VueApiCall> {
        self.vue_api_calls.iter().filter(|c| !c.in_setup_context)
    }

    /// Get all composable calls.
    #[inline]
    pub fn composable_calls(&self) -> &[ComposableCallInfo] {
        &self.composable_calls
    }

    /// Get composable calls outside setup context (potential issues).
    pub fn composable_calls_outside_setup(&self) -> impl Iterator<Item = &ComposableCallInfo> {
        self.composable_calls.iter().filter(|c| !c.in_setup_context)
    }

    /// Get all function definitions.
    #[inline]
    pub fn functions(&self) -> &[FunctionDef] {
        &self.functions
    }

    /// Get a function by ID.
    #[inline]
    pub fn get_function(&self, id: FunctionId) -> Option<&FunctionDef> {
        self.functions.get(id.as_u32() as usize)
    }

    /// Get functions by name.
    pub fn get_functions_by_name(&self, name: &str) -> Option<&[FunctionId]> {
        self.function_by_name.get(name).map(|v| v.as_slice())
    }

    /// Get all call edges.
    #[inline]
    pub fn call_edges(&self) -> &[CallEdge] {
        &self.call_edges
    }

    /// Get the setup function ID.
    #[inline]
    pub fn setup_function(&self) -> Option<FunctionId> {
        self.setup_function
    }

    /// Check if a function is a composable.
    #[inline]
    pub fn is_composable(&self, id: FunctionId) -> bool {
        self.get_function(id)
            .map(|f| f.is_composable)
            .unwrap_or(false)
    }

    /// Get the setup context kind for a given location.
    pub fn get_setup_context_kind(&self, func_id: Option<FunctionId>) -> SetupContextKind {
        match func_id {
            None => {
                // Top-level - check if we have a setup function
                if self.setup_function.is_some() {
                    SetupContextKind::SetupBody
                } else {
                    SetupContextKind::None
                }
            }
            Some(id) => {
                if Some(id) == self.setup_function {
                    SetupContextKind::SetupBody
                } else if self.setup_context_functions.contains(&id) {
                    // Check if this function is a composable
                    if self.is_composable(id) {
                        SetupContextKind::Composable
                    } else {
                        // It's a callback or nested function in setup context
                        SetupContextKind::ComposableCallback
                    }
                } else {
                    SetupContextKind::None
                }
            }
        }
    }

    /// Generate a markdown visualization of the call graph.
    pub fn to_markdown(&self) -> String {
        let mut out = String::with_capacity(2048);

        out.push_str("## Function Call Graph\n\n");

        // Setup function
        if let Some(setup_id) = self.setup_function {
            if let Some(func) = self.get_function(setup_id) {
                append!(
                    out,
                    "**Setup Function**: `{}` (offset: {}..{})\n\n",
                    func.name.as_deref().unwrap_or("<anonymous>"),
                    func.start,
                    func.end
                );
            }
        }

        // Functions in setup context
        out.push_str("### Functions in Setup Context\n\n");
        for func in &self.functions {
            if func.called_in_setup || Some(func.id) == self.setup_function {
                let marker = if func.is_composable {
                    "🔧"
                } else if func.uses_vue_apis {
                    "⚡"
                } else {
                    "📦"
                };
                append!(
                    out,
                    "- {} `{}` ({}..{})\n",
                    marker,
                    func.name.as_deref().unwrap_or("<anonymous>"),
                    func.start,
                    func.end
                );
            }
        }

        // Vue API calls
        out.push_str("\n### Vue API Calls\n\n");
        out.push_str("| API | Category | In Setup | Offset |\n");
        out.push_str("|-----|----------|----------|--------|\n");
        for call in &self.vue_api_calls {
            let in_setup = if call.in_setup_context { "✅" } else { "❌" };
            append!(
                out,
                "| `{}` | {:?} | {} | {}..{} |\n",
                call.name,
                call.category,
                in_setup,
                call.start,
                call.end
            );
        }

        // Composable calls
        if !self.composable_calls.is_empty() {
            out.push_str("\n### Composable Calls\n\n");
            out.push_str("| Composable | Source | In Setup | Offset |\n");
            out.push_str("|------------|--------|----------|--------|\n");
            for call in &self.composable_calls {
                let in_setup = if call.in_setup_context { "✅" } else { "❌" };
                let source = call.source.as_deref().unwrap_or("-");
                append!(
                    out,
                    "| `{}` | `{}` | {} | {}..{} |\n",
                    call.name,
                    source,
                    in_setup,
                    call.start,
                    call.end
                );
            }
        }

        // Issues (Vue APIs outside setup)
        let issues: Vec<_> = self.vue_api_calls_outside_setup().collect();
        if !issues.is_empty() {
            out.push_str("\n### ⚠️ Issues: Vue APIs Outside Setup Context\n\n");
            for call in issues {
                append!(
                    out,
                    "- `{}` at {}..{} - Vue {} API called outside setup context\n",
                    call.name,
                    call.start,
                    call.end,
                    match call.category {
                        VueApiCategory::Reactivity => "reactivity",
                        VueApiCategory::Lifecycle => "lifecycle",
                        VueApiCategory::DependencyInjection => "dependency injection",
                        VueApiCategory::Watcher => "watcher",
                        VueApiCategory::TemplateRef => "template ref",
                        VueApiCategory::Other => "",
                    }
                );
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::{CallGraph, SetupContextKind, VueApiCategory};
    use crate::call_graph::builder::{categorize_vue_api, is_composable_name, is_vue_api};
    use crate::scope::ScopeId;
    use vize_carton::CompactString;

    #[test]
    fn test_categorize_vue_api() {
        assert_eq!(categorize_vue_api("ref"), VueApiCategory::Reactivity);
        assert_eq!(categorize_vue_api("computed"), VueApiCategory::Reactivity);
        assert_eq!(categorize_vue_api("onMounted"), VueApiCategory::Lifecycle);
        assert_eq!(
            categorize_vue_api("provide"),
            VueApiCategory::DependencyInjection
        );
        assert_eq!(categorize_vue_api("watch"), VueApiCategory::Watcher);
        assert_eq!(
            categorize_vue_api("useTemplateRef"),
            VueApiCategory::TemplateRef
        );
        assert_eq!(categorize_vue_api("nextTick"), VueApiCategory::Other);
    }

    #[test]
    fn test_is_composable_name() {
        assert!(is_composable_name("useCounter"));
        assert!(is_composable_name("useAuth"));
        assert!(is_composable_name("useFetch"));
        assert!(!is_composable_name("use")); // Too short
        assert!(!is_composable_name("usecounter")); // Lowercase after use
        assert!(!is_composable_name("counter")); // Doesn't start with use
    }

    #[test]
    fn test_is_vue_api() {
        assert!(is_vue_api("ref"));
        assert!(is_vue_api("reactive"));
        assert!(is_vue_api("computed"));
        assert!(is_vue_api("onMounted"));
        assert!(is_vue_api("provide"));
        assert!(is_vue_api("inject"));
        assert!(is_vue_api("watch"));
        assert!(!is_vue_api("myFunction"));
        assert!(!is_vue_api("useState")); // React API, not Vue
    }

    #[test]
    fn test_call_graph_basic() {
        let mut graph = CallGraph::new();

        // Add setup function
        let setup_id = graph.add_function(
            Some(CompactString::new("setup")),
            ScopeId::new(1),
            None,
            false,
            0,
            100,
        );
        graph.set_setup_function(setup_id);

        // Add a helper function
        let helper_id = graph.add_function(
            Some(CompactString::new("useCounter")),
            ScopeId::new(2),
            None,
            false,
            110,
            200,
        );

        // Add call edge: setup -> useCounter
        graph.add_call_edge(setup_id, helper_id, 50);

        // Add Vue API call in helper
        graph.add_vue_api_call(
            CompactString::new("ref"),
            ScopeId::new(2),
            Some(helper_id),
            150,
            155,
        );

        // Propagate setup context
        graph.propagate_setup_context();

        // Verify
        assert!(graph.is_in_setup_context(Some(setup_id)));
        assert!(graph.is_in_setup_context(Some(helper_id)));

        let func = graph.get_function(helper_id).unwrap();
        assert!(func.called_in_setup);
        assert!(func.uses_vue_apis);
        assert!(func.is_composable);
    }

    #[test]
    fn test_vue_api_outside_setup() {
        let mut graph = CallGraph::new();

        // Add setup function
        let setup_id = graph.add_function(
            Some(CompactString::new("setup")),
            ScopeId::new(1),
            None,
            false,
            0,
            100,
        );
        graph.set_setup_function(setup_id);

        // Add a function NOT called from setup
        let outside_id = graph.add_function(
            Some(CompactString::new("outsideFunction")),
            ScopeId::new(2),
            None,
            false,
            200,
            300,
        );

        // Add Vue API call in the outside function
        graph.add_vue_api_call(
            CompactString::new("ref"),
            ScopeId::new(2),
            Some(outside_id),
            250,
            255,
        );

        // Propagate
        graph.propagate_setup_context();

        // Verify the issue is detected
        let issues: Vec<_> = graph.vue_api_calls_outside_setup().collect();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].name.as_str(), "ref");
    }

    #[test]
    fn test_setup_context_kind() {
        let mut graph = CallGraph::new();

        let setup_id = graph.add_function(
            Some(CompactString::new("setup")),
            ScopeId::new(1),
            None,
            false,
            0,
            100,
        );
        graph.set_setup_function(setup_id);

        let composable_id = graph.add_function(
            Some(CompactString::new("useAuth")),
            ScopeId::new(2),
            None,
            false,
            110,
            200,
        );
        graph.add_call_edge(setup_id, composable_id, 50);

        let callback_id =
            graph.add_function(None, ScopeId::new(3), Some(composable_id), true, 150, 180);
        graph.add_call_edge(composable_id, callback_id, 160);

        graph.propagate_setup_context();

        assert_eq!(
            graph.get_setup_context_kind(Some(setup_id)),
            SetupContextKind::SetupBody
        );
        assert_eq!(
            graph.get_setup_context_kind(Some(composable_id)),
            SetupContextKind::Composable
        );
        assert_eq!(
            graph.get_setup_context_kind(Some(callback_id)),
            SetupContextKind::ComposableCallback
        );
    }
}
