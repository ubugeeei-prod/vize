//! Classification shared by CLI and editor native diagnostics.

/// Recognize a TS2322 error on a generated unreachable-arm assertion.
/// `offset` is the diagnostic's byte offset in generated TypeScript.
pub fn is_unreachable_pattern_diagnostic(source: &str, offset: usize, code: Option<u32>) -> bool {
    if code != Some(2322) {
        return false;
    }
    let Some(rest) = source.get(offset..) else {
        return false;
    };
    let Some((name, suffix)) = rest.split_once(':') else {
        return false;
    };
    name.starts_with("__vize_match_")
        && name.ends_with("unreachable")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && suffix.starts_with(" __VizePatterns.Reachable<")
}

#[cfg(test)]
mod tests {
    use super::is_unreachable_pattern_diagnostic as classify;

    #[test]
    fn only_generated_reachability_assignments_are_warnings() {
        let source = "const __vize_match_2_unreachable: __VizePatterns.Reachable<T> = true;";
        assert!(classify(source, 6, Some(2322)));
        assert!(!classify(source, 6, Some(2304)));
        assert!(!classify(source, 0, Some(2322)));
        assert!(!classify("value: number = 'bad'", 0, Some(2322)));
        assert!(!classify(
            "__vize_match_0_\nunreachable: __VizePatterns.Reachable<T>",
            0,
            Some(2322)
        ));
        assert!(!classify("\u{1f600}", 1, Some(2322)));
        assert!(!classify(source, source.len() + 1, Some(2322)));
    }
}
