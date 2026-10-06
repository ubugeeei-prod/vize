//! The complete #7990 authored workspace, with installed providers only.
#![expect(
    clippy::disallowed_types,
    reason = "fixtures retain complete std byte maps"
)]

use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/symlink-workspace-staging-original"
);

#[derive(Clone, Copy)]
pub enum Layout {
    Original,
    BroadInclude,
    SrcInclude,
    AuthoredCompanions,
    CrLf,
}

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Directory,
    File(Vec<u8>),
    Symlink(PathBuf),
}

pub struct Fixture {
    _directory: tempfile::TempDir,
    pub root: PathBuf,
    pub app: PathBuf,
    pub host: PathBuf,
    pub other: PathBuf,
    pub host_source: String,
    pub other_source: String,
    before: BTreeMap<PathBuf, Entry>,
}

impl Fixture {
    pub fn new(layout: Layout) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let runtime = discover_corsa_in_ancestors(workspace)
            .expect("#7990 requires the installed native TypeScript runtime");
        let native_package = runtime
            .ancestors()
            .find(|path| {
                fs::read(path.join("package.json"))
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .is_some_and(|manifest| {
                        manifest["name"]
                            .as_str()
                            .is_some_and(|name| name.starts_with("@typescript/typescript-"))
                    })
            })
            .expect("the native executable must belong to an actual platform package");
        let native_manifest: Value =
            serde_json::from_slice(&fs::read(native_package.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(native_manifest["version"], "7.0.2");
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("#7990 requires the installed Playground Vue dependency");
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let app = root.join("app");
        for (original, target) in [
            ("package.json", "package.json"),
            ("ui/package.json", "ui/package.json"),
            ("ui/UiButton.vue.txt", "ui/UiButton.vue"),
            ("app/tsconfig.json", "app/tsconfig.json"),
            ("app/App.vue.txt", "app/App.vue"),
            ("app/Other.vue.txt", "app/Other.vue"),
        ] {
            let destination = root.join(target);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::write(
                destination,
                fs::read(Path::new(CORPUS).join(original)).unwrap(),
            )
            .unwrap();
        }
        let source_dir = if matches!(layout, Layout::SrcInclude) {
            app.join("src")
        } else {
            app.clone()
        };
        if source_dir != app {
            fs::create_dir_all(&source_dir).unwrap();
            for name in ["App.vue", "Other.vue"] {
                fs::rename(app.join(name), source_dir.join(name)).unwrap();
            }
        }
        if matches!(layout, Layout::BroadInclude | Layout::SrcInclude) {
            let config_path = app.join("tsconfig.json");
            let mut config: Value =
                serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
            config["include"] = json!([if matches!(layout, Layout::SrcInclude) {
                "src/**/*.vue"
            } else {
                "**/*.vue"
            }]);
            fs::write(config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        }
        if matches!(layout, Layout::CrLf) {
            for path in [
                source_dir.join("App.vue"),
                source_dir.join("Other.vue"),
                root.join("ui/UiButton.vue"),
            ] {
                let source = fs::read_to_string(&path).unwrap().replace('\n', "\r\n");
                fs::write(path, source).unwrap();
            }
        }
        if matches!(layout, Layout::AuthoredCompanions) {
            fs::write(
                root.join("ui/UiButton.vue.ts"),
                "export const authored = 1;\n",
            )
            .unwrap();
            fs::write(
                root.join("ui/UiButton.d.vue.ts"),
                "export default class Authored {}\n",
            )
            .unwrap();
        }
        fs::create_dir_all(app.join("node_modules")).unwrap();
        std::os::unix::fs::symlink("../../ui", app.join("node_modules/ui")).unwrap();
        let providers = root.join("node_modules");
        fs::create_dir_all(providers.join("@typescript")).unwrap();
        std::os::unix::fs::symlink(
            native_package,
            providers.join(native_manifest["name"].as_str().unwrap()),
        )
        .unwrap();
        std::os::unix::fs::symlink(vue, providers.join("vue")).unwrap();
        assert_eq!(
            discover_corsa_in_ancestors(&app)
                .unwrap()
                .canonicalize()
                .unwrap(),
            runtime.canonicalize().unwrap()
        );
        let host = source_dir.join("App.vue");
        let other = source_dir.join("Other.vue");
        let host_source = fs::read_to_string(&host).unwrap();
        let other_source = fs::read_to_string(&other).unwrap();
        let before = snapshot(&root);
        Self {
            _directory: directory,
            root,
            app,
            host,
            other,
            host_source,
            other_source,
            before,
        }
    }

    pub fn assert_authored_unchanged(&self) {
        let mut observed = snapshot(&self.root);
        // Default stdio initializes exactly this cache directory and append log
        // in vize_maestro::serve. Keep every other complete entry comparison.
        let log_dir = PathBuf::from("app/node_modules/.vize");
        assert!(!self.before.contains_key(&log_dir));
        assert_eq!(observed.remove(&log_dir), Some(Entry::Directory));
        let Some(Entry::File(bytes)) = observed.remove(&log_dir.join("lsp.log")) else {
            panic!("default stdio must retain its regular known logger");
        };
        let log = std::str::from_utf8(&bytes).unwrap();
        let (_, record) = log.lines().next().unwrap().split_once(' ').unwrap();
        assert_eq!(
            record.trim_start(),
            "INFO vize_maestro: Starting vize_maestro LSP server"
        );
        assert_eq!(
            observed, self.before,
            "every authored path, symlink and full file byte must survive generation and cleanup"
        );
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn walk(root: &Path, directory: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            let kind = entry.file_type().unwrap();
            let content = if kind.is_symlink() {
                Entry::Symlink(fs::read_link(&path).unwrap())
            } else if kind.is_dir() {
                walk(root, &path, entries);
                Entry::Directory
            } else {
                Entry::File(fs::read(path).unwrap())
            };
            entries.insert(relative, content);
        }
    }
    let mut entries = BTreeMap::new();
    walk(root, root, &mut entries);
    entries
}
