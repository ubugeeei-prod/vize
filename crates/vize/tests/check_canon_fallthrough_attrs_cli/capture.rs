//! Complete unnormalized CLI observations before the unchanged legacy assertions.
#![cfg(test)]
use serde_json::json;
use std::{
    path::Path,
    process::{Command, Output},
};
use vize_l0::cstr;

pub(super) fn retain(
    root: &Path,
    native: &Path,
    command: &Command,
    result: &std::io::Result<Output>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(capture) = std::env::var_os("VIZE_UNKNOWN_COMPAT_CAPTURE") else {
        return Ok(());
    };
    let target = Path::new(&capture)
        .join("cli")
        .join(root.file_name().ok_or("case has no name")?);
    std::fs::create_dir_all(&target)?;
    let (exit_code, status, error) = match result {
        Ok(output) => {
            std::fs::write(target.join("stdout.bin"), &output.stdout)?;
            std::fs::write(target.join("stderr.bin"), &output.stderr)?;
            (output.status.code(), cstr!("{}", output.status), None)
        }
        Err(error) => (
            None,
            cstr!("spawn-error"),
            Some(json!({"display":cstr!("{error}"),"debug":cstr!("{error:?}")})),
        ),
    };
    std::fs::write(
        target.join("result.json"),
        serde_json::to_vec_pretty(
            &json!({"exitCode":exit_code,"status":status,"spawnError":error}),
        )?,
    )?;
    copy_inputs(&root.join("src"), &target.join("inputs/src"))?;
    std::fs::copy(
        root.join("tsconfig.json"),
        target.join("inputs/tsconfig.json"),
    )?;
    std::fs::write(
        target.join("runtime.json"),
        serde_json::to_vec_pretty(&json!({
            "sourceSha":std::env::var("SOURCE_SHA").ok(), "projectRoot":root,
            "nativeBinary":native,"nativeSha256":digest(native)?,
            "executable":command.get_program().to_string_lossy(),"executableSha256":digest(Path::new(command.get_program()))?,
            "arguments":command.get_args().map(|arg|arg.to_string_lossy()).collect::<Vec<_>>(),
            "workingDirectory":command.get_current_dir(),"exitCode":exit_code,"status":status,"spawnError":error
        }))?,
    )?;
    Ok(())
}

fn copy_inputs(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for item in std::fs::read_dir(source)? {
        let item = item?;
        let output = target.join(item.file_name());
        if item.file_type()?.is_dir() {
            copy_inputs(&item.path(), &output)?;
        } else if item.file_type()?.is_file() {
            std::fs::copy(item.path(), output)?;
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
