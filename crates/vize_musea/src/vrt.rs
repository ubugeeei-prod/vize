//! VRT (Visual Regression Testing) configuration and types.
//!
//! This module provides configuration parsing and types for VRT integration.

mod config;
mod preset;

pub use config::{VrtConfig, VrtOptions, VrtThreshold};
pub use preset::{PRESET_VIEWPORTS, ViewportPreset};
