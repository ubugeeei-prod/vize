use super::TypeSourceSnapshot;
use std::{fs, path::Path, sync::Arc};

fn package(directory: &Path, name: &str) -> std::path::PathBuf {
    let root = directory.join("node_modules/@scope/api");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"exports":{".":{"types":"./index.d.ts","import":"./import.d.mts","require":"./require.d.cts"}}}"#,
    )
    .unwrap();
    let declaration = root.join("index.d.ts");
    fs::write(
        &declaration,
        format!("export type Props = {{ {name}: string }}"),
    )
    .unwrap();
    declaration.canonicalize().unwrap()
}

#[test]
fn sibling_importers_share_only_the_same_directory_and_exact_specifier() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let api = root.join("api.ts");
    fs::write(&api, "export type Props = { value: string }").unwrap();
    let outer_package = package(&root, "outer");
    let nested = root.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let inner_package = package(&nested, "inner");
    let sources = TypeSourceSnapshot::default();
    // The compatibility type resolver has no importer-mode input. Its own
    // existing types-export preference applies equally to .mts and .cts.
    for name in ["First.vue", "Second.vue", "Import.mts", "Require.cts"] {
        let importer = root.join(name);
        assert_eq!(
            sources.resolve_import(&importer, "./api"),
            Some(api.clone())
        );
        assert_eq!(
            sources.resolve_import(&importer, "@scope/api"),
            Some(outer_package.clone())
        );
    }
    assert_eq!(sources.resolutions.lock().unwrap().len(), 2);
    assert_eq!(
        sources.resolve_import(&nested.join("App.vue"), "@scope/api"),
        Some(inner_package)
    );
    assert_eq!(
        sources.resolve_import(&root.join("App.vue"), "./absent"),
        None
    );
    assert_eq!(sources.resolutions.lock().unwrap().len(), 4);
    // A filename that is literally node_modules retains the compatibility
    // resolver's existing package exclusion, even outside that directory.
    assert_eq!(
        sources.resolve_import(&root.join("node_modules"), "@scope/api"),
        None
    );
    assert_eq!(sources.resolutions.lock().unwrap().len(), 5);
    let unusual = root.join("node_modules");
    assert_eq!(sources.resolve_import(&unusual, "./later"), None);
    let later = root.join("later.ts");
    fs::write(&later, "export type Later = string").unwrap();
    assert_eq!(sources.resolve_import(&unusual, "./later"), None);
    assert_eq!(
        sources.resolve_import(&root.join("App.vue"), "./later"),
        Some(later.clone())
    );
    assert_eq!(
        TypeSourceSnapshot::default().resolve_import(&unusual, "./later"),
        Some(later)
    );
}

#[cfg(unix)]
#[test]
fn parentless_importers_retain_absolute_misses_separate_from_root_siblings() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("later.ts");
    let specifier = source.to_str().unwrap();
    let sources = TypeSourceSnapshot::default();
    assert_eq!(sources.resolve_import(Path::new("/"), specifier), None);
    fs::write(&source, "export type Later = string").unwrap();
    assert_eq!(sources.resolve_import(Path::new("/"), specifier), None);
    assert_eq!(
        sources.resolve_import(Path::new("/SnapshotHost.vue"), specifier),
        Some(source.canonicalize().unwrap())
    );
    assert_eq!(
        TypeSourceSnapshot::default().resolve_import(Path::new("/"), specifier),
        Some(source.canonicalize().unwrap())
    );
}

#[test]
fn sibling_reuse_preserves_overlay_js_substitution_and_src_alias_precedence() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let src = root.join("src");
    let components = src.join("components");
    fs::create_dir_all(&components).unwrap();
    let mut overlays = Vec::new();
    for (runtime, source) in [
        ("plain.js", "plain.ts"),
        ("jsx.jsx", "jsx.tsx"),
        ("esm.mjs", "esm.mts"),
        ("common.cjs", "common.cts"),
    ] {
        fs::write(src.join(runtime), "export const runtime = true").unwrap();
        overlays.push((
            src.join(source),
            Arc::<str>::from("export type Props = { overlay: string }"),
        ));
    }
    fs::write(src.join("preferred.ts"), "export type Props = number").unwrap();
    overlays.push((
        src.join("preferred.tsx"),
        Arc::from("export type Props = string"),
    ));
    let sources = TypeSourceSnapshot::new(overlays);
    for name in ["First.vue", "Second.mts", "Third.cts"] {
        let importer = components.join(name);
        for (specifier, source) in [
            ("../plain.js", "plain.ts"),
            ("../jsx.jsx", "jsx.tsx"),
            ("../esm.mjs", "esm.mts"),
            ("../common.cjs", "common.cts"),
            ("@/plain.js", "plain.ts"),
            ("../preferred", "preferred.ts"),
            ("../preferred.tsx", "preferred.tsx"),
        ] {
            assert_eq!(
                sources.resolve_import(&importer, specifier),
                Some(src.join(source))
            );
        }
    }
    assert_eq!(sources.resolutions.lock().unwrap().len(), 7);
}

