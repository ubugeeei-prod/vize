use std::fs;
use std::path::Path;

use super::TypeSourceSnapshot;

#[test]
fn listed_files_resolve_to_their_realpath_and_each_directory_lists_once() {
    let dir = tempfile::tempdir().unwrap();
    // Probe through the uncanonicalized temp spelling (macOS: `/var` link).
    let root = dir.path();
    fs::create_dir_all(root.join("kit")).unwrap();
    fs::create_dir_all(root.join("features/a")).unwrap();
    fs::create_dir_all(root.join("features/b")).unwrap();
    for name in ["Field.vue", "Tabs.vue"] {
        fs::write(root.join("kit").join(name), "<template />").unwrap();
    }
    let sources = TypeSourceSnapshot::default();
    for feature in ["a", "b"] {
        let importer = root.join("features").join(feature).join("Widget.vue");
        for name in ["Field.vue", "Tabs.vue"] {
            let specifier = vize_carton::cstr!("../../kit/{name}");
            assert_eq!(
                sources.resolve_import(&importer, &specifier),
                Some(root.join("kit").join(name).canonicalize().unwrap()),
            );
        }
    }
    // Each importer directory spells `kit` differently, and lists it once.
    let kit_spellings = sources
        .directories
        .lock()
        .unwrap()
        .keys()
        .filter(|directory| directory.ends_with("kit"))
        .count();
    assert_eq!(kit_spellings, 2);
}

#[test]
fn files_created_after_a_listing_still_resolve_in_the_same_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::write(root.join("first.ts"), "export type First = 1").unwrap();
    let importer = root.join("App.vue");
    let sources = TypeSourceSnapshot::default();
    assert_eq!(
        sources.resolve_import(&importer, "./first"),
        Some(root.join("first.ts"))
    );
    fs::write(root.join("later.ts"), "export type Later = 2").unwrap();
    assert_eq!(
        sources.resolve_import(&importer, "./later"),
        Some(root.join("later.ts"))
    );
}

#[test]
fn case_mismatched_specifiers_keep_the_filesystem_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::write(root.join("api.ts"), "export type Api = 1").unwrap();
    let importer = root.join("App.vue");
    let sources = TypeSourceSnapshot::default();
    assert_eq!(
        sources.resolve_import(&importer, "./api"),
        Some(root.join("api.ts"))
    );
    let upper = root.join("API.ts");
    // On a case-insensitive volume the mismatched spelling is the same file,
    // and its identity is the on-disk name `realpath` reports.
    assert_eq!(
        sources.resolve_import(&importer, "./API"),
        upper.canonicalize().ok()
    );
}

#[cfg(unix)]
#[test]
fn links_keep_their_physical_identity() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("real/sub")).unwrap();
    fs::create_dir_all(root.join("app")).unwrap();
    fs::write(root.join("real/target.ts"), "export type Target = 1").unwrap();
    fs::write(root.join("real/sub/inner.ts"), "export type Inner = 1").unwrap();
    std::os::unix::fs::symlink(root.join("real/target.ts"), root.join("app/alias.ts")).unwrap();
    std::os::unix::fs::symlink(root.join("real/sub"), root.join("app/linked")).unwrap();
    let importer = root.join("app/App.vue");
    let sources = TypeSourceSnapshot::default();
    let expect = |specifier: &str, physical: &Path| {
        assert_eq!(
            sources.resolve_import(&importer, specifier),
            Some(physical.to_path_buf()),
            "{specifier}"
        );
    };
    // A linked file, a file inside a linked directory, and `..` taken
    // physically after a link all resolve as `realpath` does.
    expect("./alias", &root.join("real/target.ts"));
    expect("./linked/inner", &root.join("real/sub/inner.ts"));
    expect("./linked/../target", &root.join("real/target.ts"));
}

#[cfg(unix)]
#[test]
fn caller_roots_take_their_realpath_from_the_physical_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("real/app")).unwrap();
    for name in ["First.vue", "Second.vue"] {
        fs::write(root.join("real/app").join(name), "<template />").unwrap();
    }
    std::os::unix::fs::symlink(root.join("real/app"), root.join("linked")).unwrap();
    let sources = TypeSourceSnapshot::default();
    for spelling in [root.join("linked"), root.join("real/app")] {
        for name in ["First.vue", "Second.vue"] {
            assert_eq!(
                sources.public_identity(&spelling.join(name)),
                root.join("real/app").join(name)
            );
        }
    }
    // Both spellings reach one physical directory.
    assert_eq!(sources.physical_directories.lock().unwrap().len(), 1);
    // Unsaved, lexical `..` and relative roots keep `source_path` exactly.
    for path in [
        root.join("linked/Unsaved.vue"),
        root.join("linked/../real/app/First.vue"),
        Path::new("Relative.vue").to_path_buf(),
    ] {
        assert_eq!(
            sources.public_identity(&path),
            super::source_path(&path),
            "{path:?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn walked_candidate_directories_match_realpath_through_links_and_parents() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("src/features/a")).unwrap();
    fs::create_dir_all(root.join("src/kit")).unwrap();
    fs::create_dir_all(root.join("elsewhere/shared")).unwrap();
    fs::write(root.join("src/features/a/App.vue"), "<template />").unwrap();
    fs::write(root.join("src/features/a/model.ts"), "export type M = 1").unwrap();
    fs::write(root.join("src/kit/Field.vue"), "<template />").unwrap();
    fs::write(root.join("elsewhere/shared/types.ts"), "export type T = 1").unwrap();
    std::os::unix::fs::symlink(root.join("elsewhere/shared"), root.join("src/shared")).unwrap();
    let sources = TypeSourceSnapshot::default();
    let importer = root.join("src/features/a/App.vue");
    for (specifier, expected) in [
        ("./model", root.join("src/features/a/model.ts")),
        ("../../kit/Field.vue", root.join("src/kit/Field.vue")),
        // A link on the way keeps the full canonicalization.
        ("../../shared/types", root.join("elsewhere/shared/types.ts")),
    ] {
        assert_eq!(
            sources.resolve_import(&importer, specifier),
            Some(expected),
            "{specifier}"
        );
    }
    let mut known: Vec<_> = sources
        .canonical_directories
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect();
    known.sort();
    assert_eq!(
        known,
        [
            root.join("elsewhere/shared"),
            root.join("src/features/a"),
            root.join("src/kit"),
        ]
    );
}
