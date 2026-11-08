use std::{ffi::OsStr, path::Path};

use super::{ConfigMarker, config_marker_name, document_root, is_config_marker};
#[cfg(target_os = "linux")]
use super::{DirectoryMarkerCache, cached_document_root};

#[test]
fn marker_names_keep_the_complete_authored_policy() {
    for (name, expected) in [
        ("package.json", Some(ConfigMarker::Package)),
        ("vite.config.ts", Some(ConfigMarker::Vite)),
        ("vite.config.", Some(ConfigMarker::Vite)),
        ("vite.config.arbitrary", Some(ConfigMarker::Vite)),
        ("vite.config.🌋", Some(ConfigMarker::Vite)),
        ("tsconfig.json", Some(ConfigMarker::TypeScript)),
        ("tsconfig.generated.json", Some(ConfigMarker::TypeScript)),
        ("tsconfig𐐀.json", Some(ConfigMarker::TypeScript)),
        ("jsconfig.json", Some(ConfigMarker::TypeScript)),
        ("Widget.vue", None),
        ("package.json.txt", None),
        ("vite.config", None),
        ("Vite.config.ts", None),
        ("tsconfig.json.txt", None),
        ("jsconfig.generated.json", None),
    ] {
        assert_eq!(config_marker_name(OsStr::new(name)), expected, "{name}");
        assert_eq!(
            is_config_marker(Path::new(name)),
            expected.is_some(),
            "{name}"
        );
    }
}

#[cfg(unix)]
#[test]
fn byte_prefilters_do_not_admit_non_utf8_markers() {
    use std::os::unix::ffi::OsStrExt;
    for bytes in [
        b"vite.config.\xff".as_slice(),
        b"tsconfig.\xff.json".as_slice(),
        b"ordinary.\xff.vue".as_slice(),
    ] {
        assert_eq!(config_marker_name(OsStr::from_bytes(bytes)), None);
        assert!(!is_config_marker(Path::new(OsStr::from_bytes(bytes))));
    }
}

#[test]
fn arbitrary_unannounced_marker_membership_stays_fresh() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let package = root.join("package");
    let src = package.join("src");
    std::fs::create_dir_all(&src).unwrap();
    let document = src.join("App.vue");
    std::fs::write(&document, "<template />").unwrap();
    std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
    assert_eq!(document_root(&document, root), root);
    std::fs::write(package.join("package.json"), "{}").unwrap();
    std::fs::write(package.join("vite.config.arbitrary"), "export default {};").unwrap();
    assert_eq!(document_root(&document, root), package);
    let config = src.join("tsconfig.generated.𐐀.json");
    std::fs::write(&config, "{}").unwrap();
    assert_eq!(document_root(&document, root), src);
    std::fs::remove_file(config).unwrap();
    assert_eq!(document_root(&document, root), package);
    std::fs::remove_file(package.join("vite.config.arbitrary")).unwrap();
    assert_eq!(document_root(&document, root), root);
}

#[cfg(target_os = "linux")]
#[test]
fn stable_directory_generations_reuse_marker_membership_and_mutations_rescan() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let package = root.join("package");
    let src = package.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
    std::fs::write(package.join("package.json"), "{}").unwrap();
    let cache = DirectoryMarkerCache::default();

    assert_eq!(cached_document_root(&src.join("A.vue"), root, &cache), root);
    let scans = cache.scans();
    assert!(scans >= 3);
    assert_eq!(cached_document_root(&src.join("B.vue"), root, &cache), root);
    assert_eq!(cache.scans(), scans);

    // File-content writes do not change marker membership or the directory's
    // entry generation, so sibling routing still reuses the classified set.
    let ordinary = src.join("ordinary.txt");
    std::fs::write(&ordinary, "one").unwrap();
    assert_eq!(cached_document_root(&src.join("C.vue"), root, &cache), root);
    let after_create = cache.scans();
    assert!(after_create > scans);
    std::fs::write(&ordinary, "two").unwrap();
    assert_eq!(cached_document_root(&src.join("D.vue"), root, &cache), root);
    assert_eq!(cache.scans(), after_create);

    let nested = src.join("tsconfig.generated.json");
    std::fs::write(&nested, "{}").unwrap();
    assert_eq!(cached_document_root(&src.join("E.vue"), root, &cache), src);
    let after_marker = cache.scans();
    assert!(after_marker > after_create);
    std::fs::remove_file(nested).unwrap();
    assert_eq!(cached_document_root(&src.join("F.vue"), root, &cache), root);
    assert!(cache.scans() > after_marker);
}

#[cfg(target_os = "linux")]
#[test]
fn restored_directory_mtime_cannot_hide_new_marker_membership() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(root.join("tsconfig.json"), "{}").unwrap();
    let cache = DirectoryMarkerCache::default();
    let document = src.join("App.vue");
    assert_eq!(cached_document_root(&document, root, &cache), root);
    let modified = std::fs::metadata(&src).unwrap().modified().unwrap();

    std::fs::write(src.join("tsconfig.hidden.json"), "{}").unwrap();
    std::fs::File::options()
        .read(true)
        .open(&src)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();

    assert_eq!(cached_document_root(&document, root, &cache), src);
}
