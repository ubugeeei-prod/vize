//! Call graph construction and Vue API categorization.
//!
//! Provides methods for building the call graph: adding functions,
//! Vue API calls, composable calls, and call edges. Also contains
//! free functions for categorizing Vue APIs and identifying composables.

use super::{
    CallEdge, CallGraph, CompactString, ComposableCallInfo, FunctionDef, FunctionId, FxHashMap,
    FxHashSet, ScopeId, SmallVec, VueApiCall, VueApiCategory,
};

impl CallGraph {
    /// Create a new empty call graph.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(functions: usize, calls: usize) -> Self {
        Self {
            functions: Vec::with_capacity(functions),
            vue_api_calls: Vec::with_capacity(calls),
            composable_calls: Vec::with_capacity(calls / 4),
            call_edges: Vec::with_capacity(calls),
            function_by_name: FxHashMap::default(),
            setup_context_functions: FxHashSet::default(),
            setup_function: None,
            next_id: 0,
        }
    }

    /// Add a function definition.
    pub fn add_function(
        &mut self,
        name: Option<CompactString>,
        scope_id: ScopeId,
        parent_function: Option<FunctionId>,
        is_arrow: bool,
        start: u32,
        end: u32,
    ) -> FunctionId {
        let id = FunctionId::new(self.next_id);
        self.next_id += 1;

        // Check if this looks like a composable
        let is_composable = name
            .as_ref()
            .map(|n| n.starts_with("use") && n.len() > 3)
            .unwrap_or(false);

        let def = FunctionDef {
            id,
            name: name.clone(),
            scope_id,
            parent_function,
            is_arrow,
            called_in_setup: false,
            uses_vue_apis: false,
            is_composable,
            start,
            end,
        };

        self.functions.push(def);

        // Index by name
        if let Some(name) = name {
            self.function_by_name.entry(name).or_default().push(id);
        }

        id
    }

    /// Mark a function as the setup function.
    #[inline]
    pub fn set_setup_function(&mut self, id: FunctionId) {
        self.setup_function = Some(id);
        self.setup_context_functions.insert(id);
    }

    /// Add a Vue API call.
    pub fn add_vue_api_call(
        &mut self,
        name: CompactString,
        scope_id: ScopeId,
        containing_function: Option<FunctionId>,
        start: u32,
        end: u32,
    ) {
        let category = categorize_vue_api(&name);
        let in_setup_context = self.is_in_setup_context(containing_function);

        self.vue_api_calls.push(VueApiCall {
            name,
            category,
            scope_id,
            containing_function,
            in_setup_context,
            start,
            end,
        });

        // Mark containing function as using Vue APIs
        if let Some(func_id) = containing_function {
            if let Some(func) = self.functions.get_mut(func_id.as_u32() as usize) {
                func.uses_vue_apis = true;
            }
        }
    }

    /// Add a composable call.
    #[allow(clippy::too_many_arguments)]
    pub fn add_composable_call(
        &mut self,
        name: CompactString,
        source: Option<CompactString>,
        scope_id: ScopeId,
        containing_function: Option<FunctionId>,
        local_binding: Option<CompactString>,
        start: u32,
        end: u32,
    ) {
        let in_setup_context = self.is_in_setup_context(containing_function);

        self.composable_calls.push(ComposableCallInfo {
            name,
            source,
            scope_id,
            containing_function,
            in_setup_context,
            local_binding,
            vue_apis_used: SmallVec::new(),
            start,
            end,
        });
    }

    /// Add a call edge between functions.
    pub fn add_call_edge(&mut self, caller: FunctionId, callee: FunctionId, call_site: u32) {
        self.call_edges.push(CallEdge {
            caller,
            callee,
            call_site,
        });

        // If caller is in setup context, callee is too
        if self.setup_context_functions.contains(&caller) {
            self.setup_context_functions.insert(callee);
            if let Some(func) = self.functions.get_mut(callee.as_u32() as usize) {
                func.called_in_setup = true;
            }
        }
    }
}

/// Categorize a Vue API by name.
pub(crate) fn categorize_vue_api(name: &str) -> VueApiCategory {
    match name {
        // Reactivity
        "ref" | "shallowRef" | "triggerRef" | "customRef" | "reactive" | "shallowReactive"
        | "readonly" | "shallowReadonly" | "computed" | "toRef" | "toRefs" | "toValue"
        | "toRaw" | "markRaw" | "isRef" | "isReactive" | "isReadonly" | "isProxy" | "unref" => {
            VueApiCategory::Reactivity
        }

        // Lifecycle
        "onMounted" | "onUpdated" | "onUnmounted" | "onBeforeMount" | "onBeforeUpdate"
        | "onBeforeUnmount" | "onErrorCaptured" | "onRenderTracked" | "onRenderTriggered"
        | "onActivated" | "onDeactivated" | "onServerPrefetch" => VueApiCategory::Lifecycle,

        // Dependency Injection
        "provide" | "inject" | "hasInjectionContext" => VueApiCategory::DependencyInjection,

        // Watchers
        "watch" | "watchEffect" | "watchPostEffect" | "watchSyncEffect" => VueApiCategory::Watcher,

        // Template Refs
        "useTemplateRef" => VueApiCategory::TemplateRef,

        // Other
        _ => VueApiCategory::Other,
    }
}

/// Check if a function name looks like a composable.
#[inline]
pub fn is_composable_name(name: &str) -> bool {
    name.starts_with("use")
        && name.len() > 3
        && name
            .chars()
            .nth(3)
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
}

/// Check if a name is a Vue API.
#[inline]
pub fn is_vue_api(name: &str) -> bool {
    matches!(
        name,
        "ref"
            | "shallowRef"
            | "triggerRef"
            | "customRef"
            | "reactive"
            | "shallowReactive"
            | "readonly"
            | "shallowReadonly"
            | "computed"
            | "toRef"
            | "toRefs"
            | "toValue"
            | "toRaw"
            | "markRaw"
            | "isRef"
            | "isReactive"
            | "isReadonly"
            | "isProxy"
            | "unref"
            | "onMounted"
            | "onUpdated"
            | "onUnmounted"
            | "onBeforeMount"
            | "onBeforeUpdate"
            | "onBeforeUnmount"
            | "onErrorCaptured"
            | "onRenderTracked"
            | "onRenderTriggered"
            | "onActivated"
            | "onDeactivated"
            | "onServerPrefetch"
            | "provide"
            | "inject"
            | "hasInjectionContext"
            | "watch"
            | "watchEffect"
            | "watchPostEffect"
            | "watchSyncEffect"
            | "useTemplateRef"
            | "nextTick"
            | "defineComponent"
            | "defineAsyncComponent"
            | "defineCustomElement"
            | "getCurrentInstance"
            | "useSlots"
            | "useAttrs"
    )
}
