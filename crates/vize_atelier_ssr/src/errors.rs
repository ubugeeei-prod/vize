//! SSR-specific compiler errors.

use serde::{Deserialize, Serialize};

/// SSR-specific error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SsrErrorCode {
    /// Unsafe attribute name that could cause XSS
    UnsafeAttrName = 65,
    /// Teleport requires a "to" prop
    NoTeleportTarget = 66,
    /// Invalid AST node encountered during SSR transform
    InvalidAstNode = 67,
}

impl SsrErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            Self::UnsafeAttrName => "Unsafe attribute name for SSR.",
            Self::NoTeleportTarget => "Missing required 'to' prop on <Teleport>.",
            Self::InvalidAstNode => "Invalid AST node encountered during SSR transform.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SsrErrorCode;

    #[test]
    fn test_error_messages() {
        assert!(!SsrErrorCode::UnsafeAttrName.message().is_empty());
        assert!(!SsrErrorCode::NoTeleportTarget.message().is_empty());
        assert!(!SsrErrorCode::InvalidAstNode.message().is_empty());
    }
}
