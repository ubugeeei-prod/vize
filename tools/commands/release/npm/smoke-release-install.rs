#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tempfile = "3"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/release/npm_publish.rs"]
mod npm_publish;

use serde_json::{Map, Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

const DEPENDENCY_SECTIONS: &[&str] = &[
    "dependencies",
    "optionalDependencies",
    "peerDependencies",
    "devDependencies",
];
const RUNTIME_PEER_DEPENDENCIES: &[(&str, &str)] = &[
    ("typescript", "6.0.3"),
    ("vite", "^8.0.0"),
    ("vite-plus", "0.1.24"),
    ("vue", "3.5.34"),
];

#[derive(Debug)]
struct Options {
    content_mapper_checks: bool,
    keep_temp: bool,
    package_dirs: Vec<PathBuf>,
    prepare_manifests: bool,
    runtime_checks: bool,
}

#[derive(Clone, Debug)]
struct PackageInfo {
    compatible: bool,
    name: String,
    tarball: PathBuf,
    version: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let options = parse_args(env::args_os().skip(1).collect())?;
    let temp_dir = canonical_temp_dir()?;
    let pack_dir = temp_dir.join("packs");
    fs::create_dir_all(&pack_dir)
        .map_err(|error| format!("cannot create {}: {error}", pack_dir.display()))?;
    let result = run_with_temp(&options, &temp_dir, &pack_dir);
    if options.keep_temp {
        println!("kept {}", temp_dir.display());
    } else {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    result
}

fn parse_args(argv: Vec<std::ffi::OsString>) -> Result<Options, String> {
    let mut options = Options {
        content_mapper_checks: false,
        keep_temp: false,
        package_dirs: Vec::new(),
        prepare_manifests: false,
        runtime_checks: false,
    };
    for arg in argv {
        let arg = arg.to_string_lossy().into_owned();
        match arg.as_str() {
            "--content-mapper-checks" => options.content_mapper_checks = true,
            "--keep-temp" => options.keep_temp = true,
            "--prepare-manifests" => options.prepare_manifests = true,
            "--runtime-checks" => options.runtime_checks = true,
            value if value.starts_with("--") => return Err(format!("Unknown argument: {value}")),
            value => options.package_dirs.push(
                PathBuf::from(value)
                    .canonicalize()
                    .unwrap_or_else(|_| PathBuf::from(value)),
            ),
        }
    }
    if options.package_dirs.is_empty() {
        return Err("Usage: rust-script tools/commands/release/npm/smoke-release-install.rs [--prepare-manifests] [--runtime-checks] [--content-mapper-checks] [--keep-temp] <package-dir>...".to_string());
    }
    Ok(options)
}

fn run_with_temp(options: &Options, temp_dir: &Path, pack_dir: &Path) -> Result<(), String> {
    let mut packages = Vec::new();
    for package_dir in &options.package_dirs {
        if !package_dir.join("package.json").is_file() {
            return Err(format!(
                "{} does not contain package.json",
                package_dir.display()
            ));
        }
        if options.prepare_manifests {
            npm_publish::prepare_publish_manifest(package_dir)?;
        }
        let package_json = read_package_json(package_dir)?;
        assert_no_workspace_protocols(package_dir, &package_json)?;
        let compatible = is_compatible_with_current_runner(&package_json);
        let is_platform_specific_subpackage =
            package_json.get("os").and_then(Value::as_array).is_some()
                || package_json.get("cpu").and_then(Value::as_array).is_some();
        if !is_platform_specific_subpackage {
            assert_publish_entrypoints_exist(package_dir, &package_json)?;
        }
        let tarball = pack_package(package_dir, pack_dir)?;
        let name = package_json
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "{} is missing name",
                    package_dir.join("package.json").display()
                )
            })?
            .to_string();
        let version = package_json
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "{} is missing version",
                    package_dir.join("package.json").display()
                )
            })?
            .to_string();
        packages.push(PackageInfo {
            compatible,
            name: name.clone(),
            tarball,
            version: version.clone(),
        });
        println!(
            "{}: {name}@{version}",
            if compatible { "install" } else { "pack-only" }
        );
    }

    let installable = packages
        .iter()
        .filter(|package| package.compatible)
        .cloned()
        .collect::<Vec<_>>();
    if installable.is_empty() {
        return Err("no package tarballs are compatible with this runner".to_string());
    }
    let include_runtime_peers = options.runtime_checks || options.content_mapper_checks;
    let install_dir = install_packed_packages(temp_dir, &installable, include_runtime_peers)?;
    if options.runtime_checks {
        run_runtime_checks(&install_dir, &installable, &packages, temp_dir)?;
    }
    if options.content_mapper_checks {
        run_installed_content_mapper_checks(&install_dir)?;
    }
    println!(
        "smoked {}/{} package tarballs",
        installable.len(),
        packages.len()
    );
    Ok(())
}

