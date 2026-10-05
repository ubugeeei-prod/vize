//! Original #7949 workspace, with genuine app-scoped Vue declarations.
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

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

pub fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

pub fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

pub fn link(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

pub fn fixture(root: &Path, root_vue: bool) -> PathBuf {
    for (name, source) in FILES {
        write(root, name, source);
    }
    // The frozen workspace already owns this exact Vue alias. Resolve its real
    // pnpm package so its @vue dependencies remain the genuine installed set.
    let vue = workspace()
        .join("tests/node_modules/vue-ssr-css-vars-oracle")
        .canonicalize()
        .unwrap();
    let package: Value =
        serde_json::from_slice(&std::fs::read(vue.join("package.json")).unwrap()).unwrap();
    assert_eq!(package["name"], "vue");
    assert_eq!(package["version"], "3.6.0-rc.10");
    link(&vue, &root.join("apps/web/node_modules/vue"));
    if root_vue {
        write(
            root,
            "package.json",
            "{ \"name\": \"root\", \"private\": true, \"dependencies\": { \"vue\": \"3.6.0-rc.10\" } }\n",
        );
        link(&vue, &root.join("node_modules/vue"));
    } else {
        assert!(!root.join("node_modules/vue").exists());
    }
    vue
}

pub fn capture(root: &Path, vue: &Path, corsa: &Path, case: &str) -> Option<PathBuf> {
    let target = PathBuf::from(std::env::var_os("VIZE_VUE_HELPER_CAPTURE")?).join(case);
    std::fs::create_dir_all(&target).unwrap();
    for (name, _) in FILES {
        let output = target.join("inputs").join(name);
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        std::fs::copy(root.join(name), output).unwrap();
    }
    let direct = |name| vue.parent().unwrap().join(name).canonicalize().unwrap();
    let runtime_dom = direct("@vue/runtime-dom");
    let runtime_core = runtime_dom
        .parent()
        .unwrap()
        .join("runtime-core")
        .canonicalize()
        .unwrap();
    let compiler_dom = direct("@vue/compiler-dom");
    for (package, directory) in [
        ("vue", vue.to_path_buf()),
        ("@vue/runtime-dom", runtime_dom),
        (
            "@vue/reactivity",
            runtime_core
                .parent()
                .unwrap()
                .join("reactivity")
                .canonicalize()
                .unwrap(),
        ),
        ("@vue/runtime-core", runtime_core),
        ("@vue/shared", direct("@vue/shared")),
        (
            "@vue/compiler-core",
            compiler_dom
                .parent()
                .unwrap()
                .join("compiler-core")
                .canonicalize()
                .unwrap(),
        ),
        ("@vue/compiler-dom", compiler_dom),
        ("@vue/compiler-sfc", direct("@vue/compiler-sfc")),
        ("@vue/runtime-vapor", direct("@vue/runtime-vapor")),
    ] {
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(directory.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["name"], package);
        assert_eq!(manifest["version"], "3.6.0-rc.10");
        let types = manifest["types"].as_str().unwrap();
        for name in ["package.json", types] {
            let output = target.join("packages").join(package).join(name);
            std::fs::create_dir_all(output.parent().unwrap()).unwrap();
            std::fs::copy(directory.join(name), output).unwrap();
        }
    }
    let configuration = root.join("apps/web/vize.config.json");
    if configuration.is_file() {
        std::fs::copy(configuration, target.join("vize.config.json")).unwrap();
    }
    std::fs::write(
        target.join("runtime.json"),
        json!({"nativeBinary":corsa,"vuePackage":vue,"sourceSha":std::env::var("SOURCE_SHA").ok()})
            .to_string(),
    )
    .unwrap();
    Some(target)
}
