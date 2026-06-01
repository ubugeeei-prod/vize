//! Layout engine module using taffy.
//!
//! Provides Flexbox-based layout calculation:
//! - Taffy integration for layout computation
//! - Flexbox style conversion
//! - Rectangle/area calculations

mod engine;
mod flex;
mod rect;

pub use engine::LayoutEngine;
pub use flex::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, Edges, FlexDirection, FlexStyle,
    FlexWrap, Gap, Inset, JustifyContent, LengthPercentageAuto, Overflow, Position,
};
pub use rect::Rect;
