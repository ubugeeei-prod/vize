//! Cross-file diagnostic types.
//!
//! Diagnostics produced by cross-file analysis that span multiple files.
//!
//! This module is split into:
//! - Core types and constructors (this file)
//! - [`rules`]: Diagnostic code identifiers for filtering/configuration
//! - [`formatting`]: Rich Markdown rendering of diagnostics

mod formatting;
mod rules;

use super::registry::FileId;
use vize_carton::CompactString;

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    /// Error - must be fixed.
    Error = 0,
    /// Warning - should be addressed.
    Warning = 1,
    /// Information - for awareness.
    Info = 2,
    /// Hint - suggestion for improvement.
    Hint = 3,
}

impl DiagnosticSeverity {
    /// Get display name.
    #[inline]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
        }
    }
}

/// Kind of cross-file diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossFileDiagnosticKind {
    // === Fallthrough Attributes ===
    /// Component doesn't use $attrs but parent passes attributes.
    UnusedFallthroughAttrs { passed_attrs: Vec<CompactString> },
    /// `inheritAttrs: false` but $attrs not explicitly bound.
    InheritAttrsDisabledUnused,
    /// Multiple root elements without explicit v-bind="$attrs".
    MultiRootMissingAttrs,

    // === Component Emits ===
    /// Emit called but not declared in defineEmits.
    UndeclaredEmit { emit_name: CompactString },
    /// Declared emit is never called.
    UnusedEmit { emit_name: CompactString },
    /// Parent listens for event not emitted by child.
    UnmatchedEventListener { event_name: CompactString },

    // === Event Bubbling ===
    /// Event emitted but no ancestor handles it.
    UnhandledEvent {
        event_name: CompactString,
        depth: usize,
    },
    /// Event handler modifiers may cause issues (.stop, .prevent).
    EventModifierIssue {
        event_name: CompactString,
        modifier: CompactString,
    },

    // === Provide/Inject ===
    /// inject() key has no matching provide() in ancestors.
    UnmatchedInject { key: CompactString },
    /// provide() key is never injected by descendants.
    UnusedProvide { key: CompactString },
    /// Type mismatch between provide and inject.
    ProvideInjectTypeMismatch {
        key: CompactString,
        provided_type: CompactString,
        injected_type: CompactString,
    },
    /// provide/inject uses string key instead of Symbol/InjectionKey.
    /// String keys lack type safety and can collide.
    ProvideInjectWithoutSymbol {
        key: CompactString,
        is_provide: bool,
    },
    /// provide() passes a non-reactive value to consumers.
    NonReactiveProvideValue { key: CompactString },

    // === Unique Element IDs ===
    /// Duplicate ID attribute across components.
    DuplicateElementId {
        id: CompactString,
        locations: Vec<(FileId, u32)>,
    },
    /// ID generated in v-for may not be unique.
    NonUniqueIdInLoop { id_expression: CompactString },

    // === Server/Client Boundary ===
    /// Browser API used in potentially SSR context.
    BrowserApiInSsr {
        api: CompactString,
        context: CompactString,
    },
    /// Async component not wrapped in Suspense.
    AsyncWithoutSuspense { component_name: CompactString },
    /// Hydration mismatch risk (client-only content).
    HydrationMismatchRisk { reason: CompactString },

    // === Error/Suspense Boundaries ===
    /// Error thrown but no onErrorCaptured in ancestors.
    UncaughtErrorBoundary,
    /// Async operation without Suspense boundary.
    MissingSuspenseBoundary,
    /// Nested Suspense without fallback.
    SuspenseWithoutFallback,

    // === Dependency Graph ===
    /// Circular dependency detected.
    CircularDependency { cycle: Vec<CompactString> },
    /// Deep import chain (performance concern).
    DeepImportChain {
        depth: usize,
        chain: Vec<CompactString>,
    },

    // === Component Resolution (Static Analysis) ===
    /// Component used in template but not imported/registered.
    UnregisteredComponent {
        component_name: CompactString,
        template_offset: u32,
    },
    /// Import specifier could not be resolved to a file.
    UnresolvedImport {
        specifier: CompactString,
        import_offset: u32,
    },

    // === Props Validation (Static Analysis) ===
    /// Prop passed to component but not declared in child's defineProps.
    UndeclaredProp {
        prop_name: CompactString,
        component_name: CompactString,
    },
    /// Required prop not passed to component.
    MissingRequiredProp {
        prop_name: CompactString,
        component_name: CompactString,
    },
    /// Prop type mismatch (literal type check).
    PropTypeMismatch {
        prop_name: CompactString,
        expected_type: CompactString,
        actual_type: CompactString,
    },

    // === Slot Validation (Static Analysis) ===
    /// Slot used but not defined in child component's defineSlots.
    UndefinedSlot {
        slot_name: CompactString,
        component_name: CompactString,
    },

    // === Setup Context Violations ===
    /// Reactivity API (ref, reactive, computed) called outside setup context.
    /// This can cause CSRP (Client-Side Rendering Problems) and state pollution.
    ReactivityOutsideSetup {
        api_name: CompactString,
        context_description: CompactString,
    },
    /// Lifecycle hook called outside setup context.
    /// These hooks must be called synchronously during setup.
    LifecycleOutsideSetup {
        hook_name: CompactString,
        context_description: CompactString,
    },
    /// Lifecycle hook that usually registers resources has no matching cleanup hook.
    LifecycleHookWithoutCleanup {
        hook_name: CompactString,
        cleanup_hook: CompactString,
    },
    /// Watcher (watch, watchEffect) called outside setup context.
    /// This can cause memory leaks as the watcher won't be automatically cleaned up.
    WatcherOutsideSetup {
        api_name: CompactString,
        context_description: CompactString,
    },
    /// Dependency injection (provide, inject) called outside setup context.
    /// These must be called during component setup.
    DependencyInjectionOutsideSetup {
        api_name: CompactString,
        context_description: CompactString,
    },
    /// Composable function called outside setup context.
    /// Composables that use Vue APIs must be called within setup.
    ComposableOutsideSetup {
        composable_name: CompactString,
        context_description: CompactString,
    },

    // === Reactivity Reference Loss ===
    /// Spread operator used on reactive object, breaking reactivity.
    /// `const copy = { ...reactive }` creates a non-reactive shallow copy.
    SpreadBreaksReactivity {
        source_name: CompactString,
        source_type: CompactString, // "reactive" | "ref" | "props"
    },
    /// Reactive variable reassigned, breaking reactivity reference.
    /// `let r = ref(0); r = ref(1)` loses the original ref.
    ReassignmentBreaksReactivity {
        variable_name: CompactString,
        original_type: CompactString,
    },
    /// Reactive value extracted to plain variable, breaking reactivity.
    /// `const count = ref(0).value` loses reactivity.
    ValueExtractionBreaksReactivity {
        source_name: CompactString,
        extracted_value: CompactString,
    },
    /// Destructuring reactive object/props without toRefs, breaking reactivity.
    /// `const { count } = props` loses reactivity.
    DestructuringBreaksReactivity {
        source_name: CompactString,
        destructured_keys: Vec<CompactString>,
        suggestion: CompactString, // "toRefs" | "toRef" | "storeToRefs"
    },
    /// Reactive reference escapes scope implicitly via function parameter.
    /// This makes the data flow implicit and harder to trace.
    ReactiveReferenceEscapes {
        variable_name: CompactString,
        escaped_via: CompactString, // "function call" | "return" | "assignment to outer scope"
        target_name: Option<CompactString>, // function name or variable name if known
    },
    /// Reactive object mutated after being passed to external function.
    /// This can cause unexpected side effects.
    ReactiveObjectMutatedAfterEscape {
        variable_name: CompactString,
        mutation_site: u32,
        escape_site: u32,
    },
    /// Circular reactive dependency detected.
    /// This can cause infinite update loops or stack overflow.
    CircularReactiveDependency { cycle: Vec<CompactString> },
    /// Watch callback that only mutates a reactive value could be computed.
    /// `watch(a, () => { b.value = transform(a.value) })` → `const b = computed(() => transform(a.value))`
    WatchMutationCanBeComputed {
        watch_source: CompactString,
        mutated_target: CompactString,
        suggested_computed: CompactString,
    },
    /// DOM API (document, window) accessed outside of lifecycle hooks or nextTick.
    /// In SSR or before mount, the DOM doesn't exist yet.
    DomAccessWithoutNextTick {
        api: CompactString,
        context: CompactString, // "setup" | "computed" | "watch callback"
    },

    // === Ultra-Strict Diagnostics (Rust-like paranoia) ===
    /// Computed property contains side effects (mutations, console.log, API calls).
    /// Computed should be pure functions - side effects make them unpredictable.
    ComputedHasSideEffects {
        computed_name: CompactString,
        side_effect: CompactString, // "mutation" | "console" | "fetch" | "assignment"
    },
    /// Reactive state declared at module scope risks Cross-request State Pollution (CSRP).
    /// In SSR, module-level state is shared across all requests.
    ReactiveStateAtModuleScope {
        variable_name: CompactString,
        reactive_type: CompactString, // "ref" | "reactive" | "computed"
    },
    /// Template ref is accessed during setup (before it's populated).
    /// Template refs are `null` until the component is mounted.
    TemplateRefAccessedBeforeMount {
        ref_name: CompactString,
        access_context: CompactString, // "setup" | "computed" | "watchEffect"
    },
    /// Reactive state accessed across an async boundary without proper handling.
    /// The component may have unmounted or the value changed before await returns.
    AsyncBoundaryCrossing {
        variable_name: CompactString,
        async_context: CompactString, // "await" | "setTimeout" | "promise callback"
    },
    /// Injected reactive state is asynchronously mutated by a consumer.
    /// Provider-owned state can be overwritten by stale consumer work.
    InjectedAsyncMutationRace {
        key: CompactString,
        target_name: CompactString,
        async_context: CompactString,
        writer_count: usize,
    },
    /// Closure captures reactive state implicitly.
    /// Like Rust's closure capture, this creates hidden dependencies.
    ClosureCapturesReactive {
        closure_context: CompactString,
        captured_variables: Vec<CompactString>,
    },
    /// Object identity comparison (===) on reactive objects.
    /// Reactive proxies have different identity than raw objects.
    ObjectIdentityComparison {
        left_operand: CompactString,
        right_operand: CompactString,
    },
    /// Reactive state is exported from module, creating global mutable state.
    /// This violates encapsulation and makes state flow hard to trace.
    ReactiveStateExported {
        variable_name: CompactString,
        export_type: CompactString, // "named" | "default" | "re-export"
    },
    /// Deep access on shallowRef/shallowReactive bypasses reactivity.
    /// Changes to nested properties won't trigger updates.
    ShallowReactiveDeepAccess {
        variable_name: CompactString,
        access_path: CompactString, // "value.nested.prop"
    },
    /// toRaw() value is mutated, bypassing reactivity entirely.
    /// Mutations to raw values don't trigger reactive updates.
    ToRawMutation {
        original_variable: CompactString,
        mutation_type: CompactString, // "property assignment" | "method call"
    },
    /// Event listener added without corresponding cleanup.
    /// This causes memory leaks if the component is destroyed.
    EventListenerWithoutCleanup {
        event_name: CompactString,
        target: CompactString, // "document" | "window" | "element"
    },
    /// Reactive array mutated with non-triggering method.
    /// Some array methods don't trigger reactive updates.
    ArrayMutationNotTriggering {
        array_name: CompactString,
        method: CompactString, // "sort" | "reverse" | "fill" direct assignment
    },
    /// Store getter accessed in setup without storeToRefs.
    /// Pinia getters need storeToRefs() to maintain reactivity.
    PiniaGetterWithoutStoreToRefs {
        store_name: CompactString,
        getter_name: CompactString,
    },
    /// watchEffect callback contains async operations.
    /// Async operations in watchEffect can cause race conditions.
    WatchEffectWithAsync {
        async_operation: CompactString, // "await" | "setTimeout" | "fetch"
    },

    // === Unified Setup Context Violation ===
    /// Vue API called outside of setup context (module-level in non-setup script).
    /// Wraps SetupContextViolationKind for unified handling.
    SetupContextViolation {
        kind: vize_croquis::setup_context::SetupContextViolationKind,
        api_name: CompactString,
    },
}

