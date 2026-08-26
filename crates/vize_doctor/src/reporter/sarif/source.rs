//! Explicit source injection and byte-to-text coordinate indexing.

use std::{error::Error, fmt};

use vize_s0::String;

/// One analyzed UTF-8 source supplied explicitly to the SARIF reporter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SarifSource<'source> {
    path: &'source str,
    text: &'source str,
}

impl<'source> SarifSource<'source> {
    /// Creates a borrowed source. Validation occurs in
    /// [`super::SarifReporter::with_sources`].
    pub const fn new(path: &'source str, text: &'source str) -> Self {
        Self { path, text }
    }

    /// Returns the workspace-relative source path.
    pub const fn path(self) -> &'source str {
        self.path
    }

    /// Returns the exact analyzed UTF-8 text.
    pub const fn text(self) -> &'source str {
        self.text
    }
}

/// Behavior when a report location has no injected source text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SarifMissingSourcePolicy {
    /// Reject the report before writing any bytes. This is the default.
    #[default]
    Reject,
    /// Emit the artifact URI but omit its text region explicitly.
    ArtifactOnly,
}

/// Invalid explicit source configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SarifSourceError {
    path: String,
    reason: &'static str,
}

impl SarifSourceError {
    pub(super) fn invalid(path: &str, reason: &'static str) -> Self {
        Self {
            path: path.into(),
            reason,
        }
    }

    pub(super) fn duplicate(path: &str) -> Self {
        Self::invalid(path, "the source path was supplied more than once")
    }

    /// Returns the rejected source path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the stable rejection reason.
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

impl fmt::Display for SarifSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid SARIF source {}: {}",
            self.path, self.reason
        )
    }
}

impl Error for SarifSourceError {}

pub(super) struct IndexedSource<'source> {
    text: &'source str,
    line_starts: Vec<u32>,
}

impl<'source> IndexedSource<'source> {
    pub(super) fn new(source: SarifSource<'source>) -> Result<Self, SarifSourceError> {
        validate_path(source.path)?;
        if source.text.len() > u32::MAX as usize {
            return Err(SarifSourceError::invalid(
                source.path,
                "the UTF-8 source is larger than the Doctor offset space",
            ));
        }
        let mut line_starts = vec![0];
        line_starts.extend(
            source
                .text
                .bytes()
                .enumerate()
                .filter(|(_, byte)| *byte == b'\n')
                .map(|(index, _)| index as u32 + 1),
        );
        Ok(Self {
            text: source.text,
            line_starts,
        })
    }

    pub(super) fn position(&self, offset: u32) -> Option<(u32, u32)> {
        let offset = offset as usize;
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return None;
        }
        let line_index = self
            .line_starts
            .partition_point(|start| *start as usize <= offset)
            - 1;
        let line_start = self.line_starts[line_index] as usize;
        let column = self.text[line_start..offset].chars().count() as u32 + 1;
        Some((line_index as u32 + 1, column))
    }
}

pub(super) fn validate_path(path: &str) -> Result<(), SarifSourceError> {
    if path.is_empty() {
        return Err(SarifSourceError::invalid(path, "the path is empty"));
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(SarifSourceError::invalid(
            path,
            "the path must be workspace-relative",
        ));
    }
    if path.contains('\\') {
        return Err(SarifSourceError::invalid(
            path,
            "the path must use slash separators",
        ));
    }
    if path.chars().any(char::is_control) {
        return Err(SarifSourceError::invalid(
            path,
            "the path contains a control character",
        ));
    }
    if path
        .split('/')
        .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err(SarifSourceError::invalid(
            path,
            "the path is not lexically normalized",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn positions_are_one_based_unicode_code_points() {
        let source = IndexedSource::new(SarifSource::new("src/a.vue", "α\r\n東京🙂")).unwrap();

        assert_eq!(source.position(0), Some((1, 1)));
        assert_eq!(source.position("α\r\n".len() as u32), Some((2, 1)));
        assert_eq!(source.position("α\r\n東京".len() as u32), Some((2, 3)));
        assert_eq!(source.position("α\r\n東京🙂".len() as u32), Some((2, 4)));
        assert_eq!(source.position(1), None);
    }
}
