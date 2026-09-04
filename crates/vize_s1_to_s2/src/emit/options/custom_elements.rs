//! The shipped lane's `CustomElementMatcher`, mirrored.
//!
//! `isCustomElement` keeps a tag an *element* that the tag rules would
//! otherwise make a component: the shipped lane checks it in
//! `lane/element.rs` after the registered-component lookup and before the
//! `component` / PascalCase / hyphen / `is` heuristic, and a match leaves
//! `tag_type` alone.
//!
//! The matcher itself lives in `vize_relief`, which the davinci stage
//! crates deliberately do not depend on, so the patterns arrive here as a
//! borrowed slice and the glob rule is mirrored rather than called. The
//! two are pinned against each other by the atelier_dom witness, which
//! can see both crates.

/// Tag patterns that keep a non-native tag an element. Exact strings, or
/// globs whose `*` matches any run of characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CustomElementPatterns<'a> {
    patterns: &'a [&'a str],
}

impl<'a> CustomElementPatterns<'a> {
    /// Borrow `patterns` as the custom-element rule.
    #[must_use]
    pub const fn new(patterns: &'a [&'a str]) -> Self {
        Self { patterns }
    }

    /// The patterns this rule carries.
    #[must_use]
    pub const fn patterns(&self) -> &'a [&'a str] {
        self.patterns
    }

    /// Whether no pattern can match.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Whether `tag` is configured as a custom element.
    #[must_use]
    pub fn matches(&self, tag: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| tag_pattern_matches(pattern, tag))
    }
}

/// The shipped `tag_pattern_matches`, mirrored rule for rule.
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
    use super::CustomElementPatterns;

    #[test]
    fn exact_patterns_match_only_that_tag() {
        let rules = CustomElementPatterns::new(&["ion-button"]);
        assert_eq!(
            (rules.matches("ion-button"), rules.matches("ion-buttons")),
            (true, false)
        );
    }

    #[test]
    fn a_trailing_wildcard_matches_the_prefix() {
        let rules = CustomElementPatterns::new(&["Tres*"]);
        assert_eq!(
            (
                rules.matches("TresMesh"),
                rules.matches("Tres"),
                rules.matches("MyTresMesh")
            ),
            (true, true, false)
        );
    }

    #[test]
    fn a_leading_wildcard_matches_the_suffix() {
        let rules = CustomElementPatterns::new(&["*-icon"]);
        assert_eq!(
            (rules.matches("app-icon"), rules.matches("app-icons")),
            (true, false)
        );
    }

    #[test]
    fn a_bare_wildcard_matches_everything_and_empty_matches_nothing() {
        let all = CustomElementPatterns::new(&["*"]);
        let empty = CustomElementPatterns::new(&[""]);
        let none = CustomElementPatterns::new(&[]);
        assert_eq!(
            (all.matches("x"), empty.matches("x"), none.matches("x")),
            (true, false, false)
        );
        assert_eq!((none.is_empty(), all.is_empty()), (true, false));
    }

    #[test]
    fn an_inner_wildcard_matches_around_it() {
        let rules = CustomElementPatterns::new(&["a*z"]);
        assert_eq!(
            (
                rules.matches("abcz"),
                rules.matches("az"),
                rules.matches("abc")
            ),
            (true, true, false)
        );
    }
}
