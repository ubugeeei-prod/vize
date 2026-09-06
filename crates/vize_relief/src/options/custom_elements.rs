//! Declarative custom-element tag matching.

use std::vec::Vec;
use vize_s0::String;

/// Declarative matcher for tags that should compile as custom elements.
///
/// This is the schema-friendly counterpart to Vue's `isCustomElement`
/// predicate. Patterns are case-sensitive tag globs where `*` matches any
/// substring, so `Tres*` matches `TresMesh` and `three-*` matches `three-mesh`.
#[derive(Debug, Clone, Default)]
pub struct CustomElementMatcher {
    patterns: Vec<String>,
    predicate: Option<fn(&str) -> bool>,
}

impl CustomElementMatcher {
    /// Create a matcher from declarative tag patterns.
    #[must_use]
    pub fn from_patterns(patterns: Vec<String>) -> Self {
        Self {
            patterns,
            predicate: None,
        }
    }

    /// Create a matcher from a static Rust predicate.
    #[must_use]
    pub fn from_static_predicate(predicate: fn(&str) -> bool) -> Self {
        Self {
            patterns: Vec::new(),
            predicate: Some(predicate),
        }
    }

    /// Return configured declarative patterns.
    #[must_use]
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }

    /// Return patterns only when this matcher has no opaque predicate branch.
    #[must_use]
    pub fn projectable_patterns(&self) -> Option<&[String]> {
        (!self.has_static_predicate()).then_some(self.patterns())
    }

    /// Whether the matcher includes an opaque static predicate.
    #[must_use]
    pub fn has_static_predicate(&self) -> bool {
        self.predicate.is_some()
    }

    /// Whether no pattern or predicate can match.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty() && self.predicate.is_none()
    }

    /// Whether `tag` is configured as a custom element.
    #[must_use]
    pub fn matches(&self, tag: &str) -> bool {
        self.predicate.is_some_and(|predicate| predicate(tag))
            || self
                .patterns
                .iter()
                .any(|pattern| tag_pattern_matches(pattern.as_str(), tag))
    }
}

fn tag_pattern_matches(pattern: &str, tag: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if pattern.bytes().all(|byte| byte == b'*') {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == tag;
    }

    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');
    let mut position = 0;
    let mut matched_any = false;

    for (index, part) in pattern
        .split('*')
        .filter(|part| !part.is_empty())
        .enumerate()
    {
        matched_any = true;
        if index == 0 && !starts_with_wildcard {
            if !tag[position..].starts_with(part) {
                return false;
            }
            position += part.len();
            continue;
        }

        let Some(found) = tag[position..].find(part) else {
            return false;
        };
        position += found + part.len();
    }

    if !matched_any {
        return false;
    }

    if !ends_with_wildcard
        && let Some(last_part) = pattern.rsplit('*').find(|part| !part.is_empty())
    {
        return tag.ends_with(last_part);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::CustomElementMatcher;
    use vize_s0::String;

    fn is_tres(tag: &str) -> bool {
        tag.starts_with("Tres")
    }

    #[test]
    fn custom_element_matcher_supports_exact_and_wildcard_patterns() {
        let matcher = CustomElementMatcher::from_patterns(vec![
            String::from("Tres*"),
            String::from("primitive"),
            String::from("three-*"),
        ]);

        assert!(matcher.matches("TresMesh"));
        assert!(matcher.matches("primitive"));
        assert!(matcher.matches("three-buffer-geometry"));
        assert!(!matcher.matches("MyComponent"));
        assert!(!matcher.matches("NestedTresMesh"));
    }

    #[test]
    fn custom_element_matcher_supports_leading_and_middle_wildcards() {
        let matcher = CustomElementMatcher::from_patterns(vec![
            String::from("*-mesh"),
            String::from("Tres*Material"),
        ]);

        assert!(matcher.matches("three-mesh"));
        assert!(matcher.matches("-mesh"));
        assert!(!matcher.matches("three-mesh-child"));
        assert!(matcher.matches("TresBasicMaterial"));
        assert!(matcher.matches("TresMaterial"));
        assert!(!matcher.matches("TresMaterialX"));
        assert!(!matcher.matches("MyTresBasicMaterial"));
    }

    #[test]
    fn custom_element_matcher_projects_only_declarative_patterns() {
        let matcher = CustomElementMatcher::from_patterns(vec![
            String::from("Tres*"),
            String::from("primitive"),
        ]);

        assert_eq!(
            matcher.projectable_patterns().unwrap(),
            [String::from("Tres*"), String::from("primitive")]
        );

        let opaque = CustomElementMatcher {
            patterns: vec![String::from("Tres*")],
            predicate: Some(is_tres),
        };
        assert!(opaque.matches("TresMesh"));
        assert!(opaque.projectable_patterns().is_none());
    }

    #[test]
    fn custom_element_matcher_treats_all_wildcard_patterns_as_match_all() {
        for pattern in ["*", "**", "***"] {
            let matcher = CustomElementMatcher::from_patterns(vec![String::from(pattern)]);

            assert!(matcher.matches("div"), "pattern {pattern} should match div");
            assert!(
                matcher.matches("TresMesh"),
                "pattern {pattern} should match TresMesh"
            );
            assert!(
                matcher.matches("three-buffer-geometry"),
                "pattern {pattern} should match three-buffer-geometry"
            );
        }
    }

    #[test]
    fn custom_element_matcher_ignores_empty_patterns() {
        let matcher = CustomElementMatcher::from_patterns(vec![String::new("")]);

        assert!(!matcher.matches(""));
        assert!(!matcher.matches("div"));
    }
}
