#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

#[path = "../../../rust/common.rs"]
mod common;
#[path = "../../../rust/editor_e2e.rs"]
mod editor_e2e;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let repo = common::repo_root()?;
    let vim = env::var("VIZE_TEST_VIM_PATH").unwrap_or_else(|_| "vim".to_string());
    let vim_lsp = env::var("VIZE_TEST_VIM_LSP_PATH").map_err(|_| {
        "VIZE_TEST_VIM_LSP_PATH must point at the pinned vim-lsp checkout used by the host test"
            .to_string()
    })?;
    let vim_lsp_plugin = PathBuf::from(&vim_lsp).join("plugin/lsp.vim");
    if !vim_lsp_plugin.exists() {
        return Err(format!(
            "vim-lsp plugin not found: {}",
            vim_lsp_plugin.display()
        ));
    }
    let plugin_root = repo.join("editors/vim");
    let spec_path = plugin_root.join("test/vize_e2e_spec.vim");
    let server_path = editor_e2e::resolve_real_server_path(&repo)?;
    let session = unique_temp_dir("vize-vim-e2e")?;
    let workspace = session.join("real-vue");
    let error_path = session.join("vim-errors.log");
    let verbose_path = session.join("vim-verbose.log");
    editor_e2e::prepare_real_vue_workspace(&workspace, false)?;
    let result = Command::new(vim)
        .arg("-Nu")
        .arg("NONE")
        .arg("-n")
        .arg("-es")
        .arg("-i")
        .arg("NONE")
        .arg(format!("-V1{}", verbose_path.display()))
        .arg("-S")
        .arg(&spec_path)
        .current_dir(&workspace)
        .env("VIZE_E2E_ERROR_PATH", &error_path)
        .env("VIZE_E2E_PLUGIN_ROOT", &plugin_root)
        .env("VIZE_E2E_SERVER", &server_path)
        .env("VIZE_E2E_WORKSPACE", &workspace)
        .stdin(Stdio::null())
        .output();
    let output = result.map_err(|error| format!("failed to run headless Vim: {error}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        for path in [&error_path, &verbose_path] {
            if path.exists() {
                eprint!("{}", common::read_text(path)?);
            }
        }
        let _ = fs::remove_dir_all(&session);
        return Err(format!(
            "headless Vim scenario failed with exit code {}",
            output.status.code().unwrap_or(1)
        ));
    }
    let _ = fs::remove_dir_all(&session);
    println!("vim real-server scenario passed");
    Ok(())
}

fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let path = env::temp_dir().join(format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    common::mkdir(&path)?;
    Ok(path)
}
