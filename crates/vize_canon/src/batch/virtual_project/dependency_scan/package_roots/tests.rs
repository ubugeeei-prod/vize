use std::path::{Path, PathBuf};

use super::PackageRoots;

fn old_membership(roots: &[PathBuf], importer: &Path, target: &Path) -> bool {
    roots
        .iter()
        .filter(|root| importer.starts_with(root))
        .any(|root| target.starts_with(root))
}

fn membership(roots: &PackageRoots, importer: &Path, target: &Path) -> bool {
    roots
        .for_importer(importer)
        .iter()
        .any(|root| target.starts_with(root))
}

#[test]
fn nested_duplicate_and_component_boundary_roots_keep_every_matching_owner() {
    let roots = ["/repo", "/repo/pkg", "/repo/pkg", "/repo/pkg/nested"].map(PathBuf::from);
    let indexed = PackageRoots::new(roots.iter().map(PathBuf::as_path));
    let targets = [
        "/repo/pkg/Child.vue",
        "/repo/other/Child.vue",
        "/other/Child.vue",
        "/repo/pkg2/Child.vue",
        "/repository/Child.vue",
    ];
    for (importer, expected) in [
        ("/repo/pkg/src/main.ts", [true, true, false, true, false]),
        ("/repo/pkg/nested/main.ts", [true, true, false, true, false]),
        ("/other/main.ts", [false; 5]),
        ("/repository/main.ts", [false; 5]),
    ] {
        let actual =
            targets.map(|target| membership(&indexed, Path::new(importer), Path::new(target)));
        assert_eq!(actual, expected);
        assert_eq!(
            actual,
            targets.map(|target| old_membership(&roots, Path::new(importer), Path::new(target)))
        );
    }
}

#[test]
fn relative_dot_parent_and_empty_roots_retain_path_component_semantics() {
    let roots = ["packages/a", "packages/a/.", "./packages/b", "../shared"].map(PathBuf::from);
    let indexed = PackageRoots::new(roots.iter().map(PathBuf::as_path));
    let targets = [
        "packages/a/Child.vue",
        "packages/a-else/Child.vue",
        "./packages/b/Child.vue",
        "../shared/Child.vue",
        "/abs/Child.vue",
    ];
    for (importer, expected) in [
        ("packages/a/main.ts", [true, false, false, false, false]),
        ("./packages/b/main.ts", [false, false, true, false, false]),
        ("../shared/main.ts", [false, false, false, true, false]),
        ("unrelated/main.ts", [false; 5]),
    ] {
        let actual =
            targets.map(|target| membership(&indexed, Path::new(importer), Path::new(target)));
        assert_eq!(actual, expected);
        assert_eq!(
            actual,
            targets.map(|target| old_membership(&roots, Path::new(importer), Path::new(target)))
        );
    }
    let empty = PackageRoots::new([Path::new("")].into_iter());
    for importer in ["", "relative.ts", "/abs/main.ts"] {
        assert_eq!(
            targets.map(|target| membership(&empty, Path::new(importer), Path::new(target))),
            [true; 5]
        );
    }
    let absent = PackageRoots::new(std::iter::empty());
    assert!(!membership(
        &absent,
        Path::new("/repo/main.ts"),
        Path::new("/repo/Child.vue")
    ));
}

#[cfg(unix)]
#[test]
fn raw_symlink_roots_are_not_replaced_by_the_canonical_target_index() {
    let temp = tempfile::tempdir().unwrap();
    let base = std::fs::canonicalize(temp.path()).unwrap();
    let physical = base.join("packages/pkg");
    let logical = base.join("node_modules/pkg");
    std::fs::create_dir_all(&physical).unwrap();
    std::fs::create_dir_all(logical.parent().unwrap()).unwrap();
    std::fs::write(physical.join("Child.vue"), "<template />").unwrap();
    std::fs::write(physical.join("main.ts"), "export {};").unwrap();
    std::os::unix::fs::symlink(&physical, &logical).unwrap();
    let importer = logical.join("main.ts");
    let canonical_importer = std::fs::canonicalize(&importer).unwrap();
    let target = std::fs::canonicalize(logical.join("Child.vue")).unwrap();
    let roots = [logical];
    let indexed = PackageRoots::new(roots.iter().map(PathBuf::as_path));
    assert_eq!(
        [
            old_membership(&roots, &importer, &target),
            membership(&indexed, &importer, &target)
        ],
        [false, false]
    );
    assert_eq!(
        [
            old_membership(&roots, &canonical_importer, &target),
            membership(&indexed, &canonical_importer, &target)
        ],
        [false, false]
    );
    let broader = [roots[0].clone(), base];
    let indexed = PackageRoots::new(broader.iter().map(PathBuf::as_path));
    assert_eq!(
        [
            old_membership(&broader, &importer, &target),
            membership(&indexed, &importer, &target)
        ],
        [true, true]
    );
}

#[cfg(windows)]
#[test]
fn windows_drive_unc_and_verbatim_prefix_roots_match_the_previous_predicate() {
    for (root, importer, target, expected) in [
        (r"C:", r"C:\pkg\main.ts", r"C:\other\Child.vue", true),
        (r"C:", r"C:\pkg\main.ts", r"D:\pkg\Child.vue", false),
        (r"C:", r"C:pkg\main.ts", r"C:other\Child.vue", true),
        (r"C:\pkg", r"C:\pkg\main.ts", r"C:\pkg2\Child.vue", false),
        (
            r"\\server\share",
            r"\\server\share\pkg\main.ts",
            r"\\server\share\other\Child.vue",
            true,
        ),
        (
            r"\\server\share",
            r"\\server\share\pkg\main.ts",
            r"\\server\other\Child.vue",
            false,
        ),
        (
            r"\\?\C:",
            r"\\?\C:\pkg\main.ts",
            r"\\?\C:\other\Child.vue",
            true,
        ),
        (
            r"\\?\C:\pkg",
            r"\\?\C:\pkg\main.ts",
            r"C:\pkg\Child.vue",
            false,
        ),
    ] {
        let roots = [PathBuf::from(root)];
        let indexed = PackageRoots::new(roots.iter().map(PathBuf::as_path));
        assert_eq!(
            old_membership(&roots, Path::new(importer), Path::new(target)),
            expected
        );
        assert_eq!(
            membership(&indexed, Path::new(importer), Path::new(target)),
            expected
        );
    }
}
