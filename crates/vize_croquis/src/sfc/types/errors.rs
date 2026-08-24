//! SFC error and warning payloads.

use serde::{Deserialize, Serialize};
use vize_carton::String;

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

impl From<vize_relief::CompilerError> for SfcError {
    fn from(err: vize_relief::CompilerError) -> Self {
        let mut code = vize_carton::String::default();
        use std::fmt::Write as _;
        let _ = write!(&mut code, "{:?}", err.code);
        Self {
            message: err.message,
            code: Some(code),
            loc: None,
        }
    }
}
