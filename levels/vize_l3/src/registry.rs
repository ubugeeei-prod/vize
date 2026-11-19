//! L3 shared-decision registry.
//!
//! L3 consumes canonical L2 meaning and therefore registers no input dialect
//! variants. Backend policy remains in the decision and placement modules.

/// The current input-neutral L3 registry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DecisionHooks;

/// The complete shared-decision hook set.
pub const REGISTRY: DecisionHooks = DecisionHooks;
