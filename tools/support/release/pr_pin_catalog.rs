use super::super::pr_github as github;
use super::metadata::{paths, text};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

pub(super) fn catalog(revision: &str, version: &str, root: &Path) -> Result<Value, String> {
    let mut npm = BTreeMap::new();
    let mut editors = BTreeMap::new();
    for path in paths(revision, root)?.into_iter().filter(|p| {
        (p.starts_with("npm/") || p.starts_with("editors/")) && p.ends_with("/package.json")
    }) {
        let manifest: Value =
            serde_json::from_str(&text(revision, &path, root)?).map_err(|e| e.to_string())?;
        if manifest.get("private").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        if manifest.get("version").and_then(Value::as_str) != Some(version) {
            return Err(format!("{revision}:{path} is not aligned at {version}."));
        }
        let name = manifest
            .get("name")
            .and_then(Value::as_str)
            .ok_or("Published npm manifest lacks name")?;
        let (catalog, identity) = if path.starts_with("npm/") {
            (&mut npm, name.to_string())
        } else {
            let publisher = manifest["publisher"]
                .as_str()
                .ok_or("Editor manifest lacks publisher")?;
            (&mut editors, format!("{publisher}.{name}"))
        };
        if catalog
            .insert(identity.clone(), json!({"path":path,"manifest":manifest}))
            .is_some()
        {
            return Err(format!(
                "Duplicate package identity {identity} within its distribution channel"
            ));
        }
    }
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let directory = env::temp_dir().join(format!(
        "vize-release-catalog-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    github::git(
        &[
            "worktree",
            "add",
            "--detach",
            directory.to_str().ok_or("Invalid catalog path")?,
            revision,
        ],
        root,
    )?;
    let result = (|| {
        let output = Command::new("cargo")
            .env("RUSTUP_TOOLCHAIN", "stable")
            .args([
                "metadata",
                "--no-deps",
                "--offline",
                "--locked",
                "--format-version",
                "1",
            ])
            .current_dir(&directory)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "Cargo publication metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        let cargo_root = Path::new(
            metadata["workspace_root"]
                .as_str()
                .ok_or("Missing Cargo catalog workspace root")?,
        );
        if fs::canonicalize(cargo_root).map_err(|e| e.to_string())?
            != fs::canonicalize(&directory).map_err(|e| e.to_string())?
        {
            return Err("Cargo metadata escaped the isolated catalog checkout.".into());
        }
        let packages = metadata["packages"]
            .as_array()
            .ok_or("Missing Cargo catalog")?;
        let mut crates = BTreeMap::new();
        for package in packages {
            if package
                .get("publish")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
            {
                continue;
            }
            let name = package["name"].as_str().ok_or("Missing crate name")?;
            if package["version"].as_str() != Some(version) {
                return Err(format!("Crate {name} at {revision} is not {version}"));
            }
            let path = Path::new(
                package["manifest_path"]
                    .as_str()
                    .ok_or("Missing manifest path")?,
            )
            .strip_prefix(cargo_root)
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("Invalid crate path")?;
            let mut dependencies: Vec<Value> = package["dependencies"].as_array().ok_or("Missing crate dependencies")?.iter().filter(|d| d["kind"].as_str() != Some("dev")).map(|d| {
                let local_path = d["path"].as_str().map(|path| Path::new(path).strip_prefix(cargo_root).map(|p|p.to_string_lossy().into_owned()).unwrap_or_else(|_|path.to_string()));
                json!({"name":d["name"],"kind":d["kind"],"target":d["target"],"optional":d["optional"],"rename":d["rename"],"localPath":local_path,"req":d["req"],"source":d["source"],"registry":d["registry"],"features":d["features"],"usesDefaultFeatures":d["uses_default_features"]})
            }).collect();
            dependencies.sort_by_key(Value::to_string);
            let mut targets = package["targets"]
                .as_array()
                .ok_or("Missing crate targets")?
                .clone();
            for target in &mut targets {
                if let Some(source) = target["src_path"].as_str() {
                    let relative = Path::new(source)
                        .strip_prefix(cargo_root)
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        .into_owned();
                    target["src_path"] = json!(relative);
                }
            }
            let mut descriptor = package.clone();
            let object = descriptor
                .as_object_mut()
                .ok_or("Invalid Cargo publication descriptor")?;
            object.remove("id");
            object.remove("manifest_path");
            object.insert("dependencies".into(), json!(dependencies));
            object.insert("targets".into(), json!(targets));
            crates.insert(
                name.to_string(),
                json!({"path":path,"manifest":text(revision,path,root)?,"publication":descriptor}),
            );
        }
        Ok(
            json!({"npm":npm,"editors":editors,"crates":crates,"cratePublisher":text(revision,"tools/moon/cmd/publish_crates/main.mbt",root)?,"nativeCatalog": native_catalog(&text(revision,"pnpm-workspace.yaml",root)?,version)}),
        )
    })();
    // This path was allocated by this invocation; no shared checkout is touched.
    let removed = github::git(
        &[
            "worktree",
            "remove",
            directory.to_str().ok_or("Invalid catalog path")?,
        ],
        root,
    );
    if result.is_ok() {
        removed?;
    } else {
        let _ = removed;
        let _ = fs::remove_dir_all(&directory);
    }
    result
}

fn native_catalog(content: &str, version: &str) -> Vec<String> {
    let mut active = false;
    content
        .lines()
        .filter_map(|line| {
            if line == "  native-binaries:" {
                active = true;
            } else if active
                && !line.is_empty()
                && !line.starts_with("    ")
                && !line.starts_with("  #")
            {
                active = false;
            }
            (active && line.trim().starts_with('"')).then(|| line.replace(version, "<release>"))
        })
        .collect()
}
