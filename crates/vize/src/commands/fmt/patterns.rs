#[allow(clippy::disallowed_types)]
pub(super) fn default_fmt_patterns() -> Vec<std::string::String> {
    vec![
        "./**/*.vue".into(),
        "./**/*.js".into(),
        "./**/*.mjs".into(),
        "./**/*.cjs".into(),
        "./**/*.ts".into(),
        "./**/*.mts".into(),
        "./**/*.cts".into(),
        "./**/*.jsx".into(),
        "./**/*.tsx".into(),
        "./**/*.json".into(),
    ]
}

#[inline]
#[allow(clippy::disallowed_types)]
pub(super) fn has_explicit_patterns(patterns: &[std::string::String]) -> bool {
    patterns != default_fmt_patterns().as_slice()
}
