#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! vize_tool_support = { path = "../../../support" }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

use vize_tool_support::common;
use vize_tool_support::editors::e2e as editor_e2e;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let repo = common::repo_root()?;
    let nvim = env::var("VIZE_TEST_NVIM_PATH").unwrap_or_else(|_| "nvim".to_string());
    let plugin_root = repo.join("editors/nvim");
    let spec_path = plugin_root.join("test/vize_e2e_spec.lua");
    let server_path = editor_e2e::resolve_real_server_path(&repo)?;
    let session = unique_temp_dir("vize-nvim-e2e")?;
    let workspace = session.join("real-vue");
    if let Some(initialized) = env::var_os("VIZE_E2E_WORKSPACE").filter(|path| !path.is_empty()) {
        copy_dir_contents(&PathBuf::from(initialized), &workspace)?;
    } else {
        editor_e2e::prepare_real_vue_workspace(&workspace, false)?;
    }
    let runtimepath = serde_json::to_string(&plugin_root.display().to_string())
        .map_err(|error| error.to_string())?;
    let spec = serde_json::to_string(&spec_path.display().to_string())
        .map_err(|error| error.to_string())?;
    let result = Command::new(nvim)
        .args([
            "--headless",
            "-u",
            "NONE",
            "--noplugin",
            "-n",
            "-i",
            "NONE",
            "-c",
            &format!("lua vim.opt.runtimepath:prepend({runtimepath})"),
            "-c",
            &format!("lua dofile({spec})"),
            "-c",
            "qall!",
        ])
        .current_dir(&workspace)
        .env("VIZE_E2E_SERVER", &server_path)
        .env("VIZE_E2E_WORKSPACE", &workspace)
        .stdin(Stdio::null())
        .output();
    let _ = fs::remove_dir_all(&session);
    let output = result.map_err(|error| format!("failed to run headless Neovim: {error}"))?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(format!(
            "headless Neovim scenario failed with exit code {}",
            output.status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn copy_dir_contents(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|error| format!("cannot create {}: {error}", to.display()))?;
    for entry in
        fs::read_dir(from).map_err(|error| format!("cannot read {}: {error}", from.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = to.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            copy_dir_contents(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)
                .map_err(|error| format!("cannot copy {}: {error}", target.display()))?;
        }
    }
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
