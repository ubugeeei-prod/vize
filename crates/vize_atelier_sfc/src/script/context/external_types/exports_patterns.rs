/// Match `key` against Node-style `exports` pattern keys such as `"./dist/*"`,
/// returning the captured wildcard segment and the pattern's target value.
///
/// The most specific pattern wins: longest literal prefix first, then longest
/// literal suffix, matching Node's package export resolution order.
pub(super) fn best_pattern_match<'a>(
    exports: &'a serde_json::Value,
    key: &'a str,
) -> Option<(&'a str, &'a serde_json::Value)> {
    let mut best: Option<(usize, usize, &str, &serde_json::Value)> = None;
    for (pattern, target) in exports.as_object()? {
        if !pattern.starts_with("./") {
            continue;
        }
        let Some((prefix, suffix)) = pattern.split_once('*') else {
            continue;
        };
        let Some(captured) = key
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix))
        else {
            continue;
        };
        if best.is_none_or(|(best_prefix, best_suffix, _, _)| {
            (prefix.len(), suffix.len()) > (best_prefix, best_suffix)
        }) {
            best = Some((prefix.len(), suffix.len(), captured, target));
        }
    }
    best.map(|(_, _, captured, target)| (captured, target))
}

#[cfg(test)]
mod tests {
    use super::best_pattern_match;

    fn exports(raw: &str) -> serde_json::Value {
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn captures_the_wildcard_segment() {
        let exports = exports(r#"{"./dist/*":{"types":"./types/*.d.ts"}}"#);
        let (captured, target) = best_pattern_match(&exports, "./dist/nested/foo").unwrap();
        assert_eq!(captured, "nested/foo");
        assert_eq!(target["types"], "./types/*.d.ts");
    }

    #[test]
    fn prefers_the_most_specific_pattern() {
        let exports =
            exports(r#"{"./*":{"types":"./root/*.d.ts"},"./dist/*.js":{"types":"./dist/*.d.ts"}}"#);
        let (captured, target) = best_pattern_match(&exports, "./dist/foo.js").unwrap();
        assert_eq!(captured, "foo");
        assert_eq!(target["types"], "./dist/*.d.ts");
    }

    #[test]
    fn ignores_condition_keys_and_unmatched_patterns() {
        let exports = exports(r#"{"types":"./root.d.ts","./dist/*":{"types":"./dist/*.d.ts"}}"#);
        assert!(best_pattern_match(&exports, "./other/foo").is_none());
    }
}
