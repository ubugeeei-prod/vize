//! Option lists whose entries are a literal or a `/pattern/flags` expression.
//!
//! An entry shaped like a JavaScript regular-expression literal is compiled
//! with Rust regex syntax, and its `i`, `m` and `s` flags are honored. An
//! entry the engine cannot compile stays a literal.

use regex::Regex;
use vize_l0::{String, cstr};

/// Compile a `/pattern/flags` entry. `None` leaves the entry a literal.
pub(crate) fn compile_pattern(entry: &str) -> Option<Regex> {
    if let Some((pattern, flags)) = javascript_regex(entry) {
        let inline_flags: String = flags
            .chars()
            .filter(|flag| matches!(flag, 'i' | 'm' | 's'))
            .collect();
        let candidate = if inline_flags.is_empty() {
            cstr!("(?:{pattern})")
        } else {
            cstr!("(?{inline_flags}:{pattern})")
        };
        if let Ok(pattern) = Regex::new(&candidate) {
            return Some(pattern);
        }
    }
    None
}

fn javascript_regex(value: &str) -> Option<(&str, &str)> {
    if !value.starts_with('/') {
        return None;
    }
    let end = value.rfind('/')?;
    if end == 0 {
        return None;
    }
    let flags = value.get(end + 1..)?;
    if !flags
        .chars()
        .all(|flag| matches!(flag, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'y'))
    {
        return None;
    }
    Some((value.get(1..end)?, flags))
}

/// Names a rule leaves alone, as eslint-plugin-vue reads an `ignores` list:
/// a literal matches the whole name, a pattern matches anywhere in it.
#[derive(Default)]
pub(crate) struct NameMatcher {
    literals: Vec<String>,
    patterns: Vec<Regex>,
}

impl NameMatcher {
    /// The matcher of an unconfigured rule: nothing matches.
    pub(crate) const fn empty() -> Self {
        Self {
            literals: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// Compile the configured entries.
    pub(crate) fn new(entries: &[String]) -> Self {
        let mut matcher = Self::empty();
        for entry in entries {
            match compile_pattern(entry) {
                Some(pattern) => matcher.patterns.push(pattern),
                None => matcher.literals.push(entry.clone()),
            }
        }
        matcher
    }

    /// Whether no entry is configured.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.literals.is_empty() && self.patterns.is_empty()
    }

    /// Whether an entry names `name`.
    pub(crate) fn matches(&self, name: &str) -> bool {
        self.literals.iter().any(|literal| literal == name)
            || self.patterns.iter().any(|pattern| pattern.is_match(name))
    }
}

#[cfg(test)]
mod tests {
    use super::NameMatcher;
    use vize_l0::String;

    fn matcher(entries: &[&str]) -> NameMatcher {
        NameMatcher::new(
            &entries
                .iter()
                .copied()
                .map(String::from)
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn a_literal_matches_the_whole_name_and_a_pattern_matches_within_it() {
        let names = matcher(&["primitive", "/^Tres/", "/-ICON$/i", "a.b"]);
        for (name, expected) in [
            ("primitive", true),
            ("Primitive", false),
            ("primitives", false),
            ("my-primitive", false),
            ("TresMesh", true),
            ("tres-mesh", false),
            ("MyTres", false),
            ("x-icon", true),
            ("a.b", true),
            ("axb", false),
        ] {
            assert_eq!(names.matches(name), expected, "{name}");
        }
        assert!(!names.is_empty());
    }

    #[test]
    fn an_uncompilable_pattern_stays_a_literal_and_no_entry_matches_nothing() {
        let names = matcher(&["/foo(?=bar)/", "/unterminated"]);
        for (name, expected) in [
            ("/foo(?=bar)/", true),
            ("foobar", false),
            ("/unterminated", true),
            ("unterminated", false),
        ] {
            assert_eq!(names.matches(name), expected, "{name}");
        }
        let nothing = NameMatcher::empty();
        assert!(nothing.is_empty());
        assert!(!nothing.matches("primitive"));
        assert!(!matcher(&[]).matches(""));
    }
}
