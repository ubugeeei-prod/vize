//! Shared Options API options-object descriptor.
//!
//! [`OptionsDescriptor`] is croquis's single, authoritative view of an Options
//! API component's options object: the resolved options-object span, every
//! top-level option key (name + span), and the per-group members
//! (`props`/`inject`/`computed`/`methods`/`data`/`setup`) with the raw key span
//! each was declared at.
//!
//! It is built by the croquis script parser using the *same* `export default` /
//! `defineComponent(...)` / identifier-bound / `as`/`satisfies`/non-null/paren
//! resolution that drives template-binding collection, so downstream consumers
//! (patina lint rules today; canon/maestro later) no longer maintain their own
//! divergent options-object walkers.
//!
//! Spans are **program-relative** (relative to the script content the options
//! object was parsed from); consumers add their own block offset. Member names
//! and spans are recorded **verbatim** — no kebab→camel normalization and, for
//! array-form `props`/`inject` entries, the span covers the full string literal
//! including its quotes — so byte-for-byte diagnostics are preserved.

use vize_carton::CompactString;

/// The Options API group a member was declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionGroup {
    /// `props: ['a'] | { a: ... }`
    Props,
    /// `inject: ['a'] | { a: ... }`
    Inject,
    /// `computed: { a() {} }`
    Computed,
    /// `methods: { a() {} }`
    Methods,
    /// `data() { return { a: 1 } }`
    Data,
    /// `setup() { return { a: 1 } }`
    Setup,
}

impl OptionGroup {
    /// Lower-case option name (`"props"`, `"data"`, ...). Stable identifier used
    /// by consumers for human-readable labels.
    #[inline]
    pub fn label(self) -> &'static str {
        match self {
            OptionGroup::Props => "props",
            OptionGroup::Inject => "inject",
            OptionGroup::Computed => "computed",
            OptionGroup::Methods => "methods",
            OptionGroup::Data => "data",
            OptionGroup::Setup => "setup",
        }
    }
}

/// A single top-level option key (e.g. `data`, `computed`, `name`) and the span
/// of its key in the options object.
#[derive(Debug, Clone)]
pub struct OptionKey {
    /// Verbatim key name as written.
    pub name: CompactString,
    /// Program-relative start of the key.
    pub start: u32,
    /// Program-relative end of the key.
    pub end: u32,
}

/// A single member declared within a tracked option group, with the raw span it
/// should be reported at.
#[derive(Debug, Clone)]
pub struct OptionMember {
    /// Verbatim member name as written (no kebab→camel normalization).
    pub name: CompactString,
    /// Program-relative start of the member's key (full string literal,
    /// including quotes, for array-form `props`/`inject` entries).
    pub start: u32,
    /// Program-relative end of the member's key.
    pub end: u32,
    /// The group this member belongs to.
    pub group: OptionGroup,
}

/// Authoritative descriptor of a resolved Options API options object.
#[derive(Debug, Clone)]
pub struct OptionsDescriptor {
    /// Program-relative span of the resolved options object (`{ ... }`).
    pub options_start: u32,
    /// Program-relative end of the resolved options object.
    pub options_end: u32,
    /// Every top-level option key, in source order.
    pub option_keys: Vec<OptionKey>,
    /// Members of the tracked groups
    /// (`props`/`inject`/`computed`/`methods`/`data`/`setup`), in source order.
    pub members: Vec<OptionMember>,
}

impl OptionsDescriptor {
    /// Members belonging to a specific group.
    pub fn members_in(&self, group: OptionGroup) -> impl Iterator<Item = &OptionMember> {
        self.members
            .iter()
            .filter(move |member| member.group == group)
    }
}
