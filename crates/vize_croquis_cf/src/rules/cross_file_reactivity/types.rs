//! Type definitions for cross-file reactivity tracking.

use crate::diagnostics::DiagnosticSeverity;
use crate::registry::FileId;
use vize_carton::{CompactString, SmallVec};
use vize_croquis::reactivity::ReactiveKind;

/// Unique identifier for a reactive value across the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactiveValueId {
    /// File where the value is defined.
    pub file_id: FileId,
    /// Name of the binding.
    pub name: CompactString,
    /// Declaration offset for disambiguation.
    pub offset: u32,
}

/// How a reactive value is exposed for cross-file usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveExposure {
    /// Exported from a module (composable return, named export).
    Export { export_name: CompactString },
    /// Provided via provide().
    Provide { key: CompactString },
    /// Passed as props to child component.
    Props {
        component_name: CompactString,
        prop_name: CompactString,
    },
    /// Exposed via Pinia store.
    PiniaStore {
        store_name: CompactString,
        property: CompactString,
    },
    /// Returned from composable function.
    ComposableReturn { composable_name: CompactString },
}

/// How a reactive value is consumed from another file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveConsumption {
    /// Imported from a module.
    Import {
        source_file: FileId,
        import_name: CompactString,
    },
    /// Injected via inject().
    Inject { key: CompactString },
    /// Received as props.
    Props { prop_name: CompactString },
    /// Used from Pinia store.
    PiniaStore { store_name: CompactString },
    /// Returned from composable call.
    ComposableCall {
        composable_name: CompactString,
        source_file: Option<FileId>,
    },
}

/// A tracked reactive value in the cross-file graph.
#[derive(Debug, Clone)]
pub struct CrossFileReactiveValue {
    /// Unique identifier.
    pub id: ReactiveValueId,
    /// Kind of reactive value (Ref, Reactive, Computed, etc.).
    pub kind: ReactiveKind,
    /// How it's exposed (if at all).
    pub exposures: SmallVec<[ReactiveExposure; 2]>,
    /// How it's consumed in other files.
    pub consumptions: SmallVec<[ReactiveConsumption; 2]>,
    /// Whether this value's reactivity was verified to flow correctly.
    pub reactivity_preserved: bool,
}

/// A flow of reactivity between files.
#[derive(Debug, Clone)]
pub struct ReactivityFlow {
    /// Source of the reactive value.
    pub source: ReactiveValueId,
    /// Target where it's consumed.
    pub target: ReactiveValueId,
    /// How the flow occurs.
    pub flow_kind: ReactivityFlowKind,
    /// Whether reactivity is preserved in this flow.
    pub preserved: bool,
    /// Reason if reactivity is lost.
    pub loss_reason: Option<ReactivityLossReason>,
}

/// Kind of reactivity flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactivityFlowKind {
    /// Composable export -> import.
    ComposableExport,
    /// Provide -> inject.
    ProvideInject,
    /// Parent props -> child props.
    PropsFlow,
    /// Pinia store -> consumer.
    StoreFlow,
    /// Direct module import.
    ModuleImport,
}

/// Reason why reactivity was lost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactivityLossReason {
    /// Destructured at consumption site.
    Destructured { props: Vec<CompactString> },
    /// Spread operator used.
    Spread,
    /// Assigned to non-reactive variable.
    NonReactiveAssignment,
    /// Passed through non-reactive intermediate.
    NonReactiveIntermediate { intermediate: CompactString },
    /// Value extracted without toRef/toRefs.
    DirectExtraction,
    /// Composable return destructured without toRefs.
    ComposableDestructure,
    /// Store destructured without storeToRefs.
    StoreDestructure,
    /// Inject result destructured.
    InjectDestructure,
}

/// Cross-file reactivity issue.
#[derive(Debug, Clone)]
pub struct CrossFileReactivityIssue {
    /// File where the issue is detected.
    pub file_id: FileId,
    /// Kind of issue.
    pub kind: CrossFileReactivityIssueKind,
    /// Offset in source.
    pub offset: u32,
    /// Related file (source of reactive value).
    pub related_file: Option<FileId>,
    /// Severity.
    pub severity: DiagnosticSeverity,
}

/// Kind of cross-file reactivity issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossFileReactivityIssueKind {
    /// Composable return value destructured.
    ComposableReturnDestructured {
        composable_name: CompactString,
        destructured_props: Vec<CompactString>,
    },
    /// Injected value destructured.
    InjectValueDestructured {
        key: CompactString,
        destructured_props: Vec<CompactString>,
    },
    /// Pinia store destructured without storeToRefs.
    StoreDestructured {
        store_name: CompactString,
        destructured_props: Vec<CompactString>,
    },
    /// Props destructured without toRefs.
    PropsDestructured {
        destructured_props: Vec<CompactString>,
    },
    /// Provide value is not reactive.
    NonReactiveProvide { key: CompactString },
    /// Reactive value lost in prop chain.
    ReactivityLostInPropChain {
        prop_name: CompactString,
        parent_component: CompactString,
    },
    /// Composable exports non-reactive value.
    ComposableExportsNonReactive {
        composable_name: CompactString,
        property: CompactString,
    },
    /// Ref passed where reactive object expected.
    RefReactiveTypeMismatch {
        expected: CompactString,
        actual: CompactString,
    },
    /// Reactive value escapes module scope unsafely.
    ReactiveEscapeUnsafe {
        value_name: CompactString,
        escape_target: CompactString,
    },
    /// Circular reactive dependency detected.
    CircularReactiveDependency { cycle: Vec<CompactString> },
    /// Stale closure captures reactive value.
    StaleClosureCapture {
        value_name: CompactString,
        closure_context: CompactString,
    },
}

/// Information about a composable function.
#[derive(Debug, Clone)]
pub(super) struct ComposableInfo {
    pub(super) name: CompactString,
    /// Returned reactive values.
    pub(super) reactive_returns: Vec<(CompactString, ReactiveKind)>,
    /// File where defined.
    pub(super) file_id: FileId,
    pub(super) offset: u32,
}

/// A provide() call definition.
#[derive(Debug, Clone)]
pub(super) struct ProvideDefinition {
    pub(super) file_id: FileId,
    pub(super) key: CompactString,
    pub(super) key_identity: CompactString,
    pub(super) value_name: CompactString,
    pub(super) is_reactive: bool,
    pub(super) reactive_kind: Option<ReactiveKind>,
    pub(super) offset: u32,
}
