//! Stable JavaScript categories and offset classes for cross-file diagnostics.

use vize_croquis_cf::CrossFileDiagnosticKind;
use vize_croquis_cf::CrossFileDiagnosticKind::*;

pub(super) fn diagnostic_kind_to_string(kind: &CrossFileDiagnosticKind) -> &'static str {
    match kind {
        UnusedFallthroughAttrs { .. } | InheritAttrsDisabledUnused | MultiRootMissingAttrs => {
            "fallthrough-attrs"
        }
        UndeclaredEmit { .. } | UnusedEmit { .. } | UnmatchedEventListener { .. } => {
            "component-emit"
        }
        UnhandledEvent { .. } | EventModifierIssue { .. } => "event-bubbling",
        UnmatchedInject { .. }
        | UnusedProvide { .. }
        | ProvideInjectTypeMismatch { .. }
        | ProvideInjectWithoutSymbol { .. }
        | NonReactiveProvideValue { .. } => "provide-inject",
        DuplicateElementId { .. } | NonUniqueIdInLoop { .. } => "unique-ids",
        BrowserApiInSsr { .. } | AsyncWithoutSuspense { .. } | HydrationMismatchRisk { .. } => {
            "ssr-boundary"
        }
        UncaughtErrorBoundary | MissingSuspenseBoundary | SuspenseWithoutFallback => {
            "error-boundary"
        }
        CircularDependency { .. } | DeepImportChain { .. } => "circular-dependency",
        UnregisteredComponent { .. } | UnresolvedImport { .. } => "component-resolution",
        UndeclaredProp { .. } | MissingRequiredProp { .. } | PropTypeMismatch { .. } => {
            "props-validation"
        }
        UndefinedSlot { .. } => "slot-validation",
        ReactivityOutsideSetup { .. }
        | LifecycleOutsideSetup { .. }
        | WatcherOutsideSetup { .. }
        | DependencyInjectionOutsideSetup { .. }
        | ComposableOutsideSetup { .. }
        | SetupContextViolation { .. } => "setup-context",
        SpreadBreaksReactivity { .. }
        | ReassignmentBreaksReactivity { .. }
        | ValueExtractionBreaksReactivity { .. }
        | DestructuringBreaksReactivity { .. } => "reactivity-loss",
        ReactiveReferenceEscapes { .. } | ReactiveObjectMutatedAfterEscape { .. } => {
            "reference-escape"
        }
        CircularReactiveDependency { .. } => "circular-reactive",
        WatchMutationCanBeComputed { .. } => "watch-pattern",
        DomAccessWithoutNextTick { .. } => "dom-access",
        ComputedHasSideEffects { .. } => "computed-purity",
        ReactiveStateAtModuleScope { .. } => "module-scope",
        TemplateRefAccessedBeforeMount { .. } => "template-ref-timing",
        AsyncBoundaryCrossing { .. } => "async-boundary",
        InjectedAsyncMutationRace { .. } => "race-condition",
        ClosureCapturesReactive { .. } => "closure-capture",
        ObjectIdentityComparison { .. } => "object-identity",
        ReactiveStateExported { .. } => "state-export",
        ShallowReactiveDeepAccess { .. } => "shallow-reactive",
        ToRawMutation { .. } => "to-raw-mutation",
        EventListenerWithoutCleanup { .. } => "event-listener-cleanup",
        LifecycleHookWithoutCleanup { .. } => "lifecycle-cleanup",
        ArrayMutationNotTriggering { .. } => "array-mutation",
        PiniaGetterWithoutStoreToRefs { .. } => "pinia-store-refs",
        WatchEffectWithAsync { .. } => "watch-effect-async",
    }
}
