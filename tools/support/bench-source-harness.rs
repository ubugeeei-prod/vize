//! Private source/report custody for the opt-in complete memory comparison.
//! These before/after observations are not compiler-open-time attestation.
use serde_json::Value;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

macro_rules! require {
    ($condition:expr, $message:expr) => {
        if !$condition {
            return Err(format!("source harness: {}", $message));
        }
    };
}

pub struct ValidatedPair {
    baseline: String,
    current: String,
}
impl ValidatedPair {
    pub fn versions(&self, baseline: &str, current: &str) -> Result<(), String> {
        require!(
            baseline == self.baseline && current == self.current,
            "report differs from own-source version"
        );
        Ok(())
    }
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("source harness: missing string {key}"))
}
fn fields(value: &Value, expected: &[&str]) -> Result<(), String> {
    let object = value.as_object().ok_or("source harness: expected object")?;
    require!(
        object.keys().map(String::as_str).collect::<BTreeSet<_>>()
            == expected.iter().copied().collect(),
        "missing or extra fields"
    );
    Ok(())
}
fn physical(path: &Path, directory: bool) -> Result<(), String> {
    let stat = fs::symlink_metadata(path)
        .map_err(|e| format!("source harness: {}: {e}", path.display()))?;
    require!(
        !stat.file_type().is_symlink()
            && if directory {
                stat.is_dir()
            } else {
                stat.is_file()
            },
        "nonregular or symlink path"
    );
    require!(
        path.is_absolute() && fs::canonicalize(path).map_err(|e| e.to_string())? == path,
        "foreign physical path"
    );
    Ok(())
}
fn bytes(path: &Path) -> Result<Vec<u8>, String> {
    physical(path, false)?;
    fs::read(path).map_err(|e| e.to_string())
}
fn json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&bytes(path)?).map_err(|e| format!("source harness: invalid JSON: {e}"))
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn blob(bytes: &[u8]) -> String {
    let mut digest = Sha1::new();
    digest.update(format!("blob {}\0", bytes.len()));
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}
fn utf8(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn completed(directory: &Path) -> Result<(Value, Vec<u8>), String> {
    let receipt = json(&directory.join("process.json"))?;
    fields(
        &receipt,
        &[
            "executable",
            "args",
            "cwd",
            "recipeEnvironment",
            "imageBefore",
            "imageAfter",
            "started",
            "finished",
            "status",
            "signal",
            "pid",
            "error",
            "stdout",
            "stderr",
        ],
    )?;
    require!(
        receipt["status"] == 0 && receipt["signal"].is_null() && receipt["error"].is_null(),
        "incomplete or failed process"
    );
    require!(
        receipt["pid"].as_u64().is_some_and(|pid| pid > 0),
        "missing process PID"
    );
    require!(
        receipt["imageBefore"].is_object() && receipt["imageBefore"] == receipt["imageAfter"],
        "missing or changed process image"
    );
    let image = &receipt["imageBefore"];
    fields(image, &["requested", "path", "resolved", "sha256"])?;
    require!(
        text(image, "requested")? == text(&receipt, "executable")?,
        "foreign requested process image"
    );
    let image_bytes = bytes(Path::new(text(image, "resolved")?))?;
    require!(
        hash(&image_bytes) == text(image, "sha256")?,
        "process image digest differs"
    );
    let stdout = bytes(&directory.join("stdout.bin"))?;
    for (field, stream) in [
        ("stdout", stdout.clone()),
        ("stderr", bytes(&directory.join("stderr.bin"))?),
    ] {
        fields(&receipt[field], &["bytes", "sha256"])?;
        require!(
            receipt[field]["bytes"].as_u64() == Some(stream.len() as u64)
                && text(&receipt[field], "sha256")? == hash(&stream),
            "whole process stream differs"
        );
    }
    Ok((receipt, stdout))
}
fn command(process: &Value, executable: &str, args: Value, root: &Path) -> Result<(), String> {
    require!(
        text(process, "executable")? == executable
            && process["args"] == args
            && text(process, "cwd")?
                == root
                    .to_str()
                    .ok_or("source harness: invalid process root")?,
        "foreign process recipe or source root"
    );
    Ok(())
}
fn pinned_image(process: &Value, executable: &str) -> Result<(), String> {
    let path = Path::new(executable);
    require!(path.is_absolute(), "nonabsolute pinned build tool");
    let resolved = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let image = &process["imageBefore"];
    require!(
        text(image, "path")? == executable
            && Path::new(text(image, "resolved")?) == resolved.as_path(),
        "foreign resolved build tool image"
    );
    Ok(())
}
#[derive(Clone, PartialEq, Eq)]
struct GitFile {
    mode: String,
    blob: String,
}
fn git_tree(raw: &[u8]) -> Result<BTreeMap<String, GitFile>, String> {
    let mut files = BTreeMap::new();
    for line in std::str::from_utf8(raw).map_err(|e| e.to_string())?.lines() {
        let (entry, file) = line
            .split_once('\t')
            .ok_or("source harness: invalid Git source entry")?;
        let pieces = entry.split_whitespace().collect::<Vec<_>>();
        require!(
            pieces.len() == 3 && matches!(pieces[1], "blob" | "commit"),
            "invalid Git source object"
        );
        require!(
            pieces[2].len() == 40
                && pieces[2]
                    .bytes()
                    .all(|x| x.is_ascii_hexdigit() && !x.is_ascii_uppercase()),
            "invalid Git blob digest"
        );
        require!(
            files
                .insert(
                    file.to_owned(),
                    GitFile {
                        mode: pieces[0].into(),
                        blob: pieces[2].into()
                    }
                )
                .is_none(),
            "duplicate Git source path"
        );
    }
    Ok(files)
}
fn source(root: &Path, tree: &BTreeMap<String, GitFile>, file: &str) -> Result<Vec<u8>, String> {
    let entry = tree
        .get(file)
        .ok_or_else(|| format!("source harness: missing pinned source {file}"))?;
    require!(entry.mode == "100644", "unexpected source mode");
    let body = bytes(&root.join(file))?;
    require!(
        blob(&body) == entry.blob,
        "source differs from exact Git blob"
    );
    Ok(body)
}
// Measurements use the contributor pin. Each arm authenticates its complete
// Git blob before comparing the unchanged measurement recipe.
const TOOLCHAIN_FILE: &str = "rust-toolchain.toml";
const MEASUREMENT_RUST: &str = "1.99.1";
fn source_toml(
    root: &Path,
    tree: &BTreeMap<String, GitFile>,
    file: &str,
) -> Result<toml::Value, String> {
    toml::from_str(&utf8(source(root, tree, file)?)?)
        .map_err(|e| format!("source harness: invalid TOML: {e}"))
}
fn directory_files(root: &Path, path: &Path, result: &mut BTreeSet<String>) -> Result<(), String> {
    physical(path, true)?;
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let stat = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        require!(!stat.file_type().is_symlink(), "symlink protocol path");
        if stat.is_dir() {
            directory_files(root, &path, result)?;
        } else {
            physical(&path, false)?;
            result.insert(
                path.strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_str()
                    .ok_or("source harness: non-UTF8 protocol path")?
                    .to_owned(),
            );
        }
    }
    Ok(())
}
fn cargo_id(directory: &Path, version: &str) -> Result<String, String> {
    let path = directory
        .to_str()
        .ok_or("source harness: non-UTF8 Cargo path")?;
    let mut encoded = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    Ok(format!("path+file://{encoded}#{version}"))
}
// The inverse of the existing source registry, not an additional suite recipe.
// Bind its complete original source too; only its already supported storage
// target rename may choose one of the two real source paths.
fn original_specs(
    root: &Path,
    tree: &BTreeMap<String, GitFile>,
) -> Result<Vec<(String, String, String, String)>, String> {
    let pairs = [
        ("davinci_harness", "selfcheck"),
        ("vize_armature", "davinci"),
        ("vize_croquis", "davinci"),
        ("vize_atelier_core", "davinci"),
        ("vize_atelier_dom", "davinci"),
        ("vize_atelier_vapor", "davinci"),
        ("vize_atelier_ssr", "davinci"),
        ("vize_l0", "pass_runtime"),
        ("vize_l0", "fact_runtime"),
        ("vize_l1_to_l2", "l1_to_l2_storage"),
        ("vize_patina", "davinci_markup"),
        ("vize_musea", "davinci_art"),
        ("vize_glyph", "formatter_instructions"),
    ];
    let mut specs = Vec::new();
    for (pkg, bench) in pairs {
        let benches = if pkg == "vize_l1_to_l2" {
            vec![bench, "davinci_storage"]
        } else {
            vec![bench]
        };
        let paths = benches
            .iter()
            .flat_map(|bench| {
                ["tools/benchmarks/crates", "davinci", "crates"].map(|directory| {
                    (
                        bench.to_string(),
                        format!("{directory}/{pkg}/benches/{bench}.rs"),
                    )
                })
            })
            .filter(|(_, path)| tree.contains_key(path))
            .collect::<Vec<_>>();
        require!(
            paths.len() == 1,
            "original provider source is missing or ambiguous"
        );
        let (bench, path) = &paths[0];
        source(root, tree, path)?;
        specs.push((
            if pkg == "vize_glyph" {
                "formatter"
            } else {
                "level"
            }
            .into(),
            pkg.into(),
            bench.clone(),
            path.clone(),
        ));
    }
    Ok(specs)
}
struct Arm {
    root: PathBuf,
    version: String,
    tree: BTreeMap<String, GitFile>,
    protocol: BTreeMap<String, Vec<u8>>,
    workspace: toml::Value,
    harness: toml::Value,
    external: BTreeMap<String, toml::Value>,
    cargo_image: Value,
    rustc_image: Value,
    provider_specs: Vec<(String, String, String, String)>,
    binaries: Vec<Value>,
}
fn arm(out: &Path, measurement: &Value, name: &str, head_root: &Path) -> Result<Arm, String> {
    let witness = &measurement["source_harness"][name];
    fields(
        witness,
        &[
            "root",
            "version",
            "package_id",
            "manifest_path",
            "manifest_sha256",
            "workspace_manifest_sha256",
            "library_source",
            "library_source_sha256",
            "source_pin",
        ],
    )?;
    let root = PathBuf::from(text(witness, "root")?);
    physical(&root, true)?;
    let provenance = out.join(name).join("provenance");
    let build = out.join(name).join("build");
    let (head_process, head) = completed(&provenance.join("head"))?;
    command(
        &head_process,
        "git",
        serde_json::json!(["rev-parse", "HEAD"]),
        &root,
    )?;
    require!(
        text(&head_process, "cwd")? == root.to_str().ok_or("source harness: invalid root")?
            && utf8(head)?.trim() == text(&measurement["source"], name)?,
        "foreign source revision"
    );
    let (status_process, status) = completed(&provenance.join("status-before"))?;
    command(
        &status_process,
        "git",
        serde_json::json!(["status", "--porcelain", "--untracked-files=all"]),
        &root,
    )?;
    require!(status.is_empty(), "source was not clean before build");
    let (tree_process, listing) = completed(&provenance.join("whole-tree"))?;
    command(
        &tree_process,
        "git",
        serde_json::json!(["ls-tree", "-r", "--full-tree", "HEAD"]),
        &root,
    )?;
    let tree = git_tree(&listing)?;
    let workspace = source_toml(&root, &tree, "Cargo.toml")?;
    let harness_path = "tools/benchmarks/crates/davinci_harness/Cargo.toml";
    let library_path = "tools/benchmarks/crates/davinci_harness/src/lib.rs";
    let harness = source_toml(&root, &tree, harness_path)?;
    require!(
        toml_at(&harness, &["package", "version", "workspace"])?.as_bool() == Some(true),
        "unsupported harness version inheritance"
    );
    let version = toml_at(&workspace, &["workspace", "package", "version"])?
        .as_str()
        .ok_or("source harness: missing actual workspace version")?
        .to_owned();
    let before = json(&build.join("harness-source-before.json"))?;
    fields(&before, &["root", "files"])?;
    require!(
        &before == &witness["source_pin"]
            && text(&before, "root")? == root.to_str().ok_or("source harness: invalid root")?,
        "missing or changed pre-build source pin"
    );
    let pins = before["files"]
        .as_array()
        .ok_or("source harness: missing source pins")?;
    require!(pins.len() == 3, "source pin population differs");
    for (pin, file) in pins.iter().zip(["Cargo.toml", harness_path, library_path]) {
        fields(pin, &["path", "git_blob", "sha256"])?;
        let body = source(&root, &tree, file)?;
        require!(
            text(pin, "path")? == file
                && text(pin, "git_blob")? == blob(&body)
                && text(pin, "sha256")? == hash(&body),
            "wrong source pin body or path"
        );
    }
    let (cargo_resolve, cargo_path) = completed(&build.join("resolve-cargo"))?;
    command(
        &cargo_resolve,
        "rustup",
        serde_json::json!(["which", "--toolchain", MEASUREMENT_RUST, "cargo"]),
        &root,
    )?;
    let cargo_path = utf8(cargo_path)?.trim().to_owned();
    let (rustc_resolve, rustc_path) = completed(&build.join("resolve-rustc"))?;
    command(
        &rustc_resolve,
        "rustup",
        serde_json::json!(["which", "--toolchain", MEASUREMENT_RUST, "rustc"]),
        &root,
    )?;
    let rustc_path = utf8(rustc_path)?.trim().to_owned();
    let (rustc_process, rustc) = completed(&build.join("rustc"))?;
    command(
        &rustc_process,
        &rustc_path,
        serde_json::json!(["-Vv"]),
        &root,
    )?;
    pinned_image(&rustc_process, &rustc_path)?;
    require!(
        text(&rustc_process, "executable")? == rustc_path
            && utf8(rustc)?.starts_with(&format!("rustc {MEASUREMENT_RUST} ")),
        "wrong build compiler"
    );
    let (metadata_process, metadata_stdout) = completed(&build.join("cargo-metadata"))?;
    require!(
        metadata_process["args"]
            == serde_json::json!(["metadata", "--locked", "--no-deps", "--format-version", "1"]),
        "changed Cargo metadata recipe"
    );
    let (build_process, build_stdout) = completed(&build.join("cargo"))?;
    require!(
        metadata_process["imageBefore"] == build_process["imageBefore"],
        "changed metadata/build Cargo image"
    );
    for process in [&metadata_process, &build_process] {
        require!(
            text(process, "executable")? == cargo_path
                && text(process, "cwd")? == root.to_str().ok_or("source harness: invalid root")?,
            "foreign source Cargo process"
        );
        pinned_image(process, &cargo_path)?;
        for (key, expected) in [
            ("RUSTC", rustc_path.as_str()),
            ("RUSTUP_TOOLCHAIN", MEASUREMENT_RUST),
            ("RUSTFLAGS", "-C target-cpu=x86-64"),
            ("CARGO_INCREMENTAL", "0"),
            ("RUSTC_WRAPPER", ""),
            ("RUSTC_WORKSPACE_WRAPPER", ""),
        ] {
            require!(
                text(&process["recipeEnvironment"], key)? == expected,
                "changed source build environment"
            );
        }
    }
    let metadata: Value = serde_json::from_slice(&metadata_stdout).map_err(|e| e.to_string())?;
    require!(
        text(&metadata, "workspace_root")?
            == root.to_str().ok_or("source harness: invalid root")?,
        "foreign Cargo metadata workspace"
    );
    let packages = metadata["packages"]
        .as_array()
        .ok_or("source harness: missing Cargo packages")?
        .iter()
        .filter(|row| row["name"] == "davinci_harness")
        .collect::<Vec<_>>();
    require!(packages.len() == 1, "missing or duplicate harness metadata");
    let manifest = root.join(harness_path);
    let library = root.join(library_path);
    let package_id = cargo_id(
        manifest
            .parent()
            .ok_or("source harness: missing manifest parent")?,
        &version,
    )?;
    require!(
        text(packages[0], "version")? == version
            && text(packages[0], "id")? == package_id
            && text(packages[0], "manifest_path")?
                == manifest
                    .to_str()
                    .ok_or("source harness: invalid manifest")?,
        "metadata differs from actual source inheritance"
    );
    let messages = utf8(build_stdout)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let libraries = messages
        .iter()
        .filter(|row| {
            row["reason"] == "compiler-artifact"
                && row["manifest_path"] == manifest.to_str().unwrap_or("")
                && row["target"]["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|kind| kind == "lib"))
        })
        .collect::<Vec<_>>();
    require!(
        libraries.len() == 1
            && libraries[0]["package_id"] == package_id
            && libraries[0]["target"]["src_path"] == library.to_str().unwrap_or(""),
        "missing or foreign compiled harness library"
    );
    require!(
        text(witness, "version")? == version
            && text(witness, "package_id")? == package_id
            && text(witness, "manifest_path")? == manifest.to_str().unwrap_or("")
            && text(witness, "library_source")? == library.to_str().unwrap_or(""),
        "source witness differs from metadata/artifact"
    );
    for (field, file) in [
        ("workspace_manifest_sha256", "Cargo.toml"),
        ("manifest_sha256", harness_path),
        ("library_source_sha256", library_path),
    ] {
        require!(
            text(witness, field)? == hash(&source(&root, &tree, file)?),
            "source witness digest differs"
        );
    }
    require!(
        json(&build.join("harness-source.json"))? == *witness,
        "changed retained build witness"
    );
    let binaries = json(&build.join("binaries.json"))?;
    let binaries = binaries
        .as_array()
        .ok_or("source harness: missing provider images")?;
    require!(binaries.len() == 13, "original13 provider images required");
    let mut protocol = BTreeMap::new();
    let registry_path = "tools/benchmarks/scripts/instruction-counts-suites.mjs";
    protocol.insert(registry_path.into(), source(&root, &tree, registry_path)?);
    for prefix in [
        "tools/benchmarks/crates/davinci_harness/",
        "tests/davinci_test_support/",
    ] {
        let expected = tree
            .keys()
            .filter(|file| file.starts_with(prefix))
            .cloned()
            .collect::<BTreeSet<_>>();
        require!(!expected.is_empty(), "missing complete protocol tree");
        let mut actual = BTreeSet::new();
        directory_files(&root, &root.join(prefix), &mut actual)?;
        require!(actual == expected, "extra or missing protocol path");
        for file in expected {
            protocol.insert(file.clone(), source(&root, &tree, &file)?);
        }
    }
    let target = head_root
        .join("target")
        .join(format!("level-memory-{name}"));
    let mut expected_args = vec![
        "bench".to_owned(),
        "--locked".into(),
        "--no-run".into(),
        "--profile".into(),
        "ci-opt".into(),
        "--features".into(),
        "davinci_harness/instruction-counts".into(),
        "--message-format=json".into(),
        "--target-dir".into(),
        target
            .to_str()
            .ok_or("source harness: invalid target")?
            .into(),
    ];
    let mut packages = BTreeSet::new();
    let mut benches = BTreeSet::new();
    let mut provider_paths = BTreeSet::new();
    let mut provider_specs = Vec::new();
    for row in binaries {
        require!(
            row["harness_source"] == *witness,
            "provider differs from source witness"
        );
        let provider = PathBuf::from(text(row, "source")?);
        let relative = provider
            .strip_prefix(&root)
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("source harness: invalid provider path")?;
        require!(
            provider_paths.insert(relative.to_owned()),
            "duplicate provider source"
        );
        require!(
            text(row, "source_sha256")? == hash(&source(&root, &tree, relative)?),
            "changed provider body"
        );
        protocol.insert(relative.into(), source(&root, &tree, relative)?);
        provider_specs.push((
            text(row, "plane")?.into(),
            text(row, "pkg")?.into(),
            text(row, "bench")?.into(),
            relative.into(),
        ));
        let image = PathBuf::from(text(row, "executable")?);
        require!(
            image.starts_with(&target) && text(row, "sha256")? == hash(&bytes(&image)?),
            "foreign or changed provider image"
        );
        let artifacts = messages
            .iter()
            .filter(|artifact| {
                artifact["reason"] == "compiler-artifact"
                    && artifact["target"]["src_path"] == provider.to_str().unwrap_or("")
                    && artifact["executable"] == image.to_str().unwrap_or("")
            })
            .collect::<Vec<_>>();
        let provider_manifest = provider
            .parent()
            .and_then(Path::parent)
            .ok_or("source harness: missing provider manifest")?
            .join("Cargo.toml");
        let provider_id = cargo_id(
            provider_manifest
                .parent()
                .ok_or("source harness: missing provider package")?,
            &version,
        )?;
        require!(
            artifacts.len() == 1
                && artifacts[0]["target"]["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bench"))
                && artifacts[0]["target"]["name"] == text(row, "bench")?
                && artifacts[0]["manifest_path"] == provider_manifest.to_str().unwrap_or("")
                && artifacts[0]["package_id"] == provider_id,
            "missing or foreign source-built provider artifact"
        );
        let pkg = text(row, "pkg")?.to_owned();
        if packages.insert(pkg.clone()) {
            expected_args.extend(["-p".into(), pkg]);
        }
    }
    for row in binaries {
        let bench = text(row, "bench")?.to_owned();
        if benches.insert(bench.clone()) {
            expected_args.extend(["--bench".into(), bench]);
        }
    }
    require!(
        build_process["args"] == serde_json::json!(expected_args),
        "changed provider build recipe"
    );
    require!(
        provider_specs == original_specs(&root, &tree)?,
        "changed original suite registry selection"
    );
    let (after_process, after) = completed(&out.join(name).join("source-after"))?;
    command(
        &after_process,
        "git",
        serde_json::json!(["diff", "--name-only", "HEAD"]),
        &root,
    )?;
    require!(
        utf8(after)?
            .lines()
            .all(|file| file.starts_with("tools/benchmarks/results/davinci/")),
        "source changed after measurements"
    );
    let lock = source_toml(&root, &tree, "Cargo.lock")?;
    let mut external = BTreeMap::new();
    for pkg in toml_at(&lock, &["package"])?
        .as_array()
        .ok_or("source harness: missing lock packages")?
        .iter()
        .filter(|pkg| pkg.get("source").is_some())
    {
        let key = format!(
            "{}:{}:{}",
            pkg.get("name")
                .and_then(toml::Value::as_str)
                .ok_or("source harness: missing lock name")?,
            pkg.get("version")
                .and_then(toml::Value::as_str)
                .ok_or("source harness: missing lock version")?,
            pkg.get("source")
                .and_then(toml::Value::as_str)
                .ok_or("source harness: missing lock source")?
        );
        require!(
            external.insert(key, pkg.clone()).is_none(),
            "duplicate external lock identity"
        );
    }
    let toolchain = source_toml(&root, &tree, TOOLCHAIN_FILE)?;
    require!(
        toml_at(&toolchain, &["toolchain", "channel"])?.as_str() == Some(MEASUREMENT_RUST),
        "foreign source toolchain pin"
    );
    Ok(Arm {
        root,
        version,
        tree,
        protocol,
        workspace,
        harness,
        external,
        cargo_image: metadata_process["imageBefore"].clone(),
        rustc_image: rustc_process["imageBefore"].clone(),
        provider_specs,
        binaries: binaries.to_vec(),
    })
}
fn toml_at<'a>(value: &'a toml::Value, keys: &[&str]) -> Result<&'a toml::Value, String> {
    let mut current = value;
    for key in keys {
        current = current
            .get(*key)
            .ok_or_else(|| format!("source harness: missing TOML {key}"))?;
    }
    Ok(current)
}
fn workspace_dependencies(table: &toml::Value, keys: &mut BTreeSet<String>) {
    if let Some(dependencies) = table.as_table() {
        for (name, value) in dependencies {
            if value.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                keys.insert(name.clone());
            }
        }
    }
}
fn comparable(base: &Arm, head: &Arm) -> Result<(), String> {
    require!(
        base.protocol == head.protocol
            && base.harness == head.harness
            && base.external == head.external
            && base.provider_specs == head.provider_specs,
        "changed reporter/scenario/provider/protocol/dependency"
    );
    require!(
        base.cargo_image == head.cargo_image && base.rustc_image == head.rustc_image,
        "changed paired build tool images"
    );
    require!(
        toml_at(&base.workspace, &["profile"])? == toml_at(&head.workspace, &["profile"])?,
        "changed paired build profile"
    );
    require!(
        source(&base.root, &base.tree, TOOLCHAIN_FILE)?
            == source(&head.root, &head.tree, TOOLCHAIN_FILE)?,
        "changed protocol toolchain"
    );
    for file in [".cargo/config", ".cargo/config.toml"] {
        require!(
            base.tree.contains_key(file) == head.tree.contains_key(file),
            "changed paired Cargo configuration"
        );
        if base.tree.contains_key(file) {
            require!(
                source(&base.root, &base.tree, file)? == source(&head.root, &head.tree, file)?,
                "changed paired Cargo configuration"
            );
        }
    }
    let mut a = toml_at(&base.workspace, &["workspace", "package"])?.clone();
    let mut b = toml_at(&head.workspace, &["workspace", "package"])?.clone();
    a.as_table_mut()
        .ok_or("source harness: missing workspace package")?
        .remove("version");
    b.as_table_mut()
        .ok_or("source harness: missing workspace package")?
        .remove("version");
    require!(a == b, "changed inherited harness package contract");
    let mut dependencies = BTreeSet::new();
    for field in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(value) = base.harness.get(field) {
            workspace_dependencies(value, &mut dependencies);
        }
    }
    if let Some(targets) = base.harness.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            for field in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(value) = target.get(field) {
                    workspace_dependencies(value, &mut dependencies);
                }
            }
        }
    }
    for name in dependencies {
        let mut a = toml_at(&base.workspace, &["workspace", "dependencies"])?
            .get(&name)
            .ok_or("source harness: missing baseline workspace dependency")?
            .clone();
        let mut b = toml_at(&head.workspace, &["workspace", "dependencies"])?
            .get(&name)
            .ok_or("source harness: missing candidate workspace dependency")?
            .clone();
        if ["vize_l0", "vize_l1", "vize_l1_to_l2", "vize_l2"].contains(&name.as_str()) {
            let base_requirement = format!("={}", base.version);
            let head_requirement = format!("={}", head.version);
            require!(
                toml_at(&base.harness, &["dev-dependencies"])?
                    .get(&name)
                    .is_some()
                    && a.get("version").and_then(toml::Value::as_str)
                        == Some(base_requirement.as_str())
                    && b.get("version").and_then(toml::Value::as_str)
                        == Some(head_requirement.as_str()),
                "foreign dev-product release version"
            );
            a.as_table_mut()
                .ok_or("source harness: wrong dependency shape")?
                .remove("version");
            b.as_table_mut()
                .ok_or("source harness: wrong dependency shape")?
                .remove("version");
        }
        require!(a == b, "changed protocol workspace dependency");
    }
    Ok(())
}
fn role(value: &Value) -> Result<(String, usize), String> {
    fields(value, &["arm", "repetition"])?;
    let arm = text(value, "arm")?;
    require!(["base", "head"].contains(&arm), "foreign report arm");
    let repetition = value["repetition"]
        .as_u64()
        .ok_or("source harness: missing repetition")?;
    require!((1..=3).contains(&repetition), "foreign report repetition");
    Ok((arm.into(), repetition as usize))
}
fn completed_normal_rows(
    out: &Path,
    run: &Path,
    source_arm: &Arm,
    rows: &Value,
    formatter_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, PathBuf>, String> {
    let mut raw_paths = BTreeMap::new();
    for (index, binary) in source_arm.binaries.iter().enumerate() {
        let directory = run.join(format!("suite-{index}"));
        let (process, _) = completed(&directory.join("process"))?;
        let probe = out
            .parent()
            .ok_or("source harness: missing probe root")?
            .join("level-memory-probes")
            .join(format!("probe-{index:02}"));
        command(
            &process,
            probe.to_str().ok_or("source harness: invalid probe path")?,
            serde_json::json!(["--bench", "--quick"]),
            &source_arm.root,
        )?;
        require!(
            text(&process["imageBefore"], "resolved")? == text(binary, "executable")?
                && text(&process["imageBefore"], "sha256")? == text(binary, "sha256")?,
            "normal process differs from source-built image"
        );
        let environment = &process["recipeEnvironment"];
        fields(
            environment,
            &[
                "PATH",
                "LANG",
                "LC_ALL",
                "MIMALLOC_RESERVE_OS_MEMORY",
                "MIMALLOC_PURGE_DELAY",
                "MIMALLOC_VERBOSE",
                "GLIBC_TUNABLES",
            ],
        )?;
        for (key, value) in [
            ("PATH", "/usr/bin:/bin"),
            ("LANG", "C"),
            ("LC_ALL", "C"),
            ("MIMALLOC_RESERVE_OS_MEMORY", "128MiB"),
            ("MIMALLOC_PURGE_DELAY", "-1"),
            ("MIMALLOC_VERBOSE", "1"),
            (
                "GLIBC_TUNABLES",
                "glibc.cpu.hwcaps=-AVX,-AVX2,-AVX512F,-AVX512VL,-AVX512BW,-AVX_Fast_Unaligned_Load,-ERMS,-FSRM,-Prefer_ERMS,-Prefer_FSRM,-SSSE3,-SSE4_1,-SSE4_2",
            ),
        ] {
            require!(
                text(environment, key)? == value,
                "changed normal allocator/environment recipe"
            );
        }
        let stderr = utf8(bytes(&directory.join("process/stderr.bin"))?)?;
        require!(
            stderr.contains("option 'reserve_os_memory': 131072 KiB")
                && stderr
                    .split("option 'purge_delay': -1")
                    .nth(1)
                    .and_then(|rest| rest.chars().next())
                    .is_some_and(char::is_whitespace)
                && !stderr
                    .lines()
                    .any(|line| line.starts_with("VIZE_INSTRUCTION_BENCH ")
                        || line.starts_with("VIZE_INSTRUCTION_ALLOCATOR ")),
            "normal allocator protocol missing or instruction mode leaked"
        );
        let suite_rows = json(&directory.join("rows.json"))?;
        let suite_rows = suite_rows
            .as_object()
            .ok_or("source harness: missing completed suite rows")?;
        require!(!suite_rows.is_empty(), "empty completed provider");
        let mut emitted = BTreeSet::new();
        for line in stderr
            .lines()
            .filter_map(|line| line.strip_prefix("davinci-harness: wrote "))
        {
            require!(
                emitted.insert(PathBuf::from(line)),
                "duplicate actual report marker"
            );
        }
        let mut expected = BTreeSet::new();
        for (id, row) in suite_rows {
            require!(
                rows.get(id) == Some(row)
                    && formatter_ids.contains(id) == (binary["plane"] == "formatter"),
                "foreign completed suite row or plane"
            );
            expected.insert(
                source_arm
                    .root
                    .join("tools/benchmarks/results/davinci")
                    .join(format!("{id}.json")),
            );
            require!(
                raw_paths
                    .insert(
                        id.clone(),
                        directory.join("reports").join(format!("{id}.json"))
                    )
                    .is_none(),
                "duplicate completed suite report"
            );
        }
        require!(
            emitted == expected,
            "missing extra or foreign actual report marker"
        );
    }
    Ok(raw_paths)
}
pub fn load(
    pair_file: &Path,
    budgets: &Path,
    baseline: &Path,
    current: &Path,
    level_ids: &BTreeSet<String>,
) -> Result<ValidatedPair, String> {
    let pair = json(pair_file)?;
    fields(
        &pair,
        &["schema_version", "measurement", "baseline", "current"],
    )?;
    require!(pair["schema_version"] == 1, "wrong pair schema");
    let measurement_file = PathBuf::from(text(&pair, "measurement")?);
    let out = measurement_file
        .parent()
        .ok_or("source harness: missing evidence root")?;
    require!(
        measurement_file == out.join("measurement.json")
            && pair_file.parent() == Some(out.join("source-harness-pairs").as_path()),
        "foreign evidence pair path"
    );
    let measurement = json(&measurement_file)?;
    require!(
        measurement["observations_complete"] == true && level_ids.len() == 101,
        "complete630 and complete101 required"
    );
    for side in ["base", "head"] {
        let revision = text(&measurement["source"], side)?;
        require!(
            revision.len() == 40
                && revision
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "invalid source revision"
        );
    }
    require!(
        measurement["source"]["base"] != measurement["source"]["head"],
        "source arms must differ"
    );
    let head_root = PathBuf::from(text(&measurement["source_harness"]["head"], "root")?);
    let base = arm(out, &measurement, "base", &head_root)?;
    let head = arm(out, &measurement, "head", &head_root)?;
    require!(base.root != head.root, "foreign equal source roots");
    comparable(&base, &head)?;
    require!(
        fs::canonicalize(budgets).map_err(|e| e.to_string())?
            == head.root.join("docs/davinci/plan/budgets.toml"),
        "foreign original budget source"
    );
    for file in [
        "budgets.toml",
        "instruction-budgets.toml",
        "formatter-instruction-registry.toml",
        "formatter-instruction-budgets.toml",
    ] {
        let path = format!("docs/davinci/plan/{file}");
        require!(
            source(&base.root, &base.tree, &path)? == source(&head.root, &head.tree, &path)?,
            "changed original population/caps"
        );
    }
    let formatter = source_toml(
        &head.root,
        &head.tree,
        "docs/davinci/plan/formatter-instruction-registry.toml",
    )?;
    let formatter_ids = toml_at(&formatter, &["bench"])?
        .as_table()
        .ok_or("source harness: missing formatter plane")?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    require!(
        formatter_ids.len() == 4 && level_ids.is_disjoint(&formatter_ids),
        "wrong original formatter plane"
    );
    let ids = level_ids
        .union(&formatter_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut identities = BTreeMap::new();
    for file in [
        "instruction-budgets.toml",
        "formatter-instruction-budgets.toml",
    ] {
        let authority = source_toml(&head.root, &head.tree, &format!("docs/davinci/plan/{file}"))?;
        for (id, value) in toml_at(&authority, &["instruction"])?
            .as_table()
            .ok_or("source harness: missing identity plane")?
        {
            require!(
                identities.insert(id.clone(), value.clone()).is_none(),
                "overlapping identity planes"
            );
        }
    }
    require!(
        identities.keys().cloned().collect::<BTreeSet<_>>() == ids,
        "missing or extra original identities"
    );
    for (name, source_arm) in [("base", &base), ("head", &head)] {
        let runs = measurement["measurements"][name]
            .as_array()
            .ok_or("source harness: missing measurement arm")?;
        require!(runs.len() == 3, "incomplete repetitions");
        for (index, rows) in runs.iter().enumerate() {
            let object = rows
                .as_object()
                .ok_or("source harness: missing complete rows")?;
            require!(
                object.keys().cloned().collect::<BTreeSet<_>>() == ids,
                "missing extra or wrong-plane reports"
            );
            let run = out.join(name).join(format!("run-{}", index + 1));
            let raw_paths = completed_normal_rows(out, &run, source_arm, rows, &formatter_ids)?;
            require!(
                raw_paths.keys().cloned().collect::<BTreeSet<_>>() == ids,
                "incomplete completed provider population"
            );
            let reports = super::load_reports(&run.join("reports"), "memory").map_err(|e| e.0)?;
            require!(
                reports.keys().cloned().collect::<BTreeSet<_>>() == ids,
                "incomplete raw105 reports"
            );
            for (id, row) in object {
                fields(
                    row,
                    &["report", "identity", "raw_sha256", "file", "raw_file"],
                )?;
                fields(
                    &row["identity"],
                    &["bench_id", "fixture", "fixture_sha256", "window"],
                )?;
                require!(
                    row["identity"]["bench_id"] == *id
                        && row["identity"]["fixture"] == row["report"]["fixture"]
                        && row["identity"]["fixture"]
                            == toml_at(&identities[id], &["fixture"])?
                                .as_str()
                                .ok_or("source harness: missing original fixture name")?
                        && row["identity"]["fixture_sha256"]
                            == toml_at(&identities[id], &["fixture_sha256"])?
                                .as_str()
                                .ok_or("source harness: missing original fixture digest")?
                        && row["identity"]["window"]
                            == toml_at(&identities[id], &["window"])?
                                .as_str()
                                .ok_or("source harness: missing original window")?,
                    "changed original input/window identity"
                );
                require!(
                    text(row, "file")?
                        == source_arm
                            .root
                            .join("tools/benchmarks/results/davinci")
                            .join(format!("{id}.json"))
                            .to_str()
                            .unwrap_or(""),
                    "foreign original report source"
                );
                let raw = PathBuf::from(text(row, "raw_file")?);
                require!(
                    raw_paths.get(id) == Some(&raw),
                    "foreign completed raw report path"
                );
                let raw_body = bytes(&raw)?;
                let copied = bytes(&run.join("reports").join(format!("{id}.json")))?;
                require!(
                    raw_body == copied
                        && text(row, "raw_sha256")? == hash(&raw_body)
                        && serde_json::from_slice::<Value>(&raw_body).map_err(|e| e.to_string())?
                            == row["report"],
                    "changed raw report custody"
                );
                let report = &reports[id];
                require!(
                    report.harness_version == source_arm.version
                        && report.platform == "linux"
                        && report.allocs.is_some()
                        && report.alloc_bytes_peak.is_some()
                        && report.rss_peak_bytes.is_some(),
                    "report differs from own source version/platform/metrics"
                );
            }
            let level =
                super::load_reports(&run.join("level-reports"), "memory-level").map_err(|e| e.0)?;
            require!(
                level.keys().cloned().collect::<BTreeSet<_>>() == *level_ids,
                "wrong complete101 judge plane"
            );
            for id in level_ids {
                require!(
                    bytes(&run.join("level-reports").join(format!("{id}.json")))?
                        == bytes(&run.join("reports").join(format!("{id}.json")))?,
                    "changed judge report body"
                );
            }
        }
    }
    let baseline_role = role(&pair["baseline"])?;
    let current_role = role(&pair["current"])?;
    require!(
        baseline_role.0 == "base" && baseline_role.1 == current_role.1,
        "foreign baseline role or unmatched repetition"
    );
    for (argument, (name, repetition)) in [(baseline, &baseline_role), (current, &current_role)] {
        require!(
            fs::canonicalize(argument).map_err(|e| e.to_string())?
                == out
                    .join(name)
                    .join(format!("run-{repetition}"))
                    .join("level-reports"),
            "swapped or foreign copied report arm"
        );
    }
    let current_version = if current_role.0 == "base" {
        base.version.clone()
    } else {
        head.version
    };
    Ok(ValidatedPair {
        baseline: base.version,
        current: current_version,
    })
}
