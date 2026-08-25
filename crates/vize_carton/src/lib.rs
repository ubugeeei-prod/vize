// vize_carton defines and bridges std types, so it needs to use them directly.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

//! Carton - The artist's toolbox for Vize.
//!
//! This crate provides the foundational utilities and data structures for the Vize compiler,
//! much like a carton (artist's portfolio case) holds all the essential tools and materials
//! an artist needs for their work.
//!
//! # Modules
//!
//! - **Allocator**: Arena-based memory allocation for efficient AST construction
//! - **Shared utilities**: DOM configuration, optimization flags, and helper functions
//! - **Telegraph**: Generic message fan-out for report emitters
//!
//! # Example
//!
//! ```
//! use vize_carton::{Allocator, Box, Vec};
//!
//! let allocator = Allocator::default();
//! let allocator = &allocator;
//!
//! // Allocate a boxed value
//! let boxed = Box::new_in(42, &allocator);
//! assert_eq!(*boxed, 42);
//!
//! // Create a vector
//! let mut vec = Vec::new_in(&allocator);
//! vec.push(1);
//! vec.push(2);
//! vec.push(3);
//! assert_eq!(vec.len(), 3);
//! ```

// Allocator modules
mod allocator;
mod boxed;
mod clone_in;
mod vec;

// Shared modules
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod corsa_resolver;
pub mod dialect;
pub mod directive;
pub mod dom_tag_config;
pub mod expression_guard;
pub mod flags;
pub mod general;
pub mod hash;
pub mod i18n;
mod i18n_supplemental;
mod i18n_supplemental_extra;
mod i18n_supplemental_extra2;
pub mod interner;
pub mod line_index;
pub mod lsp;
pub mod path;
pub mod pool;
pub mod profiler;
pub mod recursion;
pub mod source_frame;
pub mod source_range;
pub mod span;
pub mod string_builder;
pub mod telegraph;

// Re-export allocator types
pub use allocator::{Allocator, ArenaStamp};
pub use boxed::Box;
pub use clone_in::CloneIn;
pub use vec::Vec;

// Re-export the recursion guard the recursive compiler passes wrap themselves in
pub use recursion::ensure_sufficient_stack;

// Re-export oxc's arena string builder: the build-then-freeze path for values
// assembled incrementally before they are frozen into an `&'a str` node field.
pub use oxc_allocator::StringBuilder;

// Re-export compact_str::CompactString for convenience
pub use compact_str::CompactString;
pub use compact_str::CompactString as String;
pub use compact_str::ToCompactString;

// Re-export smallvec for stack-optimized collections
pub use smallvec::{SmallVec, smallvec};

// Re-export bitflags for flag types
pub use bitflags::bitflags;

// Re-export rustc-hash for fast hash maps/sets
pub use rustc_hash::{FxHashMap, FxHashSet};

// Re-export phf for compile-time perfect hash functions
pub use phf::{Map as PhfMap, Set as PhfSet, phf_map, phf_set};

// Re-export the Davinci source model types (S0 coordinates)
pub use source_frame::{SourceBlock, SourceFrameError, SourceRoot};
pub use span::Span;

// Re-export shared utilities
pub use dom_tag_config::*;
pub use flags::*;
pub use general::*;
