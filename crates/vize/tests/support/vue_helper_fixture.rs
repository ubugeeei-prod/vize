//! Original #7949 workspace, with genuine app-scoped Vue declarations.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

macro_rules! original {
    ($name:literal) => {
        include_str!(concat!(
            "../../../../tests/_fixtures/differential/typechecker/config-scoped-vue-helpers/",
            $name,
            ".txt"
        ))
    };
}

pub const APP: &str = original!("apps/web/App.vue");
pub const COUNTER: &str = original!("apps/web/Counter.vue");
pub const TOGGLE: &str = original!("apps/web/Toggle.vue");
pub const FILES: &[(&str, &str)] = &[
    ("package.json", original!("package.json")),
    ("pnpm-workspace.yaml", original!("pnpm-workspace.yaml")),
    (
        "shared/router/index.ts",
        original!("shared/router/index.ts"),
    ),
    (
        "shared/router/Link.vue",
        original!("shared/router/Link.vue"),
    ),
    ("apps/web/package.json", original!("apps/web/package.json")),
    (
        "apps/web/tsconfig.json",
        original!("apps/web/tsconfig.json"),
    ),
    ("apps/web/App.vue", APP),
    ("apps/web/Counter.vue", COUNTER),
    ("apps/web/Toggle.vue", TOGGLE),
];

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn require(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(std::io::Error::other(message).into())
    }
}

fn parent(path: &Path) -> Result<&Path> {
    path.parent()
        .ok_or_else(|| std::io::Error::other("fixture path has no parent").into())
}

fn field<'a>(manifest: &'a Value, name: &str) -> Result<&'a str> {
    manifest
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("fixture manifest lacks a string field").into())
}

pub fn workspace() -> Result<&'static Path> {
    parent(parent(Path::new(env!("CARGO_MANIFEST_DIR")))?)
}

pub fn write(root: &Path, name: &str, source: &str) -> Result<()> {
    let path = root.join(name);
    std::fs::create_dir_all(parent(&path)?)?;
    std::fs::write(path, source)?;
    Ok(())
}

pub fn link(source: &Path, target: &Path) -> Result<()> {
    std::fs::create_dir_all(parent(target)?)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target)?;
    Ok(())
}

pub fn fixture(root: &Path, root_vue: bool) -> Result<PathBuf> {
    for (name, source) in FILES {
        write(root, name, source)?;
    }
    // The frozen workspace already owns this exact Vue alias. Resolve its real
    // pnpm package so its @vue dependencies remain the genuine installed set.
    let vue = workspace()?
        .join("tests/node_modules/vue-ssr-css-vars-oracle")
        .canonicalize()?;
    let package: Value = serde_json::from_slice(&std::fs::read(vue.join("package.json"))?)?;
    require(field(&package, "name")? == "vue", "fixture is not Vue")?;
    require(
        field(&package, "version")? == "3.6.0-rc.10",
        "fixture Vue version differs",
    )?;
    link(&vue, &root.join("apps/web/node_modules/vue"))?;
    if root_vue {
        write(
            root,
            "package.json",
            "{ \"name\": \"root\", \"private\": true, \"dependencies\": { \"vue\": \"3.6.0-rc.10\" } }\n",
        )?;
        link(&vue, &root.join("node_modules/vue"))?;
    } else {
        require(
            !root.join("node_modules/vue").exists(),
            "root unexpectedly contains Vue",
        )?;
    }
    Ok(vue)
}

pub fn capture(root: &Path, vue: &Path, corsa: &Path, case: &str) -> Result<Option<PathBuf>> {
    let Some(capture) = std::env::var_os("VIZE_VUE_HELPER_CAPTURE") else {
        return Ok(None);
    };
    let target = PathBuf::from(capture).join(case);
    std::fs::create_dir_all(&target)?;
    for (name, _) in FILES {
        let output = target.join("inputs").join(name);
        std::fs::create_dir_all(parent(&output)?)?;
        std::fs::copy(root.join(name), output)?;
    }
    let direct = |name| -> Result<PathBuf> { Ok(parent(vue)?.join(name).canonicalize()?) };
    let runtime_dom = direct("@vue/runtime-dom")?;
    let runtime_core = parent(&runtime_dom)?.join("runtime-core").canonicalize()?;
    let compiler_dom = direct("@vue/compiler-dom")?;
    for (package, directory) in [
        ("vue", vue.to_path_buf()),
        ("@vue/runtime-dom", runtime_dom),
        (
            "@vue/reactivity",
            parent(&runtime_core)?.join("reactivity").canonicalize()?,
        ),
        ("@vue/runtime-core", runtime_core),
        ("@vue/shared", direct("@vue/shared")?),
        (
            "@vue/compiler-core",
            parent(&compiler_dom)?
                .join("compiler-core")
                .canonicalize()?,
        ),
        ("@vue/compiler-dom", compiler_dom),
        ("@vue/compiler-sfc", direct("@vue/compiler-sfc")?),
        ("@vue/runtime-vapor", direct("@vue/runtime-vapor")?),
    ] {
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(directory.join("package.json"))?)?;
        require(field(&manifest, "name")? == package, "package name differs")?;
        require(
            field(&manifest, "version")? == "3.6.0-rc.10",
            "package Vue version differs",
        )?;
        let types = field(&manifest, "types")?;
        for name in ["package.json", types] {
            let output = target.join("packages").join(package).join(name);
            std::fs::create_dir_all(parent(&output)?)?;
            std::fs::copy(directory.join(name), output)?;
        }
    }
    let configuration = root.join("apps/web/vize.config.json");
    if configuration.is_file() {
        std::fs::copy(configuration, target.join("vize.config.json"))?;
    }
    std::fs::write(
        target.join("runtime.json"),
        serde_json::to_vec(
            &json!({"nativeBinary":corsa,"vuePackage":vue,"sourceSha":std::env::var("SOURCE_SHA").ok()}),
        )?,
    )?;
    Ok(Some(target))
}
