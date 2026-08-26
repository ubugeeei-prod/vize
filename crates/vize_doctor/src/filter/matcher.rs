use globset::{GlobBuilder, GlobMatcher};
use vize_s0::{String, ToCompactString};

use super::{DoctorFilterDimension, DoctorFilterError};

#[derive(Debug, Clone)]
pub(super) struct PatternSet {
    matchers: Vec<GlobMatcher>,
}

impl PatternSet {
    pub(super) fn compile(
        dimension: DoctorFilterDimension,
        patterns: &[String],
    ) -> Result<Self, DoctorFilterError> {
        let matchers = patterns
            .iter()
            .map(|pattern| compile_pattern(dimension, pattern))
            .collect::<Result<_, _>>()?;
        Ok(Self { matchers })
    }

    pub(super) fn is_unrestricted(&self) -> bool {
        self.matchers.is_empty()
    }

    pub(super) fn matches(&self, value: &str) -> bool {
        self.is_unrestricted() || self.matchers.iter().any(|matcher| matcher.is_match(value))
    }

    pub(super) fn matches_optional(&self, value: Option<&str>) -> bool {
        self.is_unrestricted() || value.is_some_and(|value| self.matches(value))
    }

    pub(super) fn matches_path(&self, path: &str) -> bool {
        if !path.contains('\\') {
            return self.matches(path);
        }
        self.matches(&path.replace('\\', "/"))
    }
}

fn compile_pattern(
    dimension: DoctorFilterDimension,
    pattern: &str,
) -> Result<GlobMatcher, DoctorFilterError> {
    if pattern.is_empty() {
        return Err(DoctorFilterError::new(
            dimension,
            pattern,
            "patterns must not be empty",
        ));
    }
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .backslash_escape(false)
        .build()
        .map(|glob| glob.compile_matcher())
        .map_err(|error| DoctorFilterError::new(dimension, pattern, error.to_compact_string()))
}
