//! Isolated tracked source, preserving actual nearest-first provider scopes.

use super::super::{CONFIG_PATH, PIN, ROOT_PATH, STORY_PATH, TYPE_PATH, source};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_carton::cstr;

pub(super) fn snapshot(temporary: &Path) -> (PathBuf, Value) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let relative = Path::new("tests/_fixtures/_git/reka-ui");
    let original = repo.join(relative);
    let head = Command::new("git")
        .arg("-C")
        .arg(&original)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(
        head.status.success(),
        "the required pinned original fixture must be hydrated"
    );
    assert_eq!(std::str::from_utf8(&head.stdout).unwrap().trim(), PIN);
    let workspace = temporary.join("workspace");
    let root = workspace.join(relative);
    fs::create_dir_all(&root).unwrap();
    let archive = temporary.join("tracked.tar");
    let archived = Command::new("git")
        .arg("-C")
        .arg(&original)
        .args(["archive", "--format=tar", "--output"])
        .arg(&archive)
        .arg(PIN)
        .output()
        .unwrap();
    assert!(archived.status.success(), "{archived:?}");
    let extracted = Command::new("tar")
        .arg("-xf")
        .arg(&archive)
        .arg("-C")
        .arg(&root)
        .output()
        .unwrap();
    assert!(extracted.status.success(), "{extracted:?}");
    let tree = Command::new("git")
        .arg("-C")
        .arg(&original)
        .args(["ls-tree", "-r", "--full-tree", PIN])
        .output()
        .unwrap();
    assert!(tree.status.success(), "{tree:?}");
    for (path, corpus) in [
        (ROOT_PATH, "PinInputRoot.vue.txt"),
        (STORY_PATH, "PinInput.story.vue.txt"),
        (TYPE_PATH, "types.ts.txt"),
    ] {
        assert_eq!(
            fs::read_to_string(original.join(path)).unwrap(),
            source(corpus).as_str()
        );
        assert_eq!(
            fs::read_to_string(root.join(path)).unwrap(),
            source(corpus).as_str()
        );
    }
    assert_eq!(
        fs::read(original.join(CONFIG_PATH)).unwrap(),
        fs::read(root.join(CONFIG_PATH)).unwrap()
    );
    let mut providers = Vec::new();
    for ancestor in original
        .ancestors()
        .take_while(|ancestor| ancestor.starts_with(repo))
    {
        let provider = ancestor.join("node_modules");
        let target = workspace
            .join(ancestor.strip_prefix(repo).unwrap())
            .join("node_modules");
        providers.push(link(&provider, &target));
    }
    for scope in ["packages/core/node_modules", ".histoire/node_modules"] {
        providers.push(link(&original.join(scope), &root.join(scope)));
    }
    assert!(
        providers.iter().any(|entry| entry["present"] == true),
        "actual installed provider scopes are required"
    );
    let root = root.canonicalize().unwrap();
    let custody = json!({
        "revision":PIN,"wholeTrackedTree":std::str::from_utf8(&tree.stdout).unwrap(),
        "originalFixtureRoot":original,"isolatedSnapshotRoot":root,
        "wholeOriginalRoot":source("PinInputRoot.vue.txt"),"wholeOriginalStory":source("PinInput.story.vue.txt"),
        "wholeOriginalSharedTypes":source("types.ts.txt"),
        "declaredSourceConfig":CONFIG_PATH,"wholeDeclaredSourceConfig":fs::read_to_string(root.join(CONFIG_PATH)).unwrap(),
        "observedProviderScopesNearestFirst":providers,
        "originalCliConfigScope":"the original full App check has no explicit tsconfig; this control explicitly selects its existing core config and does not claim the two identities are equal",
    });
    (root, custody)
}

fn link(original: &Path, isolated: &Path) -> Value {
    if !original.is_dir() {
        return json!({"originalPath":original,"isolatedPath":isolated,"present":false});
    }
    let canonical = original.canonicalize().unwrap();
    fs::create_dir_all(isolated.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&canonical, isolated).unwrap();
    let vue = original.join("vue");
    let config = original.join("@vue/tsconfig");
    json!({
        "originalPath":original,"isolatedPath":isolated,"present":true,
        "originalReadlink":fs::read_link(original).ok(),"canonicalTarget":canonical,
        "isolatedReadlink":fs::read_link(isolated).unwrap(),
        "vueCanonicalTarget":vue.canonicalize().ok(),"wholeVueManifest":fs::read_to_string(vue.join("package.json")).ok(),
        "configCanonicalTarget":config.canonicalize().ok(),
        "wholeConfigProviderManifest":fs::read_to_string(config.join("package.json")).ok(),
        "wholeConfigProviderDom":fs::read_to_string(config.join("tsconfig.dom.json")).ok(),
        "wholeConfigProviderBase":fs::read_to_string(config.join("tsconfig.json")).ok(),
        "scope":cstr!("actual original provider scope, no install, dereference copy or authored fixture mutation"),
    })
}
