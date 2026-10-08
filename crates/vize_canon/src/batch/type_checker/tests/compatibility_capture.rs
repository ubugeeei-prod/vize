//! Required compatibility qualification retains failures before optional skip logic.
#![cfg(test)]
use super::BatchTypeChecker;
use crate::batch::{TypeCheckResult, TypeChecker, error::CorsaResult};
use serde_json::{Value, json};
use std::path::Path;
use vize_l0::cstr;

fn finish<T>(result: CorsaResult<T>, required: bool) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            assert!(
                !required,
                "required native compatibility result failed: {error}"
            );
            None
        }
    }
}

pub(super) fn checked<T>(root: &Path, stage: &str, result: CorsaResult<T>) -> Option<T> {
    if let Err(error) = &result {
        retain(
            root,
            stage,
            &json!({"status":"error", "display":cstr!("{error}"), "debug":cstr!("{error:?}")}),
        );
    }
    finish(result, std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some())
}

pub(super) fn check_project(root: &Path) -> Option<TypeCheckResult> {
    let result = (|| {
        let mut checker = BatchTypeChecker::new(root)?;
        checker.scan_project()?;
        checker.check_project()
    })();
    if let Ok(value) = &result {
        retain(
            root,
            "check",
            &json!({
                "status":"success", "exitCode":value.exit_code, "success":value.success,
                "diagnostics":value.diagnostics.iter().map(|d| json!({
                    "file":d.file,"line":d.line,"column":d.column,"message":d.message,
                    "code":d.code,"severity":d.severity,"blockType":d.block_type.map(|b| b.block_name())
                })).collect::<Vec<_>>()
            }),
        );
    }
    checked(root, "check", result)
}

pub(super) fn retain(root: &Path, stage: &str, value: &Value) {
    let Some(capture) = std::env::var_os("VIZE_UNKNOWN_COMPAT_CAPTURE") else {
        return;
    };
    let saved = (|| -> Result<(), Box<dyn std::error::Error>> {
        let name = root.file_name().ok_or("case has no name")?;
        let target = Path::new(&capture).join("batch").join(name).join(stage);
        std::fs::create_dir_all(&target)?;
        std::fs::write(
            target.join("result.json"),
            serde_json::to_vec_pretty(value)?,
        )?;
        copy_inputs(&root.join("src"), &target.join("inputs/src"))?;
        std::fs::copy(
            root.join("tsconfig.json"),
            target.join("inputs/tsconfig.json"),
        )?;
        if root.join("node_modules").is_dir() {
            copy_inputs(
                &root.join("node_modules"),
                &target.join("inputs/node_modules"),
            )?;
        }
        let native = std::env::var_os("CORSA_PATH").map(std::path::PathBuf::from);
        std::fs::write(
            target.join(cstr!("{stage}.json").as_str()),
            serde_json::to_vec_pretty(&json!({
                "sourceSha":std::env::var("SOURCE_SHA").ok(),"projectRoot":root,
                "nativeBinary":native,"nativeSha256":native.as_deref().map(digest).transpose()?,
                "result":value
            }))?,
        )?;
        Ok(())
    })();
    if let Err(error) = saved {
        panic!("compatibility capture failed: {error}");
    }
}

fn copy_inputs(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for item in std::fs::read_dir(source)? {
        let item = item?;
        let output = target.join(item.file_name());
        let kind = item.file_type()?;
        if kind.is_dir() {
            copy_inputs(&item.path(), &output)?;
        } else if kind.is_file() {
            std::fs::copy(item.path(), output)?;
        } else if kind.is_symlink() {
            std::fs::write(
                output.with_file_name(
                    cstr!("{}.symlink-target.txt", item.file_name().to_string_lossy()).as_str(),
                ),
                std::fs::read_link(item.path())?
                    .as_os_str()
                    .as_encoded_bytes(),
            )?;
        }
    }
    Ok(())
}

fn digest(path: &Path) -> std::io::Result<vize_l0::String> {
    use sha2::{Digest, Sha256};
    Ok(Sha256::digest(std::fs::read(path)?)
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect())
}

#[test]
fn optional_error_preserves_the_no_sdk_skip_policy() {
    assert_eq!(
        finish::<()>(Err(crate::batch::error::CorsaError::NotInitialized), false),
        None
    );
}

#[test]
#[should_panic(expected = "required native compatibility result failed")]
fn required_error_cannot_return_a_skipped_snapshot() {
    let _ = finish::<()>(Err(crate::batch::error::CorsaError::NotInitialized), true);
}

#[test]
fn required_success_preserves_the_complete_value() {
    assert_eq!(
        finish(Ok(vec![("source", 2345, "whole message")]), true),
        Some(vec![("source", 2345, "whole message")])
    );
}
