//! Invocation-local SFC slot-scope policy without changing public option literals.
//!
//! The Vue emission policy owns the implementation; this ordinary child
//! module preserves its existing identity.

include!("../../dialect/vue/emit/budget/slot_scope.rs");