fn read_package_json(package_dir: &Path) -> Result<Value, String> {
    let path = package_dir.join("package.json");
    let package_json: Value = common::read_json(&path)?;
    if package_json.get("private").and_then(Value::as_bool) == Some(true) {
        return Err(format!("{} must be publishable", path.display()));
    }
    if package_json.get("name").and_then(Value::as_str).is_none() {
        return Err(format!("{} is missing name", path.display()));
    }
    if package_json
        .get("version")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err(format!("{} is missing version", path.display()));
    }
    Ok(package_json)
}

fn assert_no_workspace_protocols(package_dir: &Path, package_json: &Value) -> Result<(), String> {
    let mut unresolved = Vec::new();
    for section in DEPENDENCY_SECTIONS {
        let Some(dependencies) = package_json.get(*section).and_then(Value::as_object) else {
            continue;
        };
        for (name, version) in dependencies {
            let Some(version) = version.as_str() else {
                continue;
            };
            if version.starts_with("workspace:") || version.starts_with("catalog:") {
                unresolved.push(format!("{section}.{name}={version}"));
            }
        }
    }
    if !unresolved.is_empty() {
        return Err(format!(
            "{} is not publishable: {}",
            package_dir.join("package.json").display(),
            unresolved.join(", ")
        ));
    }
    Ok(())
}

fn assert_publish_entrypoints_exist(
    package_dir: &Path,
    package_json: &Value,
) -> Result<(), String> {
    let mut manifest_paths = Vec::new();
    for field in ["main", "types", "bin", "exports"] {
        if let Some(value) = package_json.get(field) {
            collect_strings(value, &mut manifest_paths);
        }
    }
    manifest_paths.sort();
    manifest_paths.dedup();
    let mut missing = Vec::new();
    for manifest_path in manifest_paths {
        let Some(normalized) = normalize_manifest_path(&manifest_path) else {
            continue;
        };
        if !package_dir.join(normalized).exists() {
            missing.push(manifest_path);
        }
    }
    if !missing.is_empty() {
        let name = package_json
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("package");
        return Err(format!(
            "{name} publishes missing entrypoint files: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(value) => out.push(value.clone()),
        Value::Array(values) => {
            for value in values {
                collect_strings(value, out);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_strings(value, out);
            }
        }
        _ => {}
    }
}

fn normalize_manifest_path(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || looks_like_scheme(trimmed)
    {
        return None;
    }
    Some(trimmed.strip_prefix("./").unwrap_or(trimmed).to_string())
}

fn looks_like_scheme(value: &str) -> bool {
    let Some(index) = value.find(':') else {
        return false;
    };
    let scheme = &value[..index];
    !scheme.is_empty()
        && scheme.bytes().enumerate().all(|(index, byte)| {
            if index == 0 {
                byte.is_ascii_alphabetic()
            } else {
                byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-')
            }
        })
}

fn is_compatible_with_current_runner(package_json: &Value) -> bool {
    npm_allows(package_json.get("os"), &node_platform())
        && npm_allows(package_json.get("cpu"), &node_arch())
        && npm_allows(
            package_json.get("libc"),
            current_libc().as_deref().unwrap_or(""),
        )
}

fn npm_allows(list: Option<&Value>, current: &str) -> bool {
    let Some(values) = list.and_then(Value::as_array) else {
        return true;
    };
    let items = values.iter().filter_map(Value::as_str).collect::<Vec<_>>();
    if items.iter().any(|item| *item == format!("!{current}")) {
        return false;
    }
    let positives = items
        .iter()
        .copied()
        .filter(|item| !item.starts_with('!'))
        .collect::<Vec<_>>();
    positives.is_empty() || positives.contains(&current)
}

fn node_platform() -> String {
    match env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other,
    }
    .to_string()
}

fn node_arch() -> String {
    match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "arm" => "arm",
        other => other,
    }
    .to_string()
}

fn current_libc() -> Option<String> {
    if env::consts::OS != "linux" {
        return None;
    }
    let output = Command::new("ldd")
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if text.to_lowercase().contains("musl") {
        Some("musl".to_string())
    } else {
        Some("glibc".to_string())
    }
}

