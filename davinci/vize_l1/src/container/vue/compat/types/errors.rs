//! SFC error and warning payloads.

use serde::{Deserialize, Serialize};
use vize_l0::String;

use super::BlockLocation;

/// SFC error/warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfcError {
    /// Error message
    pub message: String,

    /// Error code
    #[serde(default)]
    pub code: Option<String>,

    /// Location
    #[serde(default)]
    pub loc: Option<BlockLocation>,
}
