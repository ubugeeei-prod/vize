use vize_croquis_cf::{CrossFileDiagnosticKind as Kind, DiagnosticSeverity as SourceSeverity};

use crate::{
    DoctorCategory, EvidenceKind, FindingAssessment, FindingConfidence, FindingImpact,
    FindingSeverity, HealthPenalty,
};

pub(super) struct RuleProfile {
    pub category: DoctorCategory,
    pub evidence: EvidenceKind,
    pub assessment: FindingAssessment,
}

pub(super) fn profile_for(kind: &Kind, severity: SourceSeverity) -> RuleProfile {
    let category = category_for(kind);
    let evidence = evidence_for(kind);
    let confidence = confidence_for(kind);
    let impact = impact_for(kind, severity);
    let doctor_severity = match severity {
        SourceSeverity::Error => FindingSeverity::Error,
        SourceSeverity::Warning => FindingSeverity::Warning,
        SourceSeverity::Info | SourceSeverity::Hint => FindingSeverity::Notice,
    };
    let points = penalty_points(doctor_severity, impact);

    RuleProfile {
        category,
        evidence,
        assessment: FindingAssessment::new(
            doctor_severity,
            confidence,
            impact,
            HealthPenalty::new(points, penalty_reason(doctor_severity, impact)),
        ),
    }
}

fn category_for(kind: &Kind) -> DoctorCategory {
    match kind {
        Kind::DuplicateElementId { .. } | Kind::NonUniqueIdInLoop { .. } => {
            DoctorCategory::Accessibility
        }
        Kind::DeepImportChain { .. }
        | Kind::WatchMutationCanBeComputed { .. }
        | Kind::ComputedHasSideEffects { .. } => DoctorCategory::Performance,
        Kind::UnusedFallthroughAttrs { .. }
        | Kind::InheritAttrsDisabledUnused
        | Kind::UnusedEmit { .. }
        | Kind::UnusedProvide { .. }
        | Kind::CircularDependency { .. }
        | Kind::ReactiveReferenceEscapes { .. }
        | Kind::ClosureCapturesReactive { .. }
        | Kind::ReactiveStateExported { .. } => DoctorCategory::Maintainability,
        Kind::BrowserApiInSsr { .. }
        | Kind::AsyncWithoutSuspense { .. }
        | Kind::HydrationMismatchRisk { .. }
        | Kind::UncaughtErrorBoundary
        | Kind::MissingSuspenseBoundary
        | Kind::SuspenseWithoutFallback
        | Kind::LifecycleHookWithoutCleanup { .. }
        | Kind::WatcherOutsideSetup { .. }
        | Kind::AsyncBoundaryCrossing { .. }
        | Kind::InjectedAsyncMutationRace { .. }
        | Kind::EventListenerWithoutCleanup { .. }
        | Kind::WatchEffectWithAsync { .. } => DoctorCategory::ProductionReadiness,
        Kind::MultiRootMissingAttrs
        | Kind::UndeclaredEmit { .. }
        | Kind::UnmatchedEventListener { .. }
        | Kind::UnhandledEvent { .. }
        | Kind::EventModifierIssue { .. }
        | Kind::UnmatchedInject { .. }
        | Kind::ProvideInjectTypeMismatch { .. }
        | Kind::ProvideInjectWithoutSymbol { .. }
        | Kind::NonReactiveProvideValue { .. }
        | Kind::UnregisteredComponent { .. }
        | Kind::UnresolvedImport { .. }
        | Kind::UndeclaredProp { .. }
        | Kind::MissingRequiredProp { .. }
        | Kind::PropTypeMismatch { .. }
        | Kind::UndefinedSlot { .. }
        | Kind::ReactivityOutsideSetup { .. }
        | Kind::LifecycleOutsideSetup { .. }
        | Kind::DependencyInjectionOutsideSetup { .. }
        | Kind::ComposableOutsideSetup { .. }
        | Kind::SpreadBreaksReactivity { .. }
        | Kind::ReassignmentBreaksReactivity { .. }
        | Kind::ValueExtractionBreaksReactivity { .. }
        | Kind::DestructuringBreaksReactivity { .. }
        | Kind::ReactiveObjectMutatedAfterEscape { .. }
        | Kind::CircularReactiveDependency { .. }
        | Kind::DomAccessWithoutNextTick { .. }
        | Kind::ReactiveStateAtModuleScope { .. }
        | Kind::TemplateRefAccessedBeforeMount { .. }
        | Kind::ObjectIdentityComparison { .. }
        | Kind::ShallowReactiveDeepAccess { .. }
        | Kind::ToRawMutation { .. }
        | Kind::ArrayMutationNotTriggering { .. }
        | Kind::PiniaGetterWithoutStoreToRefs { .. }
        | Kind::SetupContextViolation { .. } => DoctorCategory::Correctness,
    }
}

