//! Input Method Editor (IME) support for CJK input.
//!
//! This module provides IME handling for Japanese, Chinese, and Korean input.

mod candidate;
pub mod platform;
mod preedit;
mod state;

pub use candidate::{Candidate, CandidateList};
pub use preedit::{Preedit, PreeditSegment, SegmentStyle};
pub use state::{ImeEvent, ImeMode, ImeState};
