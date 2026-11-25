use super::*;

#[test]
fn resolver_importer_successor_reuses_the_exact_current_graph_predecessor() {
    let lock = fs::read(support::repository().join("pnpm-lock.yaml")).unwrap();
    let package = fs::read(support::repository().join("package.json")).unwrap();
    let previous_lock =
        crate::package_task_history::before_vite_resolver_importer("pnpm-lock.yaml", &lock)
            .unwrap();
    let root = super::super::super::load();
    let predecessor = authorize(&root, &previous_lock, &package).unwrap().unwrap();
    let current = authorize(&root, &lock, &package).unwrap().unwrap();
    assert_eq!(
        current["kind"],
        "exact-provider-qualification-vite-resolver-importer-successor"
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
    assert!(bound(
        &current["previousQualificationLock"],
        "pnpm-lock.yaml",
        &previous_lock
    ));
}
