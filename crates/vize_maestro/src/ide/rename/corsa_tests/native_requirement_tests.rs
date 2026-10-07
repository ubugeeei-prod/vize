use std::path::{Path, PathBuf};

use vize_carton::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};

pub(super) fn resolve_tsgo_binary() -> Option<PathBuf> {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1");
    let disabled = std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some();
    let request = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(|root| CorsaResolveRequest {
            project_root: Some(root),
            ..Default::default()
        });
    resolve(required, disabled, request)
}

fn resolve(
    required: bool,
    disabled: bool,
    request: Option<CorsaResolveRequest<'_>>,
) -> Option<PathBuf> {
    assert!(
        !required || !disabled,
        "VIZE_TEST_REQUIRE_TSGO=1 conflicts with VIZE_TEST_DISABLE_TSGO"
    );
    if disabled {
        return None;
    }
    let Some(request) = request else {
        assert!(
            !required,
            "required native rename workspace root is missing"
        );
        return None;
    };
    let binary = resolve_corsa_executable(request);
    if required {
        Some(binary.expect("VIZE_TEST_REQUIRE_TSGO=1 requires native rename execution"))
    } else {
        binary.ok()
    }
}

#[test]
#[should_panic(expected = "VIZE_TEST_REQUIRE_TSGO=1 requires native rename execution")]
fn required_native_missing_executable_refuses_to_skip() {
    let project = tempfile::TempDir::new().expect("isolated native lookup");
    let missing = project.path().join("missing-native");
    resolve(
        true,
        false,
        Some(CorsaResolveRequest {
            explicit_path: Some(&missing),
            project_root: Some(project.path()),
        }),
    );
}

#[test]
#[should_panic(expected = "required native rename workspace root is missing")]
fn required_native_missing_workspace_refuses_to_skip() {
    resolve(true, false, None);
}

#[test]
#[should_panic(expected = "VIZE_TEST_REQUIRE_TSGO=1 conflicts with VIZE_TEST_DISABLE_TSGO")]
fn required_native_contradictory_disable_refuses_to_skip() {
    resolve(true, true, None);
}

#[test]
fn optional_native_absence_retains_ordinary_skip_behavior() {
    let project = tempfile::TempDir::new().expect("isolated native lookup");
    let missing = project.path().join("missing-native");
    assert!(
        resolve(
            false,
            false,
            Some(CorsaResolveRequest {
                explicit_path: Some(&missing),
                project_root: Some(project.path()),
            }),
        )
        .is_none()
    );
    assert!(resolve(false, true, None).is_none());
    assert!(resolve(false, false, None).is_none());
}
