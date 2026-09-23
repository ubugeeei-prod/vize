// Build scripts cannot depend on the runtime's CompactString types or macros.
#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

fn main() {
    #[cfg(all(feature = "napi", not(target_arch = "wasm32")))]
    napi_build::setup();

    let crate_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("crate dir"));
    let root = crate_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let mut hasher = Sha256::new();
    // The source closure that builds the S2 plugin document and its facts.
    // Watching the files also refreshes this ID for dirty local builds whose
    // Git revision has not changed yet.
    for relative in [
        "Cargo.lock",
        "crates/vize_vitrine/Cargo.toml",
        "crates/vize_vitrine/build.rs",
        "crates/vize_vitrine/src/lib.rs",
        "crates/vize_vitrine/src/napi.rs",
        "crates/vize_vitrine/src/napi/plugin_sdk.rs",
        "crates/vize_vitrine/src/napi/plugin_sdk",
        "crates/vize_davinci/src",
        "crates/vize_carton/src",
        "crates/vize_s1/src",
        "crates/vize_s1_to_s2/src",
        "crates/vize_s2/src",
        "crates/vize_croquis/src",
        "crates/vize_relief/src",
        "crates/vize_armature/src",
    ] {
        let path = root.join(relative);
        hash_source(root, &path, &mut hasher);
    }
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|sha| sha.trim().to_owned())
        .unwrap_or_else(|| "source-only".to_owned());
    // A committed same-version checkout moves this path; source hashes alone
    // also distinguish dirty edits and builds without Git metadata.
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--git-path", "HEAD"])
        .current_dir(root)
        .output()
        && output.status.success()
        && let Ok(path) = String::from_utf8(output.stdout)
    {
        println!(
            "cargo:rerun-if-changed={}",
            root.join(path.trim()).display()
        );
    }
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("cargo:rustc-env=VIZE_PLUGIN_HOST_BUILD_ID={revision}:{digest}");
}

fn hash_source(root: &Path, path: &Path, hasher: &mut Sha256) {
    if path.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .expect("plugin source directory")
            .map(|entry| entry.expect("plugin source entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            hash_source(root, &entry, hasher);
        }
        return;
    }
    println!("cargo:rerun-if-changed={}", path.display());
    let relative = path.strip_prefix(root).expect("workspace-relative source");
    let relative = relative.to_string_lossy().replace('\\', "/");
    let bytes = fs::read(path).expect("plugin source file");
    for field in [relative.as_bytes(), bytes.as_slice()] {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field);
    }
}
