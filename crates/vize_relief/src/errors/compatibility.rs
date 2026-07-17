use super::{CompilerError, ErrorCode};

impl CompilerError {
    /// Returns true for parser recovery notices that are useful to compiler
    /// callers but should not surface as project diagnostics in lint or editor
    /// integrations. Vue accepts self-closing native elements in SFC templates,
    /// so the standard-mode rewrite must stay silent outside compilation.
    #[must_use]
    pub fn is_compatibility_notice(&self) -> bool {
        self.code == ErrorCode::ExtendPoint
            && self
                .message
                .starts_with("Invalid self-closing syntax on non-void HTML element")
    }
}

#[cfg(test)]
mod tests {
    use super::{CompilerError, ErrorCode};

    #[test]
    fn self_closing_html_rewrite_is_a_silent_compatibility_notice() {
        let notice = CompilerError::with_message(
            ErrorCode::ExtendPoint,
            "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.",
            None,
        );
        let duplicate = CompilerError::new(ErrorCode::DuplicateAttribute, None);
        let strict = CompilerError::with_message(
            ErrorCode::UnexpectedSolidusInTag,
            "Invalid self-closing syntax on non-void HTML element.",
            None,
        );

        assert!(notice.is_recoverable());
        assert!(notice.is_compatibility_notice());
        assert!(duplicate.is_recoverable());
        assert!(!duplicate.is_compatibility_notice());
        assert!(!strict.is_recoverable());
        assert!(!strict.is_compatibility_notice());
    }
}
