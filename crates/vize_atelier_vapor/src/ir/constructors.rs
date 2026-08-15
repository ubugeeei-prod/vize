//! Arena constructors for the IR nodes that hold collections.
//!
//! Their vectors live in the compile arena so the whole IR graph is
//! `Drop`-free (Davinci P1-10) — which is why they cannot simply be `Default`.

use vize_carton::{Allocator, Vec};

use super::{EventModifiers, EventOptions, IRDynamicInfo};

impl<'a> IRDynamicInfo<'a> {
    /// Empty dynamic info. Arena-allocated so the IR stays `Drop`-free and
    /// tearing a deep lowering down costs nothing.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            flags: 0,
            children: Vec::new_in(&allocator),
            id: None,
        }
    }
}

impl<'a> EventModifiers<'a> {
    /// No modifiers, with both lists arena-allocated.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            keys: Vec::new_in(&allocator),
            non_keys: Vec::new_in(&allocator),
            options: EventOptions::default(),
        }
    }
}
