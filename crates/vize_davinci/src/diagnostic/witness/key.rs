//! [`WitnessKey`] — a fact-group key, type-erased so a witness can carry it
//! across a compile boundary — and [`WitnessKeyed`], the round trip between a
//! group's typed key and it.

use crate::id::NodeId;
use vize_s0::String;

/// The key a [`WitnessLink`](super::WitnessLink) names its fact by.
///
/// A fact group's `Key` is any `Ord` type; a witness outlives the compile
/// that produced it and must be `'static` and comparable without the group's
/// type, so the key is erased to the shapes fact groups actually key on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessKey {
    /// The one fact of a group keyed by `()` — an artifact-level fact.
    Artifact,
    /// An S2 node.
    Node(NodeId),
    /// A numeric key: a `SymbolId`, a binding ordinal, an index.
    Index(u32),
    /// A named key: a binding, component or route name.
    Name(String),
}

/// A fact-group key that converts to and from a [`WitnessKey`].
///
/// The verifier (P4-6b) reads a link's key back into the group's typed key
/// with [`WitnessKeyed::from_witness_key`] and looks the fact up; the round
/// trip `from_witness_key(&k.to_witness_key()) == Some(k)` is the law every
/// implementation keeps.
pub trait WitnessKeyed: Sized {
    /// This key, erased.
    fn to_witness_key(&self) -> WitnessKey;

    /// The typed key `key` erases, or `None` when `key` has another shape.
    fn from_witness_key(key: &WitnessKey) -> Option<Self>;
}

impl WitnessKeyed for () {
    fn to_witness_key(&self) -> WitnessKey {
        WitnessKey::Artifact
    }

    fn from_witness_key(key: &WitnessKey) -> Option<Self> {
        matches!(key, WitnessKey::Artifact).then_some(())
    }
}

impl WitnessKeyed for NodeId {
    fn to_witness_key(&self) -> WitnessKey {
        WitnessKey::Node(*self)
    }

    fn from_witness_key(key: &WitnessKey) -> Option<Self> {
        match key {
            WitnessKey::Node(node) => Some(*node),
            WitnessKey::Artifact | WitnessKey::Index(_) | WitnessKey::Name(_) => None,
        }
    }
}

impl WitnessKeyed for u32 {
    fn to_witness_key(&self) -> WitnessKey {
        WitnessKey::Index(*self)
    }

    fn from_witness_key(key: &WitnessKey) -> Option<Self> {
        match key {
            WitnessKey::Index(index) => Some(*index),
            WitnessKey::Artifact | WitnessKey::Node(_) | WitnessKey::Name(_) => None,
        }
    }
}

impl WitnessKeyed for String {
    fn to_witness_key(&self) -> WitnessKey {
        WitnessKey::Name(self.clone())
    }

    fn from_witness_key(key: &WitnessKey) -> Option<Self> {
        match key {
            WitnessKey::Name(name) => Some(name.clone()),
            WitnessKey::Artifact | WitnessKey::Node(_) | WitnessKey::Index(_) => None,
        }
    }
}
