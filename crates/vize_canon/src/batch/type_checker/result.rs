use super::TypeCheckResult;

impl TypeCheckResult {
    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == 1)
    }

    /// Get the number of errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == 1).count()
    }

    /// Get the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == 2).count()
    }
}
