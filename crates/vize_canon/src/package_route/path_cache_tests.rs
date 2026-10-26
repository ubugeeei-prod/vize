use super::PackagePathCache;
use vize_carton::path::canonicalize_non_verbatim;

#[test]
fn successful_logical_path_also_caches_its_genuine_physical_spelling() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("dir")).unwrap();
    std::fs::write(root.path().join("entry.ts"), "export {};\n").unwrap();
    let logical = root.path().join("dir/../entry.ts");
    let physical = canonicalize_non_verbatim(&logical);
    let mut cache = PackagePathCache::default();
    assert_eq!(cache.canonicalize(&logical), physical);
    assert_eq!(cache.canonicalizations, 1);
    assert_eq!(cache.canonicalize(&physical), physical);
    assert_eq!(cache.canonicalizations, 1);
    cache.clear();
    assert_eq!(cache.canonicalize(&physical), physical);
    assert_eq!(cache.canonicalizations, 2);
}

#[cfg(unix)]
#[test]
fn physical_identity_keeps_symlink_boundaries_hardlinks_failures_and_epoch_refresh() {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("real");
    let other = root.path().join("other");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(real.join("entry.ts"), "export {};\n").unwrap();
    std::fs::write(other.join("entry.ts"), "export {};\n").unwrap();
    std::fs::hard_link(real.join("entry.ts"), real.join("hardlink.ts")).unwrap();
    let alias = root.path().join("alias");
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    std::os::unix::fs::symlink(real.join("entry.ts"), real.join("leaf.ts")).unwrap();
    let cycle = root.path().join("cycle");
    std::os::unix::fs::symlink(&cycle, &cycle).unwrap();
    let paths = [
        alias.join("entry.ts"),
        alias.join("../real/entry.ts"),
        real.join("leaf.ts"),
        real.join("hardlink.ts"),
        real.join("missing.ts"),
        real.join("missing/entry.ts"),
        cycle.join("entry.ts"),
    ];
    let expected = paths
        .iter()
        .map(|path| canonicalize_non_verbatim(path))
        .collect::<Vec<_>>();
    let mut cache = PackagePathCache::default();
    let actual = paths
        .iter()
        .map(|path| cache.canonicalize(path))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_ne!(actual[0], actual[3]);
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&other, &alias).unwrap();
    cache.clear();
    assert_eq!(
        cache.canonicalize(&alias.join("entry.ts")),
        canonicalize_non_verbatim(&other.join("entry.ts"))
    );
}

#[test]
fn listed_directories_answer_absent_probes_without_filesystem_calls() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.ts"), "export {};\n").unwrap();
    let present = dir.join("index.ts");
    let absent = dir.join("index.d.ts");
    let nested_absent = dir.join("missing/deep/index.ts");
    let mut cache = PackagePathCache::default();
    assert!(cache.is_file(&present));
    assert!(!cache.is_file(&absent));
    assert!(!cache.is_file(&nested_absent));
    // Absent entries normalize without a realpath; a present one still resolves.
    assert_eq!(
        cache.canonicalize(&absent),
        canonicalize_non_verbatim(&absent)
    );
    assert_eq!(
        cache.canonicalize(&nested_absent),
        canonicalize_non_verbatim(&nested_absent)
    );
    assert_eq!(cache.canonicalizations, 0);
    assert_eq!(
        cache.canonicalize(&present),
        canonicalize_non_verbatim(&present)
    );
    assert_eq!(cache.canonicalizations, 1);
    // The listing is an epoch snapshot, like the realpaths beside it.
    std::fs::write(&absent, "export {};\n").unwrap();
    assert!(!cache.is_file(&absent));
    cache.clear();
    assert!(cache.is_file(&absent));
}

#[test]
fn probes_that_differ_only_in_case_or_normalization_keep_their_stat() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Index.ts"), "export {};\n").unwrap();
    // Beside non-ASCII entries an ASCII probe keeps its stat too: on APFS,
    // `k.ts` matches a Kelvin-sign `K.ts` through case folding.
    let unicode = dir.join("unicode");
    std::fs::create_dir_all(&unicode).unwrap();
    std::fs::write(unicode.join("caf\u{e9}.ts"), "export {};\n").unwrap();
    std::fs::write(unicode.join("\u{212a}.ts"), "export {};\n").unwrap();
    let mut cache = PackagePathCache::default();
    for probe in [
        dir.join("index.ts"),
        dir.join("INDEX.TS"),
        dir.join("other.ts"),
        unicode.join("cafe\u{301}.ts"),
        unicode.join("k.ts"),
        unicode.join("K.ts"),
        unicode.join("other.ts"),
    ] {
        assert_eq!(
            cache.is_file(&probe),
            probe.is_file(),
            "{}",
            probe.display()
        );
    }
}

#[test]
fn windows_alias_spellings_keep_their_stat() {
    for name in ["LONGFI~1.TS", "Index.ts.", "Index.ts ", "Index.ts:stream"] {
        assert!(super::may_alias_on_windows(name.as_bytes()), "{name}");
    }
    for name in ["Index.ts", "index.d.ts", ".gitignore", "a b.ts"] {
        assert!(!super::may_alias_on_windows(name.as_bytes()), "{name}");
    }
}

#[cfg(windows)]
#[test]
fn windows_opens_listed_entries_through_alias_spellings() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("pkg");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Index.ts"), "export {};\n").unwrap();
    let mut cache = PackagePathCache::default();
    // Win32 strips trailing dots and spaces before the lookup, so these open
    // `Index.ts`; the listing must leave them to the stat.
    for probe in [dir.join("Index.ts."), dir.join("Index.ts ")] {
        assert_eq!(
            cache.is_file(&probe),
            probe.is_file(),
            "{}",
            probe.display()
        );
    }
}
