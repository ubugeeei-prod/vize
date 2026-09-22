//! Interface facets and the errors of building or recording a summary.
//!
//! A facet is one α group. An S3 code shape is not a facet, and
//! [`Facet::from_alpha_group`] refuses any group that is not one of these
//! six, so a body cannot be named as a summary entry.

use vize_s0::String;

/// α schema versions the summary reads. Bump one when that page's meaning
/// changes; the bump moves every fingerprint of that facet and refuses a
/// page still stamped with the old version.
pub mod schema {
    /// The summary folio. Bump when the page grammar changes.
    pub const SUMMARY: u16 = 1;
    /// `component-signature`: component name → type-parameter spelling.
    pub const SIGNATURE: u16 = 1;
    /// `prop-types`: prop name → type spelling.
    pub const PROPS: u16 = 1;
    /// `emit-types`: emit name → payload type spelling.
    pub const EMITS: u16 = 1;
    /// `slot-types`: slot name → scope type spelling.
    pub const SLOTS: u16 = 1;
    /// `reactivity-classes`: exported binding → reactivity class.
    pub const REACTIVITY: u16 = 1;
    /// `component-references`: resolved component identity → export name.
    pub const COMPONENTS: u16 = 1;
}

/// One interface α group. Not an S3 code shape — there is no such variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Facet {
    /// The component's exported name and type parameters.
    Signature = 0,
    /// One prop's type.
    Prop = 1,
    /// One emit's payload type.
    Emit = 2,
    /// One slot's scope type.
    Slot = 3,
    /// The reactivity class of one exported binding.
    Reactivity = 4,
    /// One referenced component, keyed by its resolved identity.
    Component = 5,
}

impl Facet {
    /// Every facet, in folio order.
    pub const ALL: [Self; 6] = [
        Self::Signature,
        Self::Prop,
        Self::Emit,
        Self::Slot,
        Self::Reactivity,
        Self::Component,
    ];

    /// The α group name.
    #[must_use]
    pub const fn group(self) -> &'static str {
        match self {
            Self::Signature => "component-signature",
            Self::Prop => "prop-types",
            Self::Emit => "emit-types",
            Self::Slot => "slot-types",
            Self::Reactivity => "reactivity-classes",
            Self::Component => "component-references",
        }
    }

    /// The α schema this summary reads for the group.
    #[must_use]
    pub const fn schema(self) -> u16 {
        match self {
            Self::Signature => schema::SIGNATURE,
            Self::Prop => schema::PROPS,
            Self::Emit => schema::EMITS,
            Self::Slot => schema::SLOTS,
            Self::Reactivity => schema::REACTIVITY,
            Self::Component => schema::COMPONENTS,
        }
    }

    /// The facet named `group`, ignoring schema.
    #[must_use]
    pub fn from_group_name(group: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|facet| facet.group() == group)
    }

    /// The facet of an α group at `schema`.
    ///
    /// # Errors
    ///
    /// [`SummaryError::NotInterface`] when `group` is not an interface α
    /// group (expression bodies and S3 code shapes land here), and
    /// [`SummaryError::Schema`] when the group's schema is not the one this
    /// summary reads.
    pub fn from_alpha_group(group: &str, schema: u16) -> Result<Self, SummaryError> {
        let Some(facet) = Self::from_group_name(group) else {
            return Err(SummaryError::NotInterface {
                group: String::from(group),
            });
        };
        if schema != facet.schema() {
            return Err(SummaryError::Schema {
                group: String::from(group),
                expected: facet.schema(),
                found: schema,
            });
        }
        Ok(facet)
    }
}

/// Why a summary could not be built, or a use could not be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryError {
    /// The declaration name is empty or contains whitespace or `=`.
    BadName { facet: Facet, name: String },
    /// The contract contains a newline, so it is not one folio line.
    BadContract { facet: Facet, name: String },
    /// Two entries of `facet` share `name`.
    Duplicate { facet: Facet, name: String },
    /// A usage named a declaration the summary does not export.
    Unknown { facet: Facet, name: String },
    /// `group` is not an interface α group.
    NotInterface { group: String },
    /// `group` is an interface group at a schema this summary does not read.
    Schema {
        group: String,
        expected: u16,
        found: u16,
    },
    /// A usage recorded no consumer name.
    EmptyConsumer,
    /// One usage recorded the same declaration twice.
    DuplicateUse { facet: Facet, name: String },
}
