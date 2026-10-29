//! Bounded host metadata primitives shared by distinct CLI admission policies.
//!
//! Keep a single ProbeIo/runtime attempt per project invocation when consumers
//! compose. These facts do not themselves admit Vue-provider or render policy,
//! and must never be retained as a global completeness witness.

pub(crate) mod authority;
pub(crate) mod io;
pub(crate) mod rc;