fn evidence_for(kind: &Kind) -> EvidenceKind {
    match kind {
        Kind::CircularDependency { .. } | Kind::DeepImportChain { .. } => EvidenceKind::BuildGraph,
        Kind::ProvideInjectTypeMismatch { .. } | Kind::PropTypeMismatch { .. } => {
            EvidenceKind::Type
        }
        Kind::UnusedFallthroughAttrs { .. }
        | Kind::InheritAttrsDisabledUnused
        | Kind::MultiRootMissingAttrs
        | Kind::UndeclaredEmit { .. }
        | Kind::UnusedEmit { .. }
        | Kind::UnmatchedEventListener { .. }
        | Kind::UnhandledEvent { .. }
        | Kind::EventModifierIssue { .. }
        | Kind::DuplicateElementId { .. }
        | Kind::NonUniqueIdInLoop { .. }
        | Kind::AsyncWithoutSuspense { .. }
        | Kind::UnregisteredComponent { .. }
        | Kind::UndeclaredProp { .. }
        | Kind::MissingRequiredProp { .. }
        | Kind::UndefinedSlot { .. } => EvidenceKind::Component,
        Kind::UnmatchedInject { .. }
        | Kind::UnusedProvide { .. }
        | Kind::ProvideInjectWithoutSymbol { .. }
        | Kind::NonReactiveProvideValue { .. }
        | Kind::ReactivityOutsideSetup { .. }
        | Kind::WatcherOutsideSetup { .. }
        | Kind::DependencyInjectionOutsideSetup { .. }
        | Kind::ComposableOutsideSetup { .. }
        | Kind::SpreadBreaksReactivity { .. }
        | Kind::ReassignmentBreaksReactivity { .. }
        | Kind::ValueExtractionBreaksReactivity { .. }
        | Kind::DestructuringBreaksReactivity { .. }
        | Kind::ReactiveReferenceEscapes { .. }
        | Kind::ReactiveObjectMutatedAfterEscape { .. }
        | Kind::CircularReactiveDependency { .. }
        | Kind::WatchMutationCanBeComputed { .. }
        | Kind::ComputedHasSideEffects { .. }
        | Kind::ReactiveStateAtModuleScope { .. }
        | Kind::AsyncBoundaryCrossing { .. }
        | Kind::InjectedAsyncMutationRace { .. }
        | Kind::ClosureCapturesReactive { .. }
        | Kind::ObjectIdentityComparison { .. }
        | Kind::ReactiveStateExported { .. }
        | Kind::ShallowReactiveDeepAccess { .. }
        | Kind::ToRawMutation { .. }
        | Kind::ArrayMutationNotTriggering { .. }
        | Kind::PiniaGetterWithoutStoreToRefs { .. }
        | Kind::WatchEffectWithAsync { .. }
        | Kind::SetupContextViolation { .. } => EvidenceKind::Reactivity,
        Kind::BrowserApiInSsr { .. }
        | Kind::HydrationMismatchRisk { .. }
        | Kind::UncaughtErrorBoundary
        | Kind::MissingSuspenseBoundary
        | Kind::SuspenseWithoutFallback
        | Kind::UnresolvedImport { .. }
        | Kind::LifecycleOutsideSetup { .. }
        | Kind::LifecycleHookWithoutCleanup { .. }
        | Kind::DomAccessWithoutNextTick { .. }
        | Kind::TemplateRefAccessedBeforeMount { .. }
        | Kind::EventListenerWithoutCleanup { .. } => EvidenceKind::ControlFlow,
    }
}

