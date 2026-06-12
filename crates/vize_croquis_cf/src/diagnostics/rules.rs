//! Diagnostic rule codes for cross-file diagnostics.
//!
//! Maps each [`CrossFileDiagnosticKind`] to a unique, filterable string code
//! (e.g. `"vize:croquis/cf/unused-attrs"`).

use super::{CrossFileDiagnostic, CrossFileDiagnosticKind};

impl CrossFileDiagnostic {
    /// Get the diagnostic code (for filtering/configuration).
    pub fn code(&self) -> &'static str {
        match &self.kind {
            // Fallthrough Attributes
            CrossFileDiagnosticKind::UnusedFallthroughAttrs { .. } => {
                "vize:croquis/cf/unused-attrs"
            }
            CrossFileDiagnosticKind::InheritAttrsDisabledUnused => {
                "vize:croquis/cf/inherit-attrs-unused"
            }
            CrossFileDiagnosticKind::MultiRootMissingAttrs => "vize:croquis/cf/multi-root-attrs",
            // Component Emits
            CrossFileDiagnosticKind::UndeclaredEmit { .. } => "vize:croquis/cf/undeclared-emit",
            CrossFileDiagnosticKind::UnusedEmit { .. } => "vize:croquis/cf/unused-emit",
            CrossFileDiagnosticKind::UnmatchedEventListener { .. } => {
                "vize:croquis/cf/unmatched-listener"
            }
            CrossFileDiagnosticKind::UnhandledEvent { .. } => "vize:croquis/cf/unhandled-event",
            CrossFileDiagnosticKind::EventModifierIssue { .. } => "vize:croquis/cf/event-modifier",
            // Provide/Inject
            CrossFileDiagnosticKind::UnmatchedInject { .. } => "vize:croquis/cf/unmatched-inject",
            CrossFileDiagnosticKind::UnusedProvide { .. } => "vize:croquis/cf/unused-provide",
            CrossFileDiagnosticKind::ProvideInjectTypeMismatch { .. } => {
                "vize:croquis/cf/provide-inject-type"
            }
            CrossFileDiagnosticKind::ProvideInjectWithoutSymbol { is_provide, .. } => {
                if *is_provide {
                    "vize:croquis/cf/provide-without-symbol"
                } else {
                    "vize:croquis/cf/inject-without-symbol"
                }
            }
            CrossFileDiagnosticKind::NonReactiveProvideValue { .. } => {
                "vize:croquis/cf/non-reactive-provide"
            }
            // Unique Element IDs
            CrossFileDiagnosticKind::DuplicateElementId { .. } => "vize:croquis/cf/duplicate-id",
            CrossFileDiagnosticKind::NonUniqueIdInLoop { .. } => "vize:croquis/cf/non-unique-id",
            // Server/Client Boundary
            CrossFileDiagnosticKind::BrowserApiInSsr { .. } => "vize:croquis/cf/browser-api-ssr",
            CrossFileDiagnosticKind::AsyncWithoutSuspense { .. } => {
                "vize:croquis/cf/async-no-suspense"
            }
            CrossFileDiagnosticKind::HydrationMismatchRisk { .. } => {
                "vize:croquis/cf/hydration-risk"
            }
            // Error/Suspense Boundaries
            CrossFileDiagnosticKind::UncaughtErrorBoundary => "vize:croquis/cf/uncaught-error",
            CrossFileDiagnosticKind::MissingSuspenseBoundary => "vize:croquis/cf/missing-suspense",
            CrossFileDiagnosticKind::SuspenseWithoutFallback => {
                "vize:croquis/cf/suspense-no-fallback"
            }
            // Dependency Graph
            CrossFileDiagnosticKind::CircularDependency { .. } => "vize:croquis/cf/circular-dep",
            CrossFileDiagnosticKind::DeepImportChain { .. } => "vize:croquis/cf/deep-import",
            // Component Resolution
            CrossFileDiagnosticKind::UnregisteredComponent { .. } => {
                "vize:croquis/cf/unregistered-component"
            }
            CrossFileDiagnosticKind::UnresolvedImport { .. } => "vize:croquis/cf/unresolved-import",
            // Props Validation
            CrossFileDiagnosticKind::UndeclaredProp { .. } => "vize:croquis/cf/undeclared-prop",
            CrossFileDiagnosticKind::MissingRequiredProp { .. } => {
                "vize:croquis/cf/missing-required-prop"
            }
            CrossFileDiagnosticKind::PropTypeMismatch { .. } => {
                "vize:croquis/cf/prop-type-mismatch"
            }
            // Slot Validation
            CrossFileDiagnosticKind::UndefinedSlot { .. } => "vize:croquis/cf/undefined-slot",
            // Setup Context Violations
            CrossFileDiagnosticKind::ReactivityOutsideSetup { .. } => {
                "vize:croquis/cf/reactivity-outside-setup"
            }
            CrossFileDiagnosticKind::LifecycleOutsideSetup { .. } => {
                "vize:croquis/cf/lifecycle-outside-setup"
            }
            CrossFileDiagnosticKind::LifecycleHookWithoutCleanup { .. } => {
                "vize:croquis/cf/lifecycle-without-cleanup"
            }
            CrossFileDiagnosticKind::WatcherOutsideSetup { .. } => {
                "vize:croquis/cf/watcher-outside-setup"
            }
            CrossFileDiagnosticKind::DependencyInjectionOutsideSetup { .. } => {
                "vize:croquis/cf/di-outside-setup"
            }
            CrossFileDiagnosticKind::ComposableOutsideSetup { .. } => {
                "vize:croquis/cf/composable-outside-setup"
            }
            // Reactivity Reference Loss
            CrossFileDiagnosticKind::SpreadBreaksReactivity { .. } => {
                "vize:croquis/cf/spread-breaks-reactivity"
            }
            CrossFileDiagnosticKind::ReassignmentBreaksReactivity { .. } => {
                "vize:croquis/cf/reassignment-breaks-reactivity"
            }
            CrossFileDiagnosticKind::ValueExtractionBreaksReactivity { .. } => {
                "vize:croquis/cf/value-extraction-breaks-reactivity"
            }
            CrossFileDiagnosticKind::DestructuringBreaksReactivity { .. } => {
                "vize:croquis/cf/destructuring-breaks-reactivity"
            }
            CrossFileDiagnosticKind::ReactiveReferenceEscapes { .. } => {
                "vize:croquis/cf/reference-escapes-scope"
            }
            CrossFileDiagnosticKind::ReactiveObjectMutatedAfterEscape { .. } => {
                "vize:croquis/cf/mutated-after-escape"
            }
            CrossFileDiagnosticKind::CircularReactiveDependency { .. } => {
                "vize:croquis/cf/circular-reactive-dependency"
            }
            CrossFileDiagnosticKind::WatchMutationCanBeComputed { .. } => {
                "vize:croquis/cf/watch-can-be-computed"
            }
            CrossFileDiagnosticKind::DomAccessWithoutNextTick { .. } => {
                "vize:croquis/cf/dom-access-without-next-tick"
            }
            // Ultra-strict diagnostics
            CrossFileDiagnosticKind::ComputedHasSideEffects { .. } => {
                "vize:croquis/cf/computed-side-effects"
            }
            CrossFileDiagnosticKind::ReactiveStateAtModuleScope { .. } => {
                "vize:croquis/cf/module-scope-reactive"
            }
            CrossFileDiagnosticKind::TemplateRefAccessedBeforeMount { .. } => {
                "vize:croquis/cf/template-ref-timing"
            }
            CrossFileDiagnosticKind::AsyncBoundaryCrossing { .. } => {
                "vize:croquis/cf/async-boundary"
            }
            CrossFileDiagnosticKind::InjectedAsyncMutationRace { .. } => {
                "vize:croquis/cf/injected-async-mutation-race"
            }
            CrossFileDiagnosticKind::ClosureCapturesReactive { .. } => {
                "vize:croquis/cf/closure-captures-reactive"
            }
            CrossFileDiagnosticKind::ObjectIdentityComparison { .. } => {
                "vize:croquis/cf/object-identity-comparison"
            }
            CrossFileDiagnosticKind::ReactiveStateExported { .. } => {
                "vize:croquis/cf/reactive-export"
            }
            CrossFileDiagnosticKind::ShallowReactiveDeepAccess { .. } => {
                "vize:croquis/cf/shallow-deep-access"
            }
            CrossFileDiagnosticKind::ToRawMutation { .. } => "vize:croquis/cf/toraw-mutation",
            CrossFileDiagnosticKind::EventListenerWithoutCleanup { .. } => {
                "vize:croquis/cf/event-listener-leak"
            }
            CrossFileDiagnosticKind::ArrayMutationNotTriggering { .. } => {
                "vize:croquis/cf/array-mutation"
            }
            CrossFileDiagnosticKind::PiniaGetterWithoutStoreToRefs { .. } => {
                "vize:croquis/cf/pinia-getter"
            }
            CrossFileDiagnosticKind::WatchEffectWithAsync { .. } => {
                "vize:croquis/cf/watcheffect-async"
            }
            CrossFileDiagnosticKind::SetupContextViolation { .. } => {
                "vize:croquis/cf/setup-context-violation"
            }
        }
    }
}