fn pack_package(package_dir: &Path, pack_dir: &Path) -> Result<PathBuf, String> {
    let before = dir_entries(pack_dir)?;
    run_command(
        env::var("NPM_BIN").unwrap_or_else(|_| "npm".to_string()),
        &["pack", "--ignore-scripts", "--pack-destination"],
        Some(pack_dir),
        package_dir,
    )?;
    let mut created = dir_entries(pack_dir)?
        .into_iter()
        .map(|(entry, ())| entry)
        .filter(|entry| entry.ends_with(".tgz") && !before.contains_key(entry))
        .map(|entry| pack_dir.join(entry))
        .collect::<Vec<_>>();
    if created.len() != 1 {
        return Err(format!(
            "expected exactly one tarball from {}",
            package_dir.display()
        ));
    }
    fs::canonicalize(created.remove(0)).map_err(|error| error.to_string())
}

fn dir_entries(dir: &Path) -> Result<BTreeMap<String, ()>, String> {
    let mut entries = BTreeMap::new();
    for entry in
        fs::read_dir(dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        entries.insert(entry.file_name().to_string_lossy().into_owned(), ());
    }
    Ok(entries)
}

fn install_packed_packages(
    temp_dir: &Path,
    packages: &[PackageInfo],
    include_runtime_peers: bool,
) -> Result<PathBuf, String> {
    let install_dir = temp_dir.join("install");
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("cannot create {}: {error}", install_dir.display()))?;
    let mut dependencies = Map::new();
    for package in packages {
        dependencies.insert(
            package.name.clone(),
            Value::String(format!("file:{}", package.tarball.display())),
        );
    }
    if include_runtime_peers {
        for (name, version) in RUNTIME_PEER_DEPENDENCIES {
            dependencies.insert((*name).to_string(), Value::String((*version).to_string()));
        }
    }
    common::write_json_pretty(
        install_dir.join("package.json"),
        &json!({
            "name": "vize-release-install-smoke",
            "private": true,
            "dependencies": Value::Object(dependencies),
        }),
    )?;
    run_command(
        env::var("NPM_BIN").unwrap_or_else(|_| "npm".to_string()),
        &[
            "install",
            "--ignore-scripts",
            "--package-lock=false",
            "--no-audit",
            "--fund=false",
            "--legacy-peer-deps",
            "--include=optional",
        ],
        None,
        &install_dir,
    )?;
    let node_modules = install_dir.join("node_modules");
    for package in packages {
        assert_installed_package(&node_modules, package)?;
    }
    Ok(install_dir)
}

fn assert_installed_package(node_modules: &Path, package: &PackageInfo) -> Result<(), String> {
    let package_dir = installed_package_dir(node_modules, &package.name);
    let package_json = read_package_json(&package_dir)?;
    let name = package_json
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let version = package_json
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("");
    if name != package.name || version != package.version {
        return Err(format!(
            "installed package mismatch for {}: {name}@{version}",
            package.name
        ));
    }
    assert_no_workspace_protocols(&package_dir, &package_json)?;
    let is_platform_specific_subpackage =
        package_json.get("os").and_then(Value::as_array).is_some()
            || package_json.get("cpu").and_then(Value::as_array).is_some();
    if !is_platform_specific_subpackage {
        assert_publish_entrypoints_exist(&package_dir, &package_json)?;
    }
    if env::consts::OS != "windows" {
        for bin_name in manifest_bin_names(&package.name, package_json.get("bin")) {
            let bin_path = node_modules.join(".bin").join(&bin_name);
            if !bin_path.exists() {
                return Err(format!("{} did not install {bin_name}", package.name));
            }
            assert_executable(
                &bin_path,
                &format!("{} installed non-executable {bin_name}", package.name),
            )?;
        }
    }
    Ok(())
}

