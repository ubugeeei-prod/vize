//! Shared test fixtures.

/// A temporary directory under the canonical temp root. macOS's `/var` is a
/// symlink to `/private/var`, and the server compares physical paths, so a
/// fixture rooted in `/var/...` would disagree with every path it resolves.
pub(crate) fn tempdir() -> std::io::Result<tempfile::TempDir> {
    let root = std::env::temp_dir();
    tempfile::tempdir_in(root.canonicalize().unwrap_or(root))
}

/// The tsgo binary for type-aware tests, or `None` to skip.
///
/// `VIZE_TEST_DISABLE_TSGO` skips explicitly (the PR envelope), and
/// `VIZE_TEST_TSGO_PATH` names a binary. Otherwise the workspace's installed
/// `@typescript/typescript-*` runtime is resolved. A missing binary fails in CI
/// (`CI` or `VIZE_TEST_REQUIRE_TSGO=1`) instead of passing without running;
/// locally it skips with a message.
pub(crate) fn tsgo() -> Option<std::path::PathBuf> {
    select_tsgo(
        |name| std::env::var_os(name),
        || {
            let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(std::path::Path::parent);
            vize_carton::corsa_resolver::resolve_corsa_executable(
                vize_carton::corsa_resolver::CorsaResolveRequest {
                    project_root: workspace,
                    ..Default::default()
                },
            )
            .map_err(|error| vize_l0::cstr!("{error:?}"))
        },
    )
}

fn select_tsgo(
    env: impl Fn(&str) -> Option<std::ffi::OsString>,
    resolve: impl FnOnce() -> Result<std::path::PathBuf, vize_l0::String>,
) -> Option<std::path::PathBuf> {
    let set = |name: &str| env(name).is_some_and(|value| !value.is_empty() && value != "false");
    let required = env("VIZE_TEST_REQUIRE_TSGO").is_some_and(|value| value == "1");
    if set("VIZE_TEST_DISABLE_TSGO") {
        assert!(
            !required,
            "VIZE_TEST_REQUIRE_TSGO=1 conflicts with VIZE_TEST_DISABLE_TSGO"
        );
        eprintln!("skipping tsgo-backed test: VIZE_TEST_DISABLE_TSGO is set");
        return None;
    }
    if let Some(path) = env("VIZE_TEST_TSGO_PATH") {
        return Some(path.into());
    }
    match resolve() {
        Ok(path) => Some(path),
        Err(error) => {
            assert!(
                !required && !set("CI"),
                "tsgo-backed test has no tsgo binary ({error}); run `vp install` or set VIZE_TEST_TSGO_PATH"
            );
            eprintln!(
                "skipping tsgo-backed test: no tsgo binary ({error}); run `vp install` or set VIZE_TEST_TSGO_PATH"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::select_tsgo;

    fn env<'a>(vars: &'a [(&str, &str)]) -> impl Fn(&str) -> Option<std::ffi::OsString> + 'a {
        move |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).into())
        }
    }

    fn missing() -> Result<std::path::PathBuf, vize_l0::String> {
        Err("NotFound".into())
    }

    #[test]
    fn local_absence_skips() {
        assert_eq!(select_tsgo(env(&[]), missing), None);
        assert_eq!(select_tsgo(env(&[("CI", "false")]), missing), None);
    }

    #[test]
    fn explicit_disable_skips_without_resolving() {
        let resolve = || -> Result<std::path::PathBuf, vize_l0::String> {
            panic!("a disabled run must not resolve tsgo")
        };
        assert_eq!(
            select_tsgo(
                env(&[("VIZE_TEST_DISABLE_TSGO", "1"), ("CI", "true")]),
                resolve
            ),
            None
        );
    }

    #[test]
    fn explicit_and_resolved_paths_are_used() {
        assert_eq!(
            select_tsgo(env(&[("VIZE_TEST_TSGO_PATH", "/opt/tsgo")]), missing),
            Some("/opt/tsgo".into())
        );
        assert_eq!(
            select_tsgo(env(&[("CI", "true")]), || Ok("/ws/tsgo".into())),
            Some("/ws/tsgo".into())
        );
    }

    #[test]
    #[should_panic(expected = "has no tsgo binary")]
    fn absence_in_ci_fails() {
        select_tsgo(env(&[("CI", "true")]), missing);
    }

    #[test]
    #[should_panic(expected = "has no tsgo binary")]
    fn required_absence_fails() {
        select_tsgo(env(&[("VIZE_TEST_REQUIRE_TSGO", "1")]), missing);
    }

    #[test]
    #[should_panic(expected = "conflicts with VIZE_TEST_DISABLE_TSGO")]
    fn required_and_disabled_conflict() {
        select_tsgo(
            env(&[
                ("VIZE_TEST_REQUIRE_TSGO", "1"),
                ("VIZE_TEST_DISABLE_TSGO", "1"),
            ]),
            missing,
        );
    }
}
