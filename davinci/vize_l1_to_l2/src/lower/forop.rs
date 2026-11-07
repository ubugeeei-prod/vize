//! Building `ui.for`: the split's sub-slices admitted position by
//! position, the whole value escape-classified when it cannot split, and
//! the hygiene scope minted for every iteration region.
//!
//! Language policy owns the implementation under `lang/`; child declarations
//! remain here to preserve every existing module identity.

mod bindings;

#[cfg(test)]
mod tests;

include!("../lang/for_op.rs");
