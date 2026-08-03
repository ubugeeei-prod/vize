const MISSING_REQUIRED_TSGO: &str =
    "VIZE_TEST_REQUIRE_TSGO is set, but no tsgo executable was found";

/// Keep dependency-free local runs skippable, but make required CI lanes fail
/// closed. `VIZE_TEST_DISABLE_TSGO` is the explicit opt-out used by the canon
/// tests and intentionally takes precedence over discovery.
pub(crate) fn required_or_skip<T>(resolved: Option<T>) -> Option<T> {
    required_or_skip_with(
        resolved,
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
        std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some(),
    )
}

pub(crate) fn required_or_skip_with<T>(
    resolved: Option<T>,
    required: bool,
    disabled: bool,
) -> Option<T> {
    if disabled {
        return None;
    }
    assert!(resolved.is_some() || !required, "{MISSING_REQUIRED_TSGO}");
    resolved
}
