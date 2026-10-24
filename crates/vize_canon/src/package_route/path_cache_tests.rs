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
