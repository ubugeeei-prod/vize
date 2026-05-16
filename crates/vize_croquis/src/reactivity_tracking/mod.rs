//! Ultra-strict Reactivity Tracking System.
//!
//! This module implements a Rust-inspired ownership and borrowing model for
//! Vue's reactivity system. It tracks reactive references with extreme precision,
//! detecting subtle bugs that would be missed by conventional linters.
//!
//! ## Design Philosophy
//!
//! Like Rust's borrow checker, this system tracks:
//! - **Ownership**: Which variable "owns" the reactive reference
//! - **Borrowing**: When reactive references are passed to functions
//! - **Lifetime**: When reactive references escape their intended scope
//! - **Moves**: When destructuring/spreading "moves" values out of reactive containers
//!
//! ## Detected Issues
//!
//! - Reactivity loss via destructuring (`const { a } = reactive({...})`)
//! - Reactivity loss via spread (`{ ...reactive({...}) }`)
//! - Reactivity loss via reassignment (`let x = reactive({}); x = {...}`)
//! - Ref value extraction without `.value` tracking
//! - Reactive reference escaping setup scope
//! - Closure capturing reactive references
//! - Implicit reference sharing through function parameters
//!
//! This module is split into:
//! - Core types and data structures (this file)
//! - [`tracker`]: The main [`ReactivityTracker`] implementation
//! - [`analysis`]: Reporting and markdown generation

mod analysis;
mod tracker;

use vize_carton::lsp::VueReactiveType;
use vize_carton::{CompactString, FxHashSet, SmallVec};

// Re-export the tracker (it is the primary public API)
pub use tracker::ReactivityTracker;

/// Unique identifier for a reactive binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReactiveBindingId(u32);

impl ReactiveBindingId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// How a reactive value was created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveOrigin {
    /// Created via ref()
    Ref,
    /// Created via shallowRef()
    ShallowRef,
    /// Created via reactive()
    Reactive,
    /// Created via shallowReactive()
    ShallowReactive,
    /// Created via readonly()
    Readonly,
    /// Created via shallowReadonly()
    ShallowReadonly,
    /// Created via computed()
    Computed,
    /// Created via toRef()
    ToRef,
    /// Created via toRefs()
    ToRefs,
    /// Injected via inject()
    Inject,
    /// From props (via defineProps)
    Props,
    /// From Pinia store
    PiniaStore,
    /// From composable function return
    ComposableReturn { composable_name: CompactString },
    /// Derived from another reactive source
    Derived { source: ReactiveBindingId },
    /// Unknown origin (conservative assumption: reactive)
    Unknown,
}

impl ReactiveOrigin {
    /// Get the Vue reactive type for this origin.
    pub fn reactive_type(&self) -> VueReactiveType {
        match self {
            Self::Ref | Self::ToRef | Self::Computed => VueReactiveType::Ref,
            Self::ShallowRef => VueReactiveType::ShallowRef,
            Self::Reactive | Self::ToRefs | Self::Props | Self::PiniaStore => {
                VueReactiveType::Reactive
            }
            Self::ShallowReactive => VueReactiveType::ShallowReactive,
            Self::Readonly => VueReactiveType::Readonly,
            Self::ShallowReadonly => VueReactiveType::ShallowReadonly,
            Self::Inject | Self::ComposableReturn { .. } | Self::Unknown => {
                VueReactiveType::Reactive
            }
            Self::Derived { .. } => VueReactiveType::Reactive,
        }
    }

    /// Check if this creates a deep reactive object.
    pub fn is_deep(&self) -> bool {
        !matches!(
            self,
            Self::ShallowRef | Self::ShallowReactive | Self::ShallowReadonly
        )
    }
}

/// State of a reactive binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingState {
    /// Binding is valid and reactive.
    Active,
    /// Reactivity was lost (e.g., via destructuring).
    ReactivityLost,
    /// Reference was moved/consumed.
    Moved,
    /// Reference escaped its scope.
    Escaped,
    /// Binding was reassigned to non-reactive value.
    Reassigned,
}

/// A tracked reactive binding.
#[derive(Debug, Clone)]
pub struct ReactiveBinding {
    /// Unique identifier.
    pub id: ReactiveBindingId,
    /// Variable name.
    pub name: CompactString,
    /// How it was created.
    pub origin: ReactiveOrigin,
    /// Current state.
    pub state: BindingState,
    /// Whether this is a `let` binding (can be reassigned).
    pub is_mutable: bool,
    /// Source location (start offset).
    pub start: u32,
    /// Source location (end offset).
    pub end: u32,
    /// Scope depth where this binding was created.
    pub scope_depth: u32,
    /// Whether `.value` was ever accessed (for refs).
    pub value_accessed: bool,
    /// Child bindings derived from this one (e.g., via toRefs).
    pub derived_bindings: SmallVec<[ReactiveBindingId; 4]>,
    /// Locations where this binding is used.
    pub use_sites: SmallVec<[UseSite; 8]>,
}

