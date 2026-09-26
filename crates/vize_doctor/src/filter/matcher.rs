use globset::{Glob, GlobBuilder, GlobMatcher, GlobSet, GlobSetBuilder};
use vize_l0::{String, ToCompactString};

use super::{DoctorFilterDimension, DoctorFilterError};

#[derive(Debug, Clone)]
pub(super) enum PatternSet {
    Unrestricted,
    Single(GlobMatcher),
    Combined(GlobSet),
    // A combined regex can reach globset's aggregate limit even though each
    // independent pattern is valid. Preserve those existing specifications.
    Independent(Vec<GlobMatcher>),
}

impl PatternSet {
    pub(super) fn compile(
        dimension: DoctorFilterDimension,
        patterns: &[String],
    ) -> Result<Self, DoctorFilterError> {
        if patterns.is_empty() {
            return Ok(Self::Unrestricted);
        }
        if let [pattern] = patterns {
            return compile_pattern(dimension, pattern)
                .map(|glob| Self::Single(glob.compile_matcher()));
        }
        let globs: Vec<Glob> = patterns
            .iter()
            .map(|pattern| compile_pattern(dimension, pattern))
            .collect::<Result<_, _>>()?;
        let mut builder = GlobSetBuilder::new();
        for glob in &globs {
            builder.add(glob.clone());
        }
        Ok(match builder.build() {
            Ok(combined) => Self::Combined(combined),
            Err(_) => Self::Independent(globs.iter().map(Glob::compile_matcher).collect()),
        })
    }

    pub(super) fn is_unrestricted(&self) -> bool {
        matches!(self, Self::Unrestricted)
    }

    pub(super) fn matches(&self, value: &str) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Single(matcher) => matcher.is_match(value),
            Self::Combined(matcher) => matcher.is_match(value),
            Self::Independent(matchers) => matchers.iter().any(|matcher| matcher.is_match(value)),
        }
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
) -> Result<Glob, DoctorFilterError> {
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
        .map_err(|error| DoctorFilterError::new(dimension, pattern, error.to_compact_string()))
}

#[cfg(test)]
mod tests;
