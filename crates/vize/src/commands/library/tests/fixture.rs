//! Fixture registries and projects for `vize lib` tests (no network).

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde_json::{Value, json};
use vize_s0::String;

use super::super::error::LibResult;
use super::super::fs_ops::{content_hash, sha256_hex};
use super::super::{LibArgs, execute};

/// One fixture item: name, files, registry dependencies, npm dependencies.
pub struct Item<'a> {
    pub name: &'a str,
    pub files: &'a [(&'a str, &'a str)],
    pub deps: &'a [&'a str],
    pub aliases: &'a [&'a str],
}

impl<'a> Item<'a> {
    pub const fn new(name: &'a str, files: &'a [(&'a str, &'a str)], deps: &'a [&'a str]) -> Self {
        Self {
            name,
            files,
            deps,
            aliases: &[],
        }
    }
}

/// Write `registry.json` + `files/**` for one kind and version into `dir`.
pub fn write_registry(dir: &Path, kind: &str, version: &str, items: &[Item<'_>]) {
    let package = if kind == "ui" {
        "@vizejs/ui"
    } else {
        "@vizejs/composable"
    };
    let default_dir = if kind == "ui" {
        "src/components/vize"
    } else {
        "src/composables/vize"
    };
    let _ = fs::remove_dir_all(dir);
    let mut manifest_items: Vec<Value> = Vec::new();
    for item in items {
        let mut files: Vec<Value> = Vec::new();
        let mut hashes: Vec<(String, String)> = Vec::new();
        for (path, content) in item.files {
            let target = dir.join("files").join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, content).unwrap();
            let sha = sha256_hex(content.as_bytes());
            hashes.push((String::from(*path), sha.clone()));
            files.push(json!({
                "path": path,
                "role": if path.ends_with(".vue") { "component" } else { "module" },
                "sha256": sha.as_str(),
                "size": content.len(),
            }));
        }
        let hash = content_hash(
            hashes
                .iter()
                .map(|(path, sha)| (path.as_str(), sha.as_str())),
        );
        manifest_items.push(json!({
            "name": item.name,
            "kind": kind,
            "title": item.name,
            "description": vize_s0::cstr!("{} fixture item", item.name).as_str(),
            "aliases": item.aliases,
            "packageSubpath": vize_s0::cstr!("./{}", item.name).as_str(),
            "entry": item.files.first().map(|(path, _)| *path).unwrap_or_default(),
            "files": files,
            "registryDependencies": item.deps,
            "dependencies": [{ "name": "vue", "range": "^3.5.0", "kind": "peer" }],
            "contentHash": hash.as_str(),
        }));
    }
    let manifest = json!({
        "schemaVersion": 1,
        "registryKind": "vize-lib",
        "package": { "name": package, "version": version },
        "kind": kind,
        "filesDirectory": "files",
        "defaultTargetDirectory": default_dir,
        "items": manifest_items,
    });
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join("registry.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

pub const ID_V1: &str = "export const createId = (prefix: string) => `${prefix}-1`;\n";
pub const STATE_V1: &str =
    "import { createId } from \"../id/id.ts\";\nexport const state = createId(\"s\");\n";
pub const RATING_TS_V1: &str =
    "export { default as Rating } from \"./rating.vue\";\nexport const size = 5;\n";
pub const RATING_VUE_V1: &str = "<script setup lang=\"ts\">\nimport { state } from \"../../foundations/state/state.ts\";\n</script>\n<template><div data-vize-ui=\"rating\" /></template>\n";

/// ui@1.0.0: rating -> state -> id.
pub fn ui_v1(dir: &Path) {
    write_registry(
        dir,
        "ui",
        "1.0.0",
        &[
            Item::new("id", &[("foundations/id/id.ts", ID_V1)], &[]),
            Item {
                aliases: &["star rating"],
                ..Item::new(
                    "rating",
                    &[
                        ("families/rating/rating.ts", RATING_TS_V1),
                        ("families/rating/rating.vue", RATING_VUE_V1),
                    ],
                    &["id", "state"],
                )
            },
            Item::new(
                "locale",
                &[("foundations/locale/locale.ts", "export {};\n")],
                &[],
            ),
            Item::new(
                "state",
                &[("foundations/state/state.ts", STATE_V1)],
                &["id"],
            ),
        ],
    );
}

pub const RATING_TS_V2: &str =
    "export { default as Rating } from \"./rating.vue\";\nexport const size = 10;\n";
pub const RATING_KEYS_V2: &str = "export const keys = [\"ArrowLeft\", \"ArrowRight\"];\n";

/// ui@2.0.0: rating.ts changed, rating.vue unchanged, rating-keys.ts added,
/// and rating now also needs `locale`.
pub fn ui_v2(dir: &Path) {
    write_registry(
        dir,
        "ui",
        "2.0.0",
        &[
            Item::new("id", &[("foundations/id/id.ts", ID_V1)], &[]),
            Item::new(
                "rating",
                &[
                    ("families/rating/rating-keys.ts", RATING_KEYS_V2),
                    ("families/rating/rating.ts", RATING_TS_V2),
                    ("families/rating/rating.vue", RATING_VUE_V1),
                ],
                &["id", "locale", "state"],
            ),
            Item::new(
                "locale",
                &[("foundations/locale/locale.ts", "export {};\n")],
                &[],
            ),
            Item::new(
                "state",
                &[("foundations/state/state.ts", STATE_V1)],
                &["id"],
            ),
        ],
    );
}

/// composable@1.0.0 with a `locale` item that collides with ui's.
pub fn composable_v1(dir: &Path) {
    write_registry(
        dir,
        "composable",
        "1.0.0",
        &[
            Item::new(
                "locale",
                &[("locale.ts", "export const useLocale = () => \"en\";\n")],
                &[],
            ),
            Item {
                aliases: &["useToggle"],
                ..Item::new(
                    "use-toggle",
                    &[("use-toggle.ts", "export const useToggle = () => true;\n")],
                    &[],
                )
            },
        ],
    );
}

/// A throwaway project plus registry directories.
pub struct Project {
    _temp: tempfile::TempDir,
    pub root: PathBuf,
    pub registries: PathBuf,
}

impl Project {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("app");
        let registries = temp.path().join("registries");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("package.json"),
            r#"{ "dependencies": { "vue": "^3.5.0" } }"#,
        )
        .unwrap();
        Self {
            root: root.canonicalize().unwrap(),
            registries,
            _temp: temp,
        }
    }

    pub fn registry(&self, name: &str) -> PathBuf {
        self.registries.join(name)
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn read(&self, relative: &str) -> String {
        String::from(fs::read_to_string(self.path(relative)).unwrap())
    }

    pub fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// Run `vize lib --root <root> <args...>`.
    pub fn run(&self, args: &[&str]) -> LibResult<String> {
        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            args: LibArgs,
        }
        let root = self.root.to_str().unwrap();
        let argv = ["vize-lib", "--root", root]
            .into_iter()
            .chain(args.iter().copied());
        execute(&TestCli::try_parse_from(argv).unwrap().args)
    }

    /// Run with `--registry <registries/name>` for each name.
    pub fn run_with(&self, registries: &[&str], args: &[&str]) -> LibResult<String> {
        let paths: Vec<String> = registries
            .iter()
            .map(|name| String::from(self.registry(name).to_str().unwrap()))
            .collect();
        let mut argv: Vec<&str> = Vec::new();
        for path in &paths {
            argv.extend(["--registry", path.as_str()]);
        }
        argv.extend(args);
        self.run(&argv)
    }

    pub fn json(&self, registries: &[&str], args: &[&str]) -> Value {
        let mut argv = vec!["--json"];
        argv.extend(args);
        serde_json::from_str(&self.run_with(registries, &argv).unwrap()).unwrap()
    }
}