fn confidence_for(kind: &Kind) -> FindingConfidence {
    match kind {
        Kind::EventModifierIssue { .. }
        | Kind::NonUniqueIdInLoop { .. }
        | Kind::BrowserApiInSsr { .. }
        | Kind::HydrationMismatchRisk { .. }
        | Kind::DeepImportChain { .. }
        | Kind::ReactiveReferenceEscapes { .. }
        | Kind::ClosureCapturesReactive { .. } => FindingConfidence::Medium,
        Kind::UnusedFallthroughAttrs { .. }
        | Kind::UnusedEmit { .. }
        | Kind::UnusedProvide { .. }
        | Kind::UnhandledEvent { .. }
        | Kind::AsyncWithoutSuspense { .. }
        | Kind::MissingSuspenseBoundary
        | Kind::SuspenseWithoutFallback
        | Kind::NonReactiveProvideValue { .. }
        | Kind::ReactiveObjectMutatedAfterEscape { .. }
        | Kind::WatchMutationCanBeComputed { .. }
        | Kind::AsyncBoundaryCrossing { .. }
        | Kind::InjectedAsyncMutationRace { .. }
        | Kind::ObjectIdentityComparison { .. }
        | Kind::PiniaGetterWithoutStoreToRefs { .. }
        | Kind::WatchEffectWithAsync { .. } => FindingConfidence::High,
        _ => FindingConfidence::Certain,
    }
}

fn impact_for(kind: &Kind, severity: SourceSeverity) -> FindingImpact {
    match kind {
        Kind::HydrationMismatchRisk { .. }
        | Kind::UncaughtErrorBoundary
        | Kind::CircularReactiveDependency { .. }
        | Kind::InjectedAsyncMutationRace { .. }
        | Kind::EventListenerWithoutCleanup { .. } => FindingImpact::High,
        Kind::UnusedFallthroughAttrs { .. }
        | Kind::InheritAttrsDisabledUnused
        | Kind::UnusedEmit { .. }
        | Kind::UnusedProvide { .. }
        | Kind::DeepImportChain { .. }
        | Kind::ReactiveReferenceEscapes { .. }
        | Kind::ClosureCapturesReactive { .. } => FindingImpact::Low,
        _ if severity == SourceSeverity::Error => FindingImpact::High,
        _ => FindingImpact::Medium,
    }
}

const fn penalty_points(severity: FindingSeverity, impact: FindingImpact) -> u8 {
    match (severity, impact) {
        (FindingSeverity::Error, FindingImpact::Critical | FindingImpact::High) => 30,
        (FindingSeverity::Error, FindingImpact::Medium) => 20,
        (FindingSeverity::Error, FindingImpact::Low) => 10,
        (FindingSeverity::Warning, FindingImpact::Critical | FindingImpact::High) => 18,
        (FindingSeverity::Warning, FindingImpact::Medium) => 12,
        (FindingSeverity::Warning, FindingImpact::Low) => 6,
        (FindingSeverity::Notice, FindingImpact::Critical | FindingImpact::High) => 8,
        (FindingSeverity::Notice, FindingImpact::Medium) => 5,
        (FindingSeverity::Notice, FindingImpact::Low) => 2,
    }
}

const fn penalty_reason(severity: FindingSeverity, impact: FindingImpact) -> &'static str {
    match (severity, impact) {
        (FindingSeverity::Error, FindingImpact::Critical | FindingImpact::High) => {
            "High-impact whole-project failure"
        }
        (FindingSeverity::Error, _) => "Whole-project failure",
        (FindingSeverity::Warning, FindingImpact::Critical | FindingImpact::High) => {
            "High-impact whole-project risk"
        }
        (FindingSeverity::Warning, _) => "Whole-project risk",
        (FindingSeverity::Notice, _) => "Whole-project improvement opportunity",
    }
}

pub(super) const fn failure_scenario(category: DoctorCategory) -> &'static str {
    match category {
        DoctorCategory::Correctness => {
            "A reachable application path violates its declared behavior."
        }
        DoctorCategory::Accessibility => {
            "An interaction or semantic relationship can fail for an assistive user."
        }
        DoctorCategory::Performance => {
            "A reachable application path performs avoidable work or retains avoidable resources."
        }
        DoctorCategory::Maintainability => {
            "A graph relationship makes application behavior harder to change safely."
        }
        DoctorCategory::Security => {
            "A reachable trust boundary can be crossed without its required guard."
        }
        DoctorCategory::ProductionReadiness => {
            "A production lifecycle path can fail without its required boundary or cleanup."
        }
    }
}
