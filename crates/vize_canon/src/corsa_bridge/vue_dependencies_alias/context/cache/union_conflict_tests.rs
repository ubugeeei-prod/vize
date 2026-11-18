//! An open host whose workspace alias leaf conflicts with another member's.
//!
//! primevue-volt: the Button.vue member placed
//! `packages/primevue/node_modules/@primevue/icons` at
//! `src/button/node_modules/@primevue/icons`, and FilesCard.vue placed it at
//! `src/node_modules/@primevue/icons`. Both are owned targets, so the union
//! refused to materialize and the language server respawned tsgo cold.
#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::super::{AliasContext, PreparedAliasContext};
use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_document::CorsaProjectEnvironment;

const HOST: &str =
    "<script setup lang=\"ts\">import { Btn } from \"@x/b\";</script><template><Btn /></template>";

fn workspace() -> (tempfile::TempDir, PathBuf) {
    fn copy_tree(source: &Path, target: &Path) {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let target = target.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/pnpm-workspace-routes");
    copy_tree(&corpus, &root);
    let links: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(corpus.join("links.json")).unwrap()).unwrap();
    for group in ["root", "pnpm"] {
        for (path, target) in links[group].as_object().unwrap() {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(target.as_str().unwrap(), path).unwrap();
        }
    }
    (temp, root)
}

fn open(
    session: &EditorMirrorSession,
    path: &Path,
    overlays: &FxHashMap<PathBuf, &str>,
) -> Result<PreparedAliasContext, crate::corsa_bridge::CorsaBridgeError> {
    AliasContext::for_hosts_cached(
        path,
        HOST,
        overlays,
        &[],
        Default::default(),
        CorsaProjectEnvironment {
            virtual_ts_options: &Default::default(),
            package_routes: &Default::default(),
            project_root: None,
            tsconfig_path: None,
            editor_session: session,
        },
    )
}

#[test]
fn a_conflicting_union_alias_keeps_the_new_host_alone_instead_of_failing() {
    let (_temp, root) = workspace();
    let hosts = ["HostA.vue", "HostC.vue"].map(|name| root.join("packages/a/src").join(name));
    for host in &hosts {
        fs::write(host, HOST).unwrap();
    }
    let overlays: FxHashMap<_, _> = hosts.iter().map(|host| (host.clone(), HOST)).collect();
    let session = EditorMirrorSession::new();
    let first = open(&session, &hosts[0], &overlays).unwrap();
    let mirror_root = first.mirror.as_ref().unwrap().virtual_root().to_path_buf();
    drop(first);

    // The package-root leaf points at the owned alias beside its importers.
    // Another member can place that alias in a sibling directory instead.
    let leaf = mirror_root.join("packages/b/node_modules/@x/c");
    let own = mirror_root.join("packages/b/src/node_modules/@x/c");
    let sibling = mirror_root.join("packages/b/src/button/node_modules/@x/c");
    {
        let mut cache = session.cache();
        let members = cache.project_members.get_mut(&mirror_root).unwrap();
        assert_eq!(members.len(), 1);
        let member = members.values_mut().next().unwrap();
        assert_eq!(member.package_links.get(&leaf), Some(&own));
        // That member owns and materialized its copy of the alias.
        let mut owned = member
            .expected_files
            .iter()
            .filter(|file| file.starts_with(&own))
            .cloned()
            .collect::<Vec<_>>();
        owned.sort();
        assert_eq!(
            owned,
            [
                "package.json",
                "src/Btn.d.vue.ts",
                "src/Btn.vue.ts",
                "src/index.ts",
                "src/util.ts"
            ]
            .map(|file| own.join(file))
        );
        for file in owned {
            let copy = sibling.join(file.strip_prefix(&own).unwrap());
            fs::create_dir_all(copy.parent().unwrap()).unwrap();
            fs::copy(&file, &copy).unwrap();
            member.expected_files.insert(copy);
        }
        member.package_links.insert(leaf.clone(), sibling.clone());
    }

    let second = open(&session, &hosts[1], &overlays)
        .expect("a conflicting member must not fail the new host's union");
    {
        let cache = session.cache();
        let members = &cache.project_members[&mirror_root];
        assert_eq!(
            members
                .keys()
                .map(|key| key.source_path.as_path())
                .collect::<Vec<_>>(),
            [hosts[1].as_path()],
            "only the new host remains in the union"
        );
        assert_eq!(
            cache
                .slots
                .keys()
                .map(|key| key.source_path.as_path())
                .collect::<Vec<_>>(),
            [hosts[1].as_path()],
            "the retired host must not reuse a context whose files left the mirror"
        );
    }
    assert_eq!(
        fs::canonicalize(&leaf).unwrap(),
        fs::canonicalize(&own).unwrap(),
        "the leaf keeps the new host's own target"
    );
    drop(second);

    // The retired host rebuilds against the current mirror on its next request.
    let rebuilt = open(&session, &hosts[0], &overlays).unwrap();
    assert_eq!(
        rebuilt.mirror.as_ref().unwrap().virtual_root(),
        mirror_root.as_path()
    );
}