#[test]
fn directory_hits_and_misses_freeze_only_until_the_next_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("First.vue");
    let second = dir.path().join("Second.vue");
    let api = dir.path().join("api.ts");
    let sources = TypeSourceSnapshot::default();
    assert_eq!(sources.resolve_import(&first, "./api"), None);
    fs::write(&api, "export type Props = { before: string }").unwrap();
    assert_eq!(sources.resolve_import(&second, "./api"), None);
    let fresh = TypeSourceSnapshot::default();
    let canonical = api.canonicalize().unwrap();
    assert_eq!(
        fresh.resolve_import(&first, "./api"),
        Some(canonical.clone())
    );
    assert_eq!(
        fresh.read(&api).as_deref(),
        Some("export type Props = { before: string }")
    );
    fs::write(&api, "export type Props = { after: number }").unwrap();
    assert_eq!(
        fresh.resolve_import(&second, "./api"),
        Some(canonical.clone())
    );
    assert_eq!(
        fresh.read(&api).as_deref(),
        Some("export type Props = { before: string }")
    );
    let edited = TypeSourceSnapshot::default();
    assert_eq!(
        edited.read(&api).as_deref(),
        Some("export type Props = { after: number }")
    );
    fs::remove_file(&api).unwrap();
    assert_eq!(fresh.resolve_import(&second, "./api"), Some(canonical));
    assert_eq!(
        TypeSourceSnapshot::default().resolve_import(&first, "./api"),
        None
    );
    fs::write(&api, "export type Props = boolean").unwrap();
    assert_eq!(
        TypeSourceSnapshot::default().resolve_import(&second, "./api"),
        Some(api.canonicalize().unwrap())
    );
}

#[test]
fn concurrent_siblings_publish_one_resolution_per_directory_and_specifier() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let mut targets = Vec::new();
    for index in 0..8 {
        let source = root.join(format!("api{index}.ts"));
        fs::write(&source, "export type Props = string").unwrap();
        targets.push(source);
    }
    let sources = TypeSourceSnapshot::default();
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        for index in 0..8 {
            let sources = &sources;
            let barrier = &barrier;
            let root = &root;
            let targets = &targets;
            scope.spawn(move || {
                barrier.wait();
                let importer = root.join(format!("Host{index}.vue"));
                for (target_index, target) in targets.iter().enumerate() {
                    assert_eq!(
                        sources.resolve_import(&importer, &format!("./api{target_index}")),
                        Some(target.clone())
                    );
                }
            });
        }
    });
    let resolutions = sources.resolutions.lock().unwrap();
    assert_eq!(resolutions.len(), targets.len());
    assert!(resolutions.values().all(|entry| entry.get().is_some()));
}

#[cfg(unix)]
#[test]
fn public_importers_refresh_retargeted_symlinks_for_saved_and_unsaved_roots() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let targets: Vec<_> = ["first", "second"]
        .into_iter()
        .map(|name| {
            let directory = root.join(name);
            fs::create_dir(&directory).unwrap();
            fs::write(directory.join("App.vue"), "<template />").unwrap();
            fs::write(directory.join("api.ts"), "export type Props = string").unwrap();
            directory
        })
        .collect();
    let link = root.join("selected");
    let sources = TypeSourceSnapshot::default();
    for target in targets {
        std::os::unix::fs::symlink(&target, &link).unwrap();
        for filename in ["App.vue", "Unsaved.vue"] {
            assert_eq!(
                sources.resolve_import(&link.join(filename), "./api"),
                Some(target.join("api.ts"))
            );
        }
        fs::remove_file(&link).unwrap();
    }
    assert_eq!(sources.resolutions.lock().unwrap().len(), 2);
}

#[test]
fn relative_importers_refresh_current_directory_in_an_isolated_process() {
    const CHILD_ENV: &str = "VIZE_SNAPSHOT_RESOLUTION_CWD_CHILD";
    if let Some(root) = std::env::var_os(CHILD_ENV) {
        let sources = TypeSourceSnapshot::default();
        for name in ["first", "second"] {
            let directory = Path::new(&root).join(name);
            std::env::set_current_dir(&directory).unwrap();
            assert_eq!(
                sources.resolve_import(Path::new("App.vue"), "./api"),
                Some(directory.join("api.ts"))
            );
        }
        assert_eq!(sources.resolutions.lock().unwrap().len(), 2);
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    for name in ["first", "second"] {
        fs::create_dir(root.join(name)).unwrap();
        fs::write(root.join(name).join("api.ts"), "export type Props = string").unwrap();
    }
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "script::context::source_snapshot::resolution_tests::relative_importers_refresh_current_directory_in_an_isolated_process",
            "--nocapture",
        ])
        .env(CHILD_ENV, &root)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}",
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(String::from_utf8_lossy(&child.stdout).contains("1 passed"));
}
