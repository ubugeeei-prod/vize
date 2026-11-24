#[cfg(creusot)]
use creusot_std::prelude::{Clone, ensures, logic};

#[cfg(creusot)]
use super::ReactivityClass::{PropsStable, Reactive, Static, Unstable};
use super::{EffectSet, ReactivityClass};

/// Where the binding value enters the component.
///
/// Under `cfg(creusot)` the discriminant derives (`Debug`, `PartialEq`, `Eq`,
/// `Hash`) are left out of the proof crate, which compares origins in logic
/// only. The product build keeps the original derive list.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq, Hash))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum BindingOrigin {
    Local,
    Prop,
    ProvideInject,
    TemplateRef,
}

impl BindingOrigin {
    /// Stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Prop => "prop",
            Self::ProvideInject => "provide-inject",
            Self::TemplateRef => "template-ref",
        }
    }

    /// Parse the stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"local" => Some(Self::Local),
            b"prop" => Some(Self::Prop),
            b"provide-inject" => Some(Self::ProvideInject),
            b"template-ref" => Some(Self::TemplateRef),
            _ => None,
        }
    }

    /// The P3-2 rule table: a local value is static, a prop is props-stable,
    /// and a provided, injected or template-ref value is reactive.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn demand(self) -> ReactivityClass {
        match self {
            Self::Local => Static,
            Self::Prop => PropsStable,
            Self::ProvideInject | Self::TemplateRef => Reactive,
        }
    }

    /// The best possible class for this origin.
    #[cfg_attr(creusot, ensures(result == self.demand()))]
    #[must_use]
    pub const fn class_floor(self) -> ReactivityClass {
        match self {
            Self::Local => ReactivityClass::Static,
            Self::Prop => ReactivityClass::PropsStable,
            Self::ProvideInject | Self::TemplateRef => ReactivityClass::Reactive,
        }
    }
}

/// Escape-analysis result for one binding.
///
/// Under `cfg(creusot)` the discriminant derives are left out of the proof
/// crate, as for [`BindingOrigin`].
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq, Hash))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum EscapeKind {
    None,
    Returned,
    Stored,
    Global,
}

impl EscapeKind {
    /// Stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Returned => "returned",
            Self::Stored => "stored",
            Self::Global => "global",
        }
    }

    /// Parse the stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"none" => Some(Self::None),
            b"returned" => Some(Self::Returned),
            b"stored" => Some(Self::Stored),
            b"global" => Some(Self::Global),
            _ => None,
        }
    }

    /// The P3-2 rule table: returned, stored and global escapes demote the
    /// class through props-stable, reactive and unstable.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn demand(self) -> ReactivityClass {
        match self {
            Self::None => Static,
            Self::Returned => PropsStable,
            Self::Stored => Reactive,
            Self::Global => Unstable,
        }
    }

    /// The lattice floor implied by escape analysis.
    #[cfg_attr(creusot, ensures(result == self.demand()))]
    #[must_use]
    pub const fn class_floor(self) -> ReactivityClass {
        match self {
            Self::None => ReactivityClass::Static,
            Self::Returned => ReactivityClass::PropsStable,
            Self::Stored => ReactivityClass::Reactive,
            Self::Global => ReactivityClass::Unstable,
        }
    }
}

/// The class the P3-2 rule table demands of a binding: the join of the
/// origin's, every effect's and the escape's demand, and `reactive` for a
/// provide/inject origin (the Effekt lexical/dynamic rule).
#[cfg(creusot)]
#[logic(open)]
pub fn demanded(origin: BindingOrigin, effects: EffectSet, escape: EscapeKind) -> ReactivityClass {
    let cap = match origin {
        BindingOrigin::ProvideInject => Reactive,
        _ => Static,
    };
    origin
        .demand()
        .join_logic(effects.floor())
        .join_logic(escape.demand())
        .join_logic(cap)
}

/// The naive declarative-rule evaluator on the value axis: join the floor
/// every premise demands, then apply the provide/inject cap.
///
/// It reads nothing but the three axes, so the Creusot proofs in
/// `davinci/vize_l3/src/lattice/creusot.rs` verify this exact source against
/// the rule table.
#[cfg_attr(creusot, ensures(result == demanded(origin, effects, escape)))]
#[must_use]
pub const fn classify(
    origin: BindingOrigin,
    effects: EffectSet,
    escape: EscapeKind,
) -> ReactivityClass {
    let class = origin
        .class_floor()
        .join(effects.class_floor())
        .join(escape.class_floor());
    match origin {
        BindingOrigin::ProvideInject => class.join(ReactivityClass::Reactive),
        _ => class,
    }
}
