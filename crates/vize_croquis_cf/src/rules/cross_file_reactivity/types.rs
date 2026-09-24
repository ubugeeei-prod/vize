//! Type definitions for cross-file reactivity tracking.

use crate::diagnostics::DiagnosticSeverity;
use crate::registry::FileId;
use vize_carton::CompactString;

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

/// A flow of reactivity between files.
#[derive(Debug, Clone)]
pub struct ReactivityFlow {
    /// Source of the reactive value.
    pub source: ReactiveValueId,
    /// Target where it's consumed.
    pub target: ReactiveValueId,
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

/// A provide() call definition.
#[derive(Debug, Clone)]
pub(super) struct ProvideDefinition {
    pub(super) file_id: FileId,
    pub(super) key: CompactString,
    pub(super) key_identity: CompactString,
    pub(super) value_name: CompactString,
    pub(super) is_reactive: bool,
    pub(super) offset: u32,
}
