//! Namespace shared by surface parsing and legacy ASTs.
use serde::{Deserialize, Serialize};

/// Namespace for elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Namespace {
    #[default]
    Html = 0,
    Svg = 1,
    MathMl = 2,
}
