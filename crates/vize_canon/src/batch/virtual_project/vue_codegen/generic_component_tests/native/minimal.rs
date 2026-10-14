//! Tracked whole-SFC cause control needs no upstream Git fixture hydration.

use super::super::{CONFIG_PATH, GENERIC, ROOT_PATH, STORY_PATH, source};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_carton::{String, cstr};

pub(super) fn root_source(generic: &str) -> String {
    source("minimal-root.vue.txt")
        .replace(
            cstr!("generic=\"{GENERIC}\"").as_str(),
            cstr!("generic=\"{generic}\"").as_str(),
        )
        .into()
}

pub(super) fn install(root: &Path) -> (PathBuf, Value) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    for path in [ROOT_PATH, STORY_PATH, CONFIG_PATH] {
        fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
    }
    fs::write(
        root.join(CONFIG_PATH),
        source("minimal-tsconfig.json.txt").as_str(),
    )
    .unwrap();
    fs::write(
        root.join("packages/core/src/PinInput/index.ts"),
        source("minimal-index.ts.txt").as_str(),
    )
    .unwrap();
    let provider = repo.join("tests/node_modules").canonicalize().unwrap();
    assert!(
        provider.join("vue/package.json").is_file(),
        "standard installed Vue provider is required"
    );
    std::os::unix::fs::symlink(&provider, root.join("node_modules")).unwrap();
    let root = root.canonicalize().unwrap();
    let packet = json!({
        "scope":"tracked minimal whole-SFC native cause control; no ReKa Git checkout or install",
        "wholeRoot":source("minimal-root.vue.txt"),"wholeStory":source("minimal-story.vue.txt"),
        "wholeBarrel":source("minimal-index.ts.txt"),
        "wholeConfig":source("minimal-tsconfig.json.txt"),"declaredSourceConfig":root.join(CONFIG_PATH),
        "providerScope":provider,"wholeVueManifest":fs::read_to_string(provider.join("vue/package.json")).unwrap(),
        "vueCanonicalTarget":provider.join("vue").canonicalize().unwrap(),
        "isolatedProviderLink":fs::read_link(root.join("node_modules")).unwrap(),
    });
    (root, packet)
}
