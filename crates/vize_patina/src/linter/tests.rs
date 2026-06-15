use super::Linter;
use crate::LintPreset;
use vize_carton::{Allocator, ToCompactString};

mod basic;
mod directives;
mod jsx;
mod jsx_fallback;
mod no_top_level_ref;
mod sfc;
