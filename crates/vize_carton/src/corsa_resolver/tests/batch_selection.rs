#![cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    )
))]

use super::super::{
    CorsaResolveError, CorsaResolveRequest, platform_suffix, resolve_batch_with_env,
};
use super::{resolve, write_file, write_tsrs_install, write_typescript_manifest};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn resolve_batch(
    explicit_path: Option<&Path>,
    project_root: Option<&Path>,
    env: &[(&str, &Path)],
) -> Result<PathBuf, CorsaResolveError> {
    let request = CorsaResolveRequest {
        explicit_path,
        project_root,
    };
    resolve_batch_with_env(request, |name| {
        env.iter()
            .find(|(env_name, _)| *env_name == name)
            .map(|(_, path)| OsString::from(path.as_os_str()))
    })
}

#[test]
fn batch_discovery_prefers_declared_tsrs_but_keeps_explicit_authority() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let suffix = platform_suffix();
    let (_, tsrs) = write_tsrs_install(root);
    write_typescript_manifest(
        &root.join("node_modules/typescript/package.json"),
        suffix,
        "7.0.2",
    );
    let typescript = root
        .join("node_modules/@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
        .join("lib/tsc");
    write_file(&typescript);
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"typescript":"7.0.2","tsrs":"0.18.0"}}"#,
    )
    .unwrap();

    assert_eq!(
        resolve(None, Some(root), &[]).unwrap(),
        typescript.canonicalize().unwrap()
    );
    assert_eq!(
        resolve_batch(None, Some(root), &[]).unwrap(),
        tsrs.canonicalize().unwrap()
    );
    assert_eq!(
        resolve_batch(Some(&typescript), Some(root), &[]).unwrap(),
        typescript.canonicalize().unwrap()
    );
    assert_eq!(
        resolve_batch(None, Some(root), &[("TSGO_PATH", &typescript)]).unwrap(),
        typescript.canonicalize().unwrap()
    );
}