fn manifest_bin_names(package_name: &str, bin: Option<&Value>) -> Vec<String> {
    match bin {
        Some(Value::String(_)) => vec![package_name.to_string()],
        Some(Value::Object(map)) => map.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

fn installed_package_dir(node_modules: &Path, name: &str) -> PathBuf {
    if let Some((scope, package_name)) = name
        .strip_prefix('@')
        .and_then(|value| value.split_once('/'))
    {
        node_modules.join(format!("@{scope}")).join(package_name)
    } else {
        node_modules.join(name)
    }
}

fn resolve_installed_bin(
    install_dir: &Path,
    package_name: &str,
    bin_name: &str,
) -> Result<PathBuf, String> {
    let package_dir = installed_package_dir(&install_dir.join("node_modules"), package_name);
    let package_json = read_package_json(&package_dir)?;
    let relative = match package_json.get("bin") {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Object(map)) => map
            .get(bin_name)
            .or_else(|| map.get(package_name.rsplit('/').next().unwrap_or(package_name)))
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
    .ok_or_else(|| {
        format!("installed {package_name} does not expose a \"{bin_name}\" bin entry")
    })?;
    Ok(package_dir.join(relative))
}

fn run_runtime_checks(
    install_dir: &Path,
    packages: &[PackageInfo],
    all_packages: &[PackageInfo],
    temp_dir: &Path,
) -> Result<(), String> {
    let _ = (all_packages, temp_dir);
    write_runtime_smoke_project(install_dir)?;
    if packages
        .iter()
        .any(|package| package.name == "@vizejs/native")
    {
        run_node_eval(
            install_dir,
            r#"
const required = require("@vizejs/native");
(async () => {
  const imported = await import("@vizejs/native");
  const importedNative = imported.default ?? imported;
  for (const [label, native] of [["require", required], ["import", importedNative]]) {
    if (typeof native.compileSfc !== "function") throw new Error(`compileSfc missing from ${label} smoke`);
    const result = native.compileSfc(
      '<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>',
      { filename: "Smoke.vue", isTs: true },
    );
    if (!result || result.errors.length > 0 || typeof result.code !== "string" || result.code.length === 0) {
      throw new Error(`compileSfc ${label} runtime smoke failed`);
    }
  }
})().catch((error) => { console.error(error); process.exit(1); });
"#,
        )?;
        println!("runtime: @vizejs/native require/import compileSfc");
    }
    if packages.iter().any(|package| package.name == "vize") {
        let vize_bin = resolve_installed_bin(install_dir, "vize", "vize")?;
        run_command(
            env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
            &[vize_bin.to_string_lossy().as_ref(), "--version"],
            None,
            install_dir,
        )?;
        println!("runtime: vize --version");
        run_command(
            env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
            &[
                vize_bin.to_string_lossy().as_ref(),
                "check",
                "src/App.vue",
                "--format",
                "json",
                "--quiet",
                "--no-config",
            ],
            None,
            install_dir,
        )?;
        println!("runtime: vize check");
        run_command(
            env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
            &[
                vize_bin.to_string_lossy().as_ref(),
                "lint",
                "src/App.vue",
                "--format",
                "json",
                "--quiet",
                "--no-config",
            ],
            None,
            install_dir,
        )?;
        println!("runtime: vize lint");
        run_init_typecheck_checks(install_dir, &vize_bin)?;
    }
    if packages
        .iter()
        .any(|package| package.name == "@vizejs/vite-plugin")
    {
        let vite_bin = resolve_installed_bin(install_dir, "vite", "vite")?;
        run_command(
            env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
            &[vite_bin.to_string_lossy().as_ref(), "build"],
            None,
            install_dir,
        )?;
        println!("runtime: @vizejs/vite-plugin vite build");
    }
    Ok(())
}

fn write_runtime_smoke_project(install_dir: &Path) -> Result<(), String> {
    let source_dir = install_dir.join("src");
    fs::create_dir_all(&source_dir)
        .map_err(|error| format!("cannot create {}: {error}", source_dir.display()))?;
    common::write_text(
        install_dir.join("index.html"),
        "<div id=\"app\"></div><script type=\"module\" src=\"/src/main.ts\"></script>\n",
    )?;
    common::write_json_pretty(
        install_dir.join("tsconfig.json"),
        &json!({
            "compilerOptions": {
                "lib": ["ES2022", "DOM", "DOM.Iterable"],
                "module": "ESNext",
                "moduleResolution": "Bundler",
                "strict": true,
                "target": "ES2022",
                "types": [],
            },
            "include": ["src/**/*.ts", "src/**/*.vue"],
        }),
    )?;
    common::write_text(
        install_dir.join("vite.config.mjs"),
        "import { defineConfig } from \"vite\";\nimport vize from \"@vizejs/vite-plugin\";\n\nexport default defineConfig({\n  plugins: [vize()],\n  build: { outDir: \"dist\", emptyOutDir: true },\n});\n",
    )?;
    common::write_text(
        source_dir.join("App.vue"),
        "<template>\n  <button class=\"smoke\" @click=\"count++\">{{ label }} {{ count }}</button>\n</template>\n\n<script setup lang=\"ts\">\nimport { ref } from \"vue\";\n\nconst label: string = \"vize smoke\";\nconst count = ref(0);\n</script>\n\n<style scoped>\n.smoke {\n  color: #0f766e;\n}\n</style>\n",
    )?;
    common::write_text(
        source_dir.join("main.ts"),
        "import { createApp } from \"vue\";\nimport App from \"./App.vue\";\n\ncreateApp(App).mount(\"#app\");\n",
    )
}

fn run_init_typecheck_checks(install_dir: &Path, vize_bin: &Path) -> Result<(), String> {
    for language in ["typescript", "javascript"] {
        let project = install_dir.join(format!("init-smoke-{language}"));
        fs::create_dir_all(&project)
            .map_err(|error| format!("cannot create {}: {error}", project.display()))?;
        let mut dev_dependencies = Map::new();
        dev_dependencies.insert("vue".to_string(), json!("3.5.34"));
        if language == "typescript" {
            dev_dependencies.insert("typescript".to_string(), json!("6.0.3"));
        }
        common::write_json_pretty(
            project.join("package.json"),
            &json!({
                "name": format!("vize-init-{language}-smoke"),
                "private": true,
                "type": "module",
                "devDependencies": Value::Object(dev_dependencies),
            }),
        )?;
        run_command(
            env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
            &[
                vize_bin.to_string_lossy().as_ref(),
                "init",
                "--yes",
                "--typecheck",
                "--no-install",
            ],
            None,
            &project,
        )?;
        run_command(
            env::var("NPM_BIN").unwrap_or_else(|_| "npm".to_string()),
            &["run", "--silent", "vize:check"],
            None,
            &project,
        )?;
    }
    Ok(())
}

fn run_installed_content_mapper_checks(install_dir: &Path) -> Result<(), String> {
    let tsgo = env::var("VIZE_TEST_CONTENT_MAPPER_TSGO").unwrap_or_default();
    if tsgo.is_empty() {
        return Ok(());
    }
    assert_installed_mapper_contract(install_dir)?;
    let root = repo_root()?;
    let project_dir = install_dir.join("content mapper project with spaces");
    copy_dir(
        &root.join("crates/vize/tests/fixtures/content_mapper_project"),
        &project_dir,
    )?;
    let common_args = ["--runExternalCode", "--pretty", "false"];
    run_command(
        tsgo.clone(),
        &[
            common_args[0],
            common_args[1],
            common_args[2],
            "--noEmit",
            "-p",
            "tsconfig.json",
        ],
        None,
        &project_dir,
    )?;
    let broken = run_command_allow_failure(
        tsgo.clone(),
        &[
            common_args[0],
            common_args[1],
            common_args[2],
            "--noEmit",
            "-p",
            "tsconfig.error.json",
        ],
        &project_dir,
    )?;
    if broken.status == 0 {
        return Err("installed mapper error fixture unexpectedly passed".to_string());
    }
    let broken_output = format!("{}\n{}", broken.stdout, broken.stderr);
    for (file, code) in [
        ("errors/Broken.vue", "TS2322"),
        ("errors/JavaScriptConsumer.js", "TS2322"),
        ("errors/JsxConsumer.jsx", "TS2322"),
        ("src/Unused.vue", "TS6133"),
    ] {
        if !broken_output
            .lines()
            .any(|line| line.contains(file) && line.contains(code))
        {
            return Err(format!("{file} did not report {code}:\n{broken_output}"));
        }
    }
    run_command(
        tsgo.clone(),
        &[
            common_args[0],
            common_args[1],
            common_args[2],
            "-p",
            "tsconfig.emit.json",
        ],
        None,
        &project_dir,
    )?;
    if !["App.d.vue.ts", "App.vue.d.ts"]
        .iter()
        .any(|name| project_dir.join("dist").join(name).exists())
    {
        return Err("installed mapper did not emit an App.vue declaration".to_string());
    }
    if !project_dir.join("dist/main.d.ts").exists() {
        return Err("installed mapper did not emit dist/main.d.ts".to_string());
    }
    fs::copy(
        project_dir.join("consumer/verify.ts"),
        project_dir.join("dist/verify.ts"),
    )
    .map_err(|error| format!("cannot stage declaration consumer: {error}"))?;
    let consumer_args = [
        "--ignoreConfig",
        "--noEmit",
        "--strict",
        "--module",
        "preserve",
        "--moduleResolution",
        "bundler",
        "--allowArbitraryExtensions",
        "--pretty",
        "false",
        "dist/verify.ts",
    ];
    run_command(tsgo.clone(), &consumer_args, None, &project_dir)?;
    let tsc = install_dir.join("node_modules/typescript/bin/tsc");
    let mut node_args = vec![tsc.to_string_lossy().to_string()];
    node_args.extend(consumer_args.iter().map(|arg| (*arg).to_string()));
    run_command_owned(
        env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
        &node_args,
        &project_dir,
    )?;
    println!("runtime: packed vize Content Mapper check and declaration emit");
    Ok(())
}

fn assert_installed_mapper_contract(install_dir: &Path) -> Result<(), String> {
    let package_root = install_dir.join("node_modules/vize");
    if fs::symlink_metadata(&package_root)
        .map_err(|error| format!("cannot stat {}: {error}", package_root.display()))?
        .file_type()
        .is_symlink()
    {
        return Err(format!("{} must not be a symlink", package_root.display()));
    }
    let manifest = read_package_json(&package_root)?;
    let contract = manifest
        .get("typescript")
        .and_then(|value| value.get("contentMapper"));
    if contract
        != Some(&json!({
            "exec": ["node", "./bin/vize", "content-mapper"],
            "compilerOptions": ["noUnusedLocals"],
        }))
    {
        return Err(format!(
            "{} must expose the production typescript.contentMapper contract",
            package_root.join("package.json").display()
        ));
    }
    let mapper_path = package_root.join("bin/vize");
    if !mapper_path.is_file() {
        return Err(format!("{} must exist", mapper_path.display()));
    }
    if env::consts::OS != "windows" {
        assert_executable(
            &mapper_path,
            &format!("{} must be executable", mapper_path.display()),
        )?;
    }
    Ok(())
}

fn run_node_eval(cwd: &Path, script: &str) -> Result<(), String> {
    run_command(
        env::var("NODE_BIN").unwrap_or_else(|_| "node".to_string()),
        &["-e", script],
        None,
        cwd,
    )
}

fn run_command(
    command: String,
    args: &[&str],
    extra_arg: Option<&Path>,
    cwd: &Path,
) -> Result<(), String> {
    let mut owned = args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    if let Some(extra_arg) = extra_arg {
        owned.push(extra_arg.to_string_lossy().into_owned());
    }
    run_command_owned(command, &owned, cwd)
}

fn run_command_owned(command: String, args: &[String], cwd: &Path) -> Result<(), String> {
    let output = run_command_allow_failure(command.clone(), args, cwd)?;
    if output.status != 0 {
        let detail = format!("{}{}", output.stdout, output.stderr);
        return Err(format!(
            "{} failed with exit {}{}",
            common::command_line(&command, args),
            output.status,
            if detail.trim().is_empty() {
                String::new()
            } else {
                format!("\n{}", detail.trim())
            }
        )
        .trim()
        .to_string());
    }
    Ok(())
}

fn run_command_allow_failure(
    command: String,
    args: &[impl AsRef<std::ffi::OsStr>],
    cwd: &Path,
) -> Result<common::CommandOutput, String> {
    let output = Command::new(&command)
        .args(args.iter().map(AsRef::as_ref))
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {command}: {error}"))?;
    Ok(common::CommandOutput {
        status: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn canonical_temp_dir() -> Result<PathBuf, String> {
    let dir = tempfile::Builder::new()
        .prefix("vize-release-smoke-")
        .tempdir()
        .map_err(|error| format!("cannot create temp dir: {error}"))?;
    let path = dir.keep();
    fs::canonicalize(&path)
        .map_err(|error| format!("cannot canonicalize {}: {error}", path.display()))
}

fn assert_executable(path: &Path, message: &str) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path)
            .map_err(|error| format!("cannot stat {}: {error}", path.display()))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            return Err(message.to_string());
        }
    }
    Ok(())
}

fn copy_dir(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|error| format!("cannot create {}: {error}", target.display()))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("cannot read {}: {error}", source.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).map_err(|error| {
                format!(
                    "cannot copy {} to {}: {error}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn repo_root() -> Result<PathBuf, String> {
    common::repo_root().or_else(|_| {
        Path::new(file!())
            .ancestors()
            .find(|candidate| {
                candidate.join("Cargo.toml").is_file()
                    && candidate.join("pnpm-workspace.yaml").is_file()
            })
            .map(Path::to_path_buf)
            .ok_or_else(|| "cannot resolve Vize repository root from script path".to_string())
    })
}
