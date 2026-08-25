use std::path::{Path, PathBuf};

use vize_s0::cstr;

pub(crate) fn resolve(workspace_root: &Path) -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH").filter(|path| Path::new(path).is_file()) {
        return Some(path.to_string_lossy().into_owned());
    }

    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    if let Some(native_tsgo) = resolve_workspace_native_tsgo(workspace_root) {
        return Some(native_tsgo.display().to_string());
    }

    for candidate in [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate.display().to_string());
        }
    }

    None
}

fn resolve_workspace_native_tsgo(workspace_root: &Path) -> Option<PathBuf> {
    let platform_suffix = native_preview_platform_suffix();
    let package_name = cstr!("@typescript/native-preview-{platform_suffix}");

    let pnpm_root = workspace_root.join("node_modules/.pnpm");
    if let Ok(entries) = std::fs::read_dir(&pnpm_root) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with(cstr!("@typescript+native-preview-{platform_suffix}@").as_str()) {
                continue;
            }

            if let Some(path) = first_existing_tsgo_binary(
                entry
                    .path()
                    .join("node_modules")
                    .join("@typescript")
                    .join(package_name.as_str())
                    .join("lib"),
            ) {
                return Some(path);
            }
        }
    }

    first_existing_tsgo_binary(
        workspace_root
            .join("node_modules")
            .join("@typescript")
            .join(package_name.as_str())
            .join("lib"),
    )
}

fn first_existing_tsgo_binary(lib_dir: PathBuf) -> Option<PathBuf> {
    for executable in test_corsa_executable_names() {
        let candidate = lib_dir.join(executable);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn test_corsa_executable_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["tsgo.exe", "tsgo", "corsa.exe", "corsa"]
    } else {
        &["tsgo", "corsa"]
    }
}

fn native_preview_platform_suffix() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-arm64"
        } else {
            "linux-x64"
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "win32-arm64"
        } else {
            "win32-x64"
        }
    } else {
        ""
    }
}
