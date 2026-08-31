use super::{CompilerError, ErrorCode};

impl CompilerError {
    /// Returns true for parser recovery notices that are useful to compiler
    /// callers but should not surface as project diagnostics in lint or editor
    /// integrations. Vue accepts self-closing native elements in SFC templates,
    /// so the standard-mode rewrite must stay silent outside compilation.
    #[must_use]
    pub fn is_compatibility_notice(&self) -> bool {
        if self.code != ErrorCode::ExtendPoint {
            return false;
        }
        self.message
            .starts_with("Invalid self-closing syntax on non-void HTML element")
            || self
                .message
                .starts_with("Nested anchor start tag closed the previous anchor")
            || self
                .message
                .starts_with("Nested button start tag closed the previous button")
            || self
                .message
                .starts_with("HTML tree construction ignored this end tag because the element was already closed before a nested start tag")
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

    #[test]
    fn nested_anchor_and_button_tree_recovery_are_compatibility_notices() {
        for message in [
            "Nested anchor start tag closed the previous anchor before inserting the new one.",
            "Nested button start tag closed the previous button before inserting the new one.",
            "HTML tree construction ignored this end tag because the element was already closed before a nested start tag.",
        ] {
            let notice = CompilerError::with_message(ErrorCode::ExtendPoint, message, None);
            assert!(notice.is_recoverable(), "{message}");
            assert!(notice.is_compatibility_notice(), "{message}");
        }
    }

    #[test]
    fn unrelated_tree_recovery_is_not_a_compatibility_notice() {
        let deep = CompilerError::with_message(
            ErrorCode::ExtendPoint,
            "Element nesting is too deep.",
            None,
        );
        let invalid = CompilerError::new(ErrorCode::InvalidEndTag, None);

        assert!(!deep.is_recoverable());
        assert!(!deep.is_compatibility_notice());
        assert!(!invalid.is_recoverable());
        assert!(!invalid.is_compatibility_notice());
    }
}
