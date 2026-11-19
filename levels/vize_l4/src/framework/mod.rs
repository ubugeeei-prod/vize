//! Framework-owned output policy.

pub use crate::targets::ts::vue::*;

// This disabled ordinary declaration keeps the physically Vue-owned DOM policy
// bodies in the standard rustfmt module graph without changing runtime identity.
#[cfg(any())]
mod vue {
    mod dom {
        mod budget;
        mod entry;
    }
}
