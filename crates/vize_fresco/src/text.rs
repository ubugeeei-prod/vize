//! Text processing module for CJK support.
//!
//! Provides Unicode-aware text handling:
//! - Character width calculation (East Asian Width)
//! - Grapheme segmentation
//! - Text wrapping

mod segment;
mod width;
mod wrap;

pub use segment::{SegmentedText, TextSegment, TextSegmentIter, segment, segment_vec};
pub use width::TextWidth;
pub use wrap::{TextWrap, WrapMode};
