//! Original #7874 bytes and a separate authored direct TypeScript contract.
#![expect(clippy::disallowed_types, reason = "fixture IO uses std strings")]
#![expect(clippy::disallowed_macros, reason = "fixture paths use std formatting")]
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../../tests/_fixtures/differential/typechecker/unknown-template-options/",
            $name,
            ".txt"
        ))
    };
}
pub const CHILD: &str = source!("src/Child.vue");
pub const PARENT: &str = source!("src/Parent.vue");
pub const CONFIG: &str = source!("tsconfig.json");
const ORACLE: &str = source!("Oracle.ts");
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(std::io::Error::other(message).into())
    }
}
pub fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) -> Result<()> {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
pub fn link_vue(root: &Path) -> Result<PathBuf> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("workspace missing")?;
    let vue = workspace
        .join("tests/node_modules/vue-ssr-css-vars-oracle")
        .canonicalize()?;
    let manifest: Value = serde_json::from_slice(&std::fs::read(vue.join("package.json"))?)?;
    require(
        manifest.get("name").and_then(Value::as_str) == Some("vue"),
        "not genuine Vue",
    )?;
    require(
        manifest.get("version").and_then(Value::as_str) == Some("3.6.0-rc.10"),
        "Vue version changed",
    )?;
    std::fs::create_dir_all(root.join("node_modules"))?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(&vue, root.join("node_modules/vue"))?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&vue, root.join("node_modules/vue"))?;
    Ok(vue)
}
pub fn project(root: &Path, config: &[u8]) -> Result<PathBuf> {
    write(root, "src/Child.vue", CHILD)?;
    write(root, "src/Parent.vue", PARENT)?;
    write(root, "tsconfig.json", config)?;
    link_vue(root)
}

pub fn cases() -> Result<Vec<(&'static str, Vec<u8>, Vec<usize>)>> {
    let original: Value = serde_json::from_str(CONFIG)?;
    let mut result = vec![("original", CONFIG.as_bytes().to_vec(), vec![0, 1, 2])];
    for (name, options, expected) in [
        (
            "props-false",
            json!({"checkUnknownComponents":true,"checkUnknownProps":false,"checkUnknownDirectives":true}),
            vec![1, 2],
        ),
        (
            "directives-false",
            json!({"checkUnknownComponents":true,"checkUnknownProps":true,"checkUnknownDirectives":false}),
            vec![0, 1],
        ),
        (
            "all-false",
            json!({"checkUnknownComponents":false,"checkUnknownProps":false,"checkUnknownDirectives":false}),
            vec![],
        ),
        ("absent", json!({}), vec![]),
        (
            "strict-defaults",
            json!({"strictTemplates":true}),
            vec![0, 1, 2],
        ),
        (
            "strict-explicit-false",
            json!({"strictTemplates":true,"checkUnknownComponents":false,"checkUnknownProps":false,"checkUnknownDirectives":false}),
            vec![],
        ),
    ] {
        let mut config = original.clone();
        let object = config
            .as_object_mut()
            .ok_or("original config is not object")?;
        if name == "absent" {
            object.remove("vueCompilerOptions");
        } else {
            object.insert("vueCompilerOptions".into(), options);
        }
        result.push((name, serde_json::to_vec(&config)?, expected));
    }
    Ok(result)
}

fn digest(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(Sha256::digest(std::fs::read(path)?)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
pub fn retain(
    root: &Path,
    name: &str,
    corsa: &Path,
    command: &Command,
    output: &std::process::Output,
) -> Result<()> {
    let Some(target) = std::env::var_os("VIZE_UNKNOWN_TEMPLATE_CAPTURE") else {
        return Ok(());
    };
    let target = PathBuf::from(target).join(name);
    write(&target, "stdout.txt", &output.stdout)?;
    write(&target, "stderr.txt", &output.stderr)?;
    for file in [
        "src/Child.vue",
        "src/Parent.vue",
        "tsconfig.json",
        "Oracle.ts",
    ] {
        if root.join(file).is_file() {
            write(&target, file, std::fs::read(root.join(file))?)?;
        }
    }
    let version = Command::new(corsa).arg("--version").output()?;
    write(&target, "native-version.stdout.txt", &version.stdout)?;
    write(&target, "native-version.stderr.txt", &version.stderr)?;
    require(
        version.status.success()
            && version.stdout == b"Version 7.0.2\n"
            && version.stderr.is_empty(),
        "native version differs",
    )?;
    write(
        &target,
        "runtime.json",
        serde_json::to_vec(&json!({
            "sourceSha":std::env::var("SOURCE_SHA").ok(),"nativeBinary":corsa,"nativeSha256":digest(corsa)?,
            "executable":command.get_program().to_string_lossy(),"executableSha256":digest(Path::new(command.get_program()))?,"exitCode":output.status.code(),
            "workingDirectory":command.get_current_dir(),"arguments":command.get_args().map(|arg|arg.to_string_lossy()).collect::<Vec<_>>(),
            "inputHashes":{"parent":digest(&root.join(if name == "oracle" {"Oracle.ts"} else {"src/Parent.vue"}))?,"config":digest(&root.join("tsconfig.json"))?}
        }))?,
    )?;
    Ok(())
}

/// The oracle is authored independently; its complete native output supplies the
/// compiler-version-specific type display, never a captured Vize expectation.
pub fn messages(corsa: &Path) -> Result<Vec<String>> {
    let directory = tempfile::tempdir()?;
    let root = directory.path().canonicalize()?;
    link_vue(&root)?;
    write(&root, "Oracle.ts", ORACLE)?;
    let original: Value = serde_json::from_str(CONFIG)?;
    write(
        &root,
        "tsconfig.json",
        serde_json::to_vec(
            &json!({"compilerOptions":original.get("compilerOptions"),"files":["Oracle.ts"]}),
        )?,
    )?;
    let mut command = Command::new(corsa);
    command
        .current_dir(&root)
        .args(["-p", "tsconfig.json", "--pretty", "false"]);
    let output = command.output()?;
    retain(&root, "oracle", corsa, &command, &output)?;
    require(
        output.status.code() == Some(1) && output.stderr.is_empty(),
        "native oracle status differs",
    )?;
    let stdout = std::str::from_utf8(&output.stdout)?;
    let mut messages = Vec::new();
    for (line, (position, code)) in
        stdout
            .lines()
            .zip([("12,24", 2353), ("13,9", 2339), ("14,9", 2339)])
    {
        let prefix = format!("Oracle.ts({position}): error TS{code}: ");
        messages.push(
            line.strip_prefix(&prefix)
                .ok_or("whole native vector has an unexpected row")?
                .to_owned(),
        );
    }
    require(
        stdout.lines().count() == 3 && messages.len() == 3,
        "native oracle must return exactly three complete rows",
    )?;
    require(
        messages.get(1).map(String::as_str)
            == Some("Property 'NotImported' does not exist on type '{}'."),
        "component message changed",
    )?;
    require(
        messages.get(2).map(String::as_str)
            == Some("Property 'vNotADirective' does not exist on type '{}'."),
        "directive message changed",
    )?;
    Ok(messages)
}
