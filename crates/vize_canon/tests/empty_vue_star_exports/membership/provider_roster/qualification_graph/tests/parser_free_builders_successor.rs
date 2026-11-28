use super::*;

#[test]
fn parser_free_builders_successor_binds_the_whole_current_lock_to_its_predecessor() {
    let lock = fs::read(support::repository().join("pnpm-lock.yaml")).unwrap();
    let package = fs::read(support::repository().join("package.json")).unwrap();
    let previous_lock =
        crate::package_task_history::before_parser_free_builders("pnpm-lock.yaml", &lock).unwrap();
    let root = super::super::super::load();
    let predecessor = authorize(&root, &previous_lock, &package).unwrap().unwrap();
    let current = authorize(&root, &lock, &package).unwrap().unwrap();
    assert_eq!(
        current["kind"],
        "exact-provider-qualification-parser-free-builders-successor"
    );
    assert_eq!(current["nativeAcceptanceCredit"], false);
    assert_eq!(
        current["previousQualificationLock"],
        predecessor["finalQualificationLock"]
    );
    assert_eq!(
        current["finalQualificationPackage"],
        predecessor["finalQualificationPackage"]
    );
    assert_eq!(
        current["workspaceManifest"],
        predecessor["workspaceManifest"]
    );
    assert!(bound(
        &current["finalQualificationLock"],
        "pnpm-lock.yaml",
        &lock
    ));
    let mut changed = lock.clone();
    changed.push(b'\n');
    assert!(
        crate::package_task_history::before_parser_free_builders("pnpm-lock.yaml", &changed)
            .is_none()
    );
}