impl ReactiveBinding {
    /// Check if destructuring this binding would lose reactivity.
    pub fn loses_reactivity_on_destructure(&self) -> bool {
        self.origin
            .reactive_type()
            .loses_reactivity_on_destructure()
    }

    /// Check if spreading this binding would lose reactivity.
    pub fn loses_reactivity_on_spread(&self) -> bool {
        self.origin.reactive_type().loses_reactivity_on_spread()
    }

    /// Check if this is a ref type (needs .value access).
    pub fn is_ref_type(&self) -> bool {
        self.origin.reactive_type().is_ref()
    }
}

/// How a reactive binding is used.
#[derive(Debug, Clone)]
pub struct UseSite {
    /// Type of usage.
    pub kind: UseSiteKind,
    /// Source location.
    pub start: u32,
    pub end: u32,
}

/// Kind of use site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseSiteKind {
    /// Simple read: `foo`
    Read,
    /// Property access: `foo.bar`
    PropertyAccess { property: CompactString },
    /// Value access on ref: `foo.value`
    ValueAccess,
    /// Destructuring: `const { a } = foo`
    Destructure { extracted_props: Vec<CompactString> },
    /// Spread: `{ ...foo }`
    Spread,
    /// Passed as function argument: `fn(foo)`
    FunctionArg {
        fn_name: CompactString,
        arg_index: usize,
    },
    /// Returned from function: `return foo`
    Return,
    /// Assigned to variable: `bar = foo`
    Assignment { target: CompactString },
    /// Used in template expression.
    TemplateExpression,
    /// Reassignment: `foo = newValue`
    Reassignment,
    /// Captured in closure.
    ClosureCapture { closure_start: u32 },
    /// Passed to external API (window, localStorage, etc.)
    ExternalEscape { target: CompactString },
}

/// A reactivity violation detected by the tracker.
#[derive(Debug, Clone)]
pub struct ReactivityViolation {
    /// The binding that was violated.
    pub binding_id: ReactiveBindingId,
    /// Kind of violation.
    pub kind: ViolationKind,
    /// Location of the violation.
    pub start: u32,
    pub end: u32,
    /// Human-readable message.
    pub message: CompactString,
    /// Suggested fix.
    pub suggestion: Option<CompactString>,
    /// Severity level.
    pub severity: ViolationSeverity,
}

/// Kind of reactivity violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationKind {
    /// Destructuring reactive object loses reactivity.
    DestructuringLoss { extracted_props: Vec<CompactString> },
    /// Spreading reactive object loses reactivity.
    SpreadLoss,
    /// Reassigning reactive variable.
    Reassignment,
    /// Ref used without .value in non-template context.
    MissingValueAccess,
    /// Reactive reference escaping setup scope.
    ScopeEscape { escape_target: CompactString },
    /// Reactive reference captured in closure that may outlive component.
    UnsafeClosureCapture,
    /// Reactive object passed to external API without toRaw.
    ExternalMutation,
    /// Using reactive primitive in wrong context.
    WrongUnwrapContext,
    /// Pinia store destructured without storeToRefs.
    PiniaDestructure,
    /// Props destructured without toRefs.
    PropsDestructure,
    /// Inject result destructured.
    InjectDestructure,
    /// toRefs called on non-reactive object.
    ToRefsOnNonReactive,
    /// Double unwrap (.value.value or toValue(toValue(x))).
    DoubleUnwrap,
    /// Reactive assignment to const (logic error).
    ReactiveConst,
    /// Shallow reactive with deep mutation expectation.
    ShallowDeepMismatch,
}

/// Severity of a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    /// Definite bug that will cause runtime issues.
    Error,
    /// Likely bug or suspicious pattern.
    Warning,
    /// Code smell or potential issue.
    Info,
    /// Suggestion for improvement.
    Hint,
}

/// Scope for tracking reactive bindings.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ReactiveScope {
    /// Scope depth (0 = module level, 1 = setup, etc.)
    pub depth: u32,
    /// Bindings created in this scope.
    pub bindings: FxHashSet<ReactiveBindingId>,
    /// Whether this is a setup scope (where reactive APIs should be called).
    pub is_setup_scope: bool,
    /// Whether this is inside an async function.
    pub is_async: bool,
    /// Parent scope (if any).
    pub parent_scope: Option<u32>,
}
