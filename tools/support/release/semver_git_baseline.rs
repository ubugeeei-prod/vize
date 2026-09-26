// The real PR parent stays the API baseline across a physical package rename.
// Extract its exact tree; adapt Cargo identity lookup only. Source, dependency
// keys/versions, features and the baseline workspace version remain untouched.
use super::{DocumentMut, value};
use std::{fs, path::Path, process::Command};

pub fn prepare(
    package: &str,
    old_name: &str,
    destination: &Path,
    base: &str,
) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", &format!("{base}^{{commit}}")])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("actual Git base is unavailable".into());
    }
    let sha = String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .to_string();
    let current_manifest = Command::new("git")
        .args(["show", &format!("{sha}:crates/{package}/Cargo.toml")])
        .output()
        .map_err(|e| e.to_string())?;
    if current_manifest.status.success() {
        let document: DocumentMut = String::from_utf8(current_manifest.stdout)
            .map_err(|e| e.to_string())?
            .parse()
            .map_err(|e: toml_edit::TomlError| e.to_string())?;
        if document["package"]["name"].as_str() != Some(package) {
            return Err("actual base new-package manifest identity mismatch".into());
        }
        return Ok(None); // Preserve --baseline-rev when package lookup already works.
    }
    let archive = destination.join("actual-base.tar");
    let output = Command::new("git")
        .args(["archive", "--format=tar", "--output"])
        .arg(&archive)
        .arg(&sha)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "cannot archive actual base: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let status = Command::new("tar")
        .arg("-xf")
        .arg(&archive)
        .arg("-C")
        .arg(destination)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("cannot extract actual Git base".into());
    }
    let workspace_path = destination.join("Cargo.toml");
    let root = destination.join("crates").join(old_name);
    let manifest_path = root.join("Cargo.toml");
    let (manifest, workspace) = adapt_manifests(
        &fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?,
        &fs::read_to_string(&workspace_path).map_err(|e| e.to_string())?,
        old_name,
        package,
    )?;
    fs::write(&manifest_path, manifest).map_err(|e| e.to_string())?;
    fs::write(&workspace_path, workspace).map_err(|e| e.to_string())?;
    eprintln!(
        "SemVer baseline: exact Git parent {sha}, {old_name} -> {package}; package/lib and workspace package lookup identity only, Rust API/dependency keys/versions unchanged"
    );
    Ok(Some(root.display().to_string()))
}

fn adapt_manifests(
    source: &str,
    workspace_source: &str,
    old_name: &str,
    package: &str,
) -> Result<(String, String), String> {
    let mut manifest: DocumentMut = source
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    let mut workspace: DocumentMut = workspace_source
        .parse()
        .map_err(|e: toml_edit::TomlError| e.to_string())?;
    if manifest["package"]["name"].as_str() != Some(old_name) {
        return Err("actual base old-package identity mismatch".into());
    }
    if let Some(name) = manifest.get("lib").and_then(|lib| lib.get("name")) {
        if name.as_str() != Some(old_name) {
            return Err("actual base library identity mismatch".into());
        }
        manifest["lib"]["name"] = value(package);
    }
    let dependency = workspace["workspace"]["dependencies"][old_name]
        .as_table_like_mut()
        .ok_or("actual base workspace dependency lookup is missing")?;
    if dependency.get("path").and_then(|path| path.as_str()) != Some(&format!("crates/{old_name}"))
    {
        return Err("actual base workspace dependency path mismatch".into());
    }
    if let Some(name) = dependency.get("package") {
        if name.as_str() != Some(old_name) {
            return Err("actual base workspace package identity mismatch".into());
        }
    }
    // The dependency key/path/version stay old, so every old Rust import and
    // reverse workspace consumer still resolves the original source package.
    dependency.insert("package", value(package));
    manifest["package"]["name"] = value(package);
    Ok((manifest.to_string(), workspace.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_base_metadata_keeps_old_versions_aliases_and_library_path() {
        let source = "[package]\nname = \"vize_s1_to_s2\"\nversion.workspace = true\n[lib]\nname = \"vize_s1_to_s2\"\npath = \"src/lib.rs\"\n[dependencies]\nvize_s1.workspace = true\n";
        let workspace = "[workspace.package]\nversion = \"0.427.99\"\n[workspace.dependencies]\nvize_s1_to_s2 = { path = \"crates/vize_s1_to_s2\", version = \"=0.427.99\" }\n";
        let (actual, workspace_actual) =
            adapt_manifests(source, workspace, "vize_s1_to_s2", "vize_l1_to_l2").unwrap();
        assert_eq!(actual.replace("vize_l1_to_l2", "vize_s1_to_s2"), source);
        let mut restored: DocumentMut = workspace_actual.parse().unwrap();
        restored["workspace"]["dependencies"]["vize_s1_to_s2"]
            .as_table_like_mut()
            .unwrap()
            .remove("package");
        assert_eq!(restored.to_string(), workspace);
        assert!(
            adapt_manifests(
                source,
                &workspace.replace("crates/vize_s1_to_s2", "crates/wrong"),
                "vize_s1_to_s2",
                "vize_l1_to_l2"
            )
            .is_err()
        );
    }
}