/// A cross-file diagnostic with location information.
#[derive(Debug, Clone)]
pub struct CrossFileDiagnostic {
    /// Diagnostic kind.
    pub kind: CrossFileDiagnosticKind,
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Primary file where the issue originates.
    pub primary_file: FileId,
    /// Start offset in the primary file.
    pub primary_offset: u32,
    /// End offset in the primary file (for highlighting range).
    pub primary_end_offset: u32,
    /// Related files involved in this diagnostic.
    pub related_files: Vec<(FileId, u32, CompactString)>,
    /// Human-readable message.
    pub message: CompactString,
    /// Optional fix suggestion.
    pub suggestion: Option<CompactString>,
}

impl CrossFileDiagnostic {
    /// Create a new diagnostic.
    pub fn new(
        kind: CrossFileDiagnosticKind,
        severity: DiagnosticSeverity,
        primary_file: FileId,
        primary_offset: u32,
        message: impl Into<CompactString>,
    ) -> Self {
        Self {
            kind,
            severity,
            primary_file,
            primary_offset,
            primary_end_offset: primary_offset, // Default to same as start
            related_files: Vec::new(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a new diagnostic with span (start and end offset).
    pub fn with_span(
        kind: CrossFileDiagnosticKind,
        severity: DiagnosticSeverity,
        primary_file: FileId,
        primary_offset: u32,
        primary_end_offset: u32,
        message: impl Into<CompactString>,
    ) -> Self {
        Self {
            kind,
            severity,
            primary_file,
            primary_offset,
            primary_end_offset,
            related_files: Vec::new(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Set the end offset for the diagnostic span.
    pub fn with_end_offset(mut self, end_offset: u32) -> Self {
        self.primary_end_offset = end_offset;
        self
    }

    /// Add a related file location.
    pub fn with_related(
        mut self,
        file: FileId,
        offset: u32,
        description: impl Into<CompactString>,
    ) -> Self {
        self.related_files.push((file, offset, description.into()));
        self
    }

    /// Add a fix suggestion.
    pub fn with_suggestion(mut self, suggestion: impl Into<CompactString>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Check if this is an error.
    #[inline]
    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
    }

    /// Check if this is a warning.
    #[inline]
    pub fn is_warning(&self) -> bool {
        self.severity == DiagnosticSeverity::Warning
    }
}

#[cfg(test)]
#[path = "diagnostics/diagnostics_tests.rs"]
mod tests;
