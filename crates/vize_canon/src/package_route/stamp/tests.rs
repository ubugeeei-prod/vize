use super::{InputStamp, InputStampCache};
#[cfg(feature = "native")]
use super::{InputStampReadStats, InputStampValidation};

#[test]
fn stamp_cache_captures_each_path_once_per_epoch() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("package.json");
    std::fs::write(&file, r#"{"name":"pkg"}"#).unwrap();
    let stamp = InputStamp::capture(&file);
    let mut cache = InputStampCache::default();

    assert!(stamp.is_current_with_cache(&mut cache));
    assert!(stamp.is_current_with_cache(&mut cache));
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.captures(), 1);
}

#[test]
fn registered_source_stamps_do_not_reuse_filesystem_cache_authority() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.ts");
    let content = b"export const value = 1;";
    std::fs::write(&path, content).unwrap();
    let stamp = InputStamp::capture_source(&path, content);
    let mut cache = InputStampCache::default();
    cache.capture(&path);
    assert!(stamp.is_current());
    assert!(stamp.is_current_with_cache(&mut cache));
    std::fs::write(&path, "export const value = 2;").unwrap();
    assert!(!stamp.is_current());
    assert!(!stamp.is_current_with_cache(&mut cache));
    assert_eq!(cache.captures(), 1);
}

#[test]
fn same_size_and_mtime_edits_still_change_full_content_stamps() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.json");
    let mut content = vec![b' '; 8193];
    std::fs::write(&path, &content).unwrap();
    let physical = InputStamp::capture(&path);
    let source = InputStamp::capture_source(&path, &content);
    let original = std::fs::metadata(&path).unwrap();
    #[cfg(feature = "native")]
    let observed = crate::ConfigurationObservations::capture(std::slice::from_ref(&path));
    #[cfg(feature = "native")]
    assert!(observed.changed_paths().is_empty());
    assert!(physical.is_current());
    assert!(source.is_current());

    // Preserve metadata and the complete prefix; only the final byte changes.
    *content.last_mut().unwrap() = b'!';
    std::fs::write(&path, &content).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(original.modified().unwrap()))
        .unwrap();
    let modified = std::fs::metadata(&path).unwrap();
    assert_eq!(modified.len(), original.len());
    assert_eq!(modified.modified().unwrap(), original.modified().unwrap());
    assert!(!physical.is_current());
    assert!(!source.is_current());
    #[cfg(feature = "native")]
    assert_eq!(observed.changed_paths(), vec![path.clone()]);
    assert!(InputStamp::capture(&path).is_current());
    assert!(InputStamp::capture_source(&path, &content).is_current());
}

#[cfg(unix)]
#[test]
fn source_stamps_track_symlink_content_and_target_without_becoming_always_dirty() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("target.ts");
    let link = root.path().join("source.ts");
    let content = b"export const value = 1;";
    std::fs::write(&target, content).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let stamp = InputStamp::capture_source(&link, content);
    assert!(stamp.is_current());
    std::fs::write(&target, "export const value = 2;").unwrap();
    assert!(!stamp.is_current());
    let other = root.path().join("other.ts");
    std::fs::write(&other, content).unwrap();
    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&other, &link).unwrap();
    assert!(!stamp.is_current());
    assert!(InputStamp::capture_source(&link, content).is_current());
}

#[cfg(feature = "native")]
#[test]
fn validation_compares_every_consumed_revision_to_fresh_bytes_in_both_orders() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.ts");
    std::fs::write(&path, b"omega").unwrap();
    let stale = InputStamp::capture_source(&path, b"alpha");
    let current = InputStamp::capture_source(&path, b"omega");
    for (stamps, expected) in [
        ([&stale, &current], [false, true]),
        ([&current, &stale], [true, false]),
    ] {
        let mut validation = InputStampValidation::default();
        let mut stats = InputStampReadStats::default();
        let actual = stamps.map(|stamp| validation.is_current(stamp, Some(&mut stats)));
        assert_eq!(actual, expected);
        assert_eq!(validation.len(), 1);
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.bytes, 5);
    }
}

#[cfg(feature = "native")]
#[test]
fn validation_does_not_reuse_absence_across_calls() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("absent.json");
    let observed = crate::ConfigurationObservations::capture(std::slice::from_ref(&path));
    assert!(observed.changed_paths().is_empty());
    std::fs::write(&path, b"{}").unwrap();
    assert_eq!(observed.changed_paths(), vec![path]);
}

#[test]
fn content_identity_retains_the_complete_standard_sha256_digest() {
    assert_eq!(
        super::digest(b""),
        [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ],
    );
    assert_eq!(
        super::digest(b"abc"),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ],
    );
}

/// Installed files are stamped by metadata, never read; a same-size,
/// same-mtime rewrite still moves their status-change time. Authored
/// sources keep a content digest.
#[cfg(unix)]
#[test]
fn installed_files_use_metadata_identity_and_still_see_same_mtime_rewrites() {
    let root = tempfile::tempdir().unwrap();
    let installed = root.path().join("node_modules/pkg/index.d.ts");
    let authored = root.path().join("src/index.ts");
    for path in [&installed, &authored] {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "export const a = 1;\n").unwrap();
    }
    // A just-written installed file is racy: hashed until it settles.
    let stamp = super::InputStamp::capture(&installed);
    assert!(stamp.content_digest.is_some());
    assert!(stamp.installed_identity.is_some());
    assert!(
        super::InputStamp::capture(&authored)
            .content_digest
            .is_some()
    );

    let modified = std::fs::metadata(&installed).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(&installed, "export const b = 2;\n").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&installed)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(!stamp.is_current());
}

/// A settled installed file compares by metadata alone; a manifest keeps
/// its digest.
#[cfg(unix)]
#[test]
fn settled_installed_stamps_compare_by_metadata_and_manifests_stay_hashed() {
    let root = tempfile::tempdir().unwrap();
    let manifest = root.path().join("node_modules/pkg/package.json");
    std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    std::fs::write(&manifest, "{}").unwrap();
    let hashed = super::InputStamp::capture(&manifest);
    assert!(hashed.installed_identity.is_none());
    assert!(hashed.content_digest.is_some());

    let installed = root.path().join("node_modules/pkg/index.d.ts");
    std::fs::write(&installed, "export {};\n").unwrap();
    let racy = super::InputStamp::capture(&installed);
    let settled = super::InputStamp {
        content_digest: None,
        ..racy.clone()
    };
    assert_eq!(racy, settled);
    let moved = super::InputStamp {
        installed_identity: racy
            .installed_identity
            .map(|(dev, ino, sec, nsec)| (dev, ino, sec, nsec + 1)),
        ..settled.clone()
    };
    assert_ne!(settled, moved);
}
