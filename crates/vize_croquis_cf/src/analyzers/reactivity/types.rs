use crate::registry::FileId;
use vize_carton::CompactString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactivityIssueKind {
    /// Destructuring a reactive object loses reactivity.
    DestructuredReactive {
        source_name: CompactString,
        destructured_props: Vec<CompactString>,
    },
    /// Destructuring a ref without .value loses reactivity.
    DestructuredRef { ref_name: CompactString },
    /// Reactive value passed to non-reactive context.
    ReactivityLost {
        value_name: CompactString,
        context: CompactString,
    },
    /// Ref used without .value in script.
    MissingValueAccess { ref_name: CompactString },
    /// toRef/toRefs should be used instead of destructuring.
    ShouldUseToRefs { source_name: CompactString },
    /// Reactive value assigned to plain variable.
    ReactiveToPlain {
        source_name: CompactString,
        target_name: CompactString,
    },
    /// Plain reactive snapshot passed through a function boundary.
    ReactiveSnapshotPassedToCall {
        source_name: CompactString,
        argument_name: CompactString,
        callee_name: CompactString,
    },
    /// Getter-backed context method extracted to a plain binding.
    GetterCallToPlain {
        context_name: CompactString,
        getter_name: CompactString,
        target_name: CompactString,
        callee_name: CompactString,
        source_name: CompactString,
    },
    /// storeToRefs should be used for Pinia store.
    ShouldUseStoreToRefs { store_name: CompactString },
    /// Computed without return statement.
    ComputedWithoutReturn { computed_name: CompactString },
    /// Watch source is not reactive.
    NonReactiveWatchSource { source_expression: CompactString },
    /// Prop passed to ref() which creates a copy.
    PropPassedToRef { prop_name: CompactString },
}

/// Information about a reactivity issue.
#[derive(Debug, Clone)]
pub struct ReactivityIssue {
    /// File where the issue occurs.
    pub file_id: FileId,
    /// Kind of issue.
    pub kind: ReactivityIssueKind,
    /// Offset in source.
    pub offset: u32,
    /// The reactive source involved.
    pub source: Option<CompactString>,
}

pub(super) struct InternalIssue {
    pub(super) kind: ReactivityIssueKind,
    pub(super) offset: u32,
    pub(super) end_offset: Option<u32>,
    pub(super) source: Option<CompactString>,
}
