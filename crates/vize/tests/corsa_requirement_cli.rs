#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn available_corsa_never_skips() {
    let _runtime_entrypoint = corsa_requirement::required_or_skip::<String>;
    assert_eq!(
        corsa_requirement::required_or_skip_with(Some("tsgo"), true, false),
        Some("tsgo")
    );
}

#[test]
fn missing_optional_corsa_keeps_the_local_skip() {
    assert_eq!(
        corsa_requirement::required_or_skip_with::<()>(None, false, false),
        None
    );
}

#[test]
fn explicit_disable_keeps_the_opt_out_with_or_without_corsa() {
    assert_eq!(
        corsa_requirement::required_or_skip_with(Some("tsgo"), true, true),
        None
    );
    assert_eq!(
        corsa_requirement::required_or_skip_with::<()>(None, true, true),
        None
    );
}

#[test]
#[should_panic(
    expected = "VIZE_TEST_REQUIRE_TSGO is set, but no TypeScript 7/Corsa executable was found"
)]
fn missing_required_corsa_fails_closed() {
    let _ = corsa_requirement::required_or_skip_with::<()>(None, true, false);
}
