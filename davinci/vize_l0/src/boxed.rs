//! Arena-allocated Box type.
//!
//! Re-exports [`oxc_allocator::Box`] for unified type usage across all crates.
//! Its const assertion rejects `Drop` payloads, which is what keeps every
//! arena-resident node in the compiler string-free (Davinci P1-10).

pub use oxc_allocator::Box;
