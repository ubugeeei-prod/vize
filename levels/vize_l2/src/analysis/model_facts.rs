//! Canonical product schema for L2 `ui.model` validation.
//!
//! These facts describe whether a canonical model binding is realizable, not
//! how a particular input dialect discovers the fault. The transitional Vue
//! pass still performs that validation while consumers can depend on this L2
//! schema directly.

/// Why one `ui.model` is unrealizable where it stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFault {
    /// The model value is an iteration or slot-scope binding.
    OnScope,
    /// A plain element's model carries an argument.
    ArgOnElement,
}

/// Per-`ui.model` validation facts, keyed by the binding op's page-order id.
///
/// Entries exist only for faulted models (sparse-table discipline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelFacts {
    /// The first failing check, in the validator's check order.
    pub fault: ModelFault,
}

// Facts cross compile boundaries with their artifact.
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ModelFault>();
    assert_owned::<ModelFacts>();
};

/// Both types are one-byte product vocabulary on every target.
const _: () = {
    assert!(core::mem::size_of::<ModelFault>() == 1);
    assert!(core::mem::size_of::<ModelFacts>() == 1);
};
