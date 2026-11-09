use super::*;

#[test]
fn historical_tooling_successor_remains_an_exact_predecessor() {
    let predecessor = tooling_update();
    let predecessor_bytes = fs::read(support::corpus().join(TOOLING_UPDATE)).unwrap();
    let update = fresco_update();
    assert_eq!(support::hash(&predecessor_bytes), TOOLING_UPDATE_HASH);
    assert_eq!(
        update["previousProviderQualificationRecord"]["sha256"],
        TOOLING_UPDATE_HASH
    );
    assert_eq!(
        update["previousQualificationLock"],
        predecessor["finalQualificationLock"]
    );
    assert_eq!(
        update["finalQualificationPackage"],
        predecessor["finalQualificationPackage"]
    );
}

#[test]
fn historical_root_dependency_successor_remains_the_exact_predecessor() {
    let predecessor = dependency_ownership_update();
    let predecessor_bytes = fs::read(support::corpus().join(dependency_ownership::UPDATE)).unwrap();
    let update = strict_checker_update();
    assert_eq!(
        support::hash(&predecessor_bytes),
        dependency_ownership::UPDATE_HASH
    );
    assert_eq!(
        update["previousProviderQualificationRecord"]["sha256"],
        dependency_ownership::UPDATE_HASH
    );
    assert_eq!(
        update["previousQualificationLock"],
        predecessor["finalQualificationLock"]
    );
    assert_eq!(
        update["previousQualificationPackage"],
        predecessor["finalQualificationPackage"]
    );
}

#[test]
fn historical_create_fresco_successor_retains_its_frozen_predecessor() {
    let current_predecessor = strict_checker_update();
    let update = create_fresco_update();
    assert_eq!(
        update["previousProviderQualificationRecord"]["sha256"],
        "24019bc545fbecb6d92ef7729ea99248f1fb514e3e0266e321331aec844f8313"
    );
    assert_ne!(
        update["previousProviderQualificationRecord"]["sha256"],
        strict_checker::UPDATE_HASH
    );
    assert_ne!(
        update["previousQualificationLock"],
        current_predecessor["finalQualificationLock"]
    );
    assert_eq!(update["nativeAcceptanceCredit"], false);
}

#[test]
fn historical_docs_successor_closes_the_create_fresco_chain() {
    let predecessor = create_fresco_update();
    let predecessor_bytes = fs::read(support::corpus().join(create_fresco::UPDATE)).unwrap();
    let update = docs_metadata_update();
    assert_eq!(
        support::hash(&predecessor_bytes),
        create_fresco::UPDATE_HASH
    );
    assert_eq!(
        update["previousProviderQualificationRecord"]["sha256"],
        create_fresco::UPDATE_HASH
    );
    assert_eq!(
        update["previousQualificationLock"],
        predecessor["finalQualificationLock"]
    );
    assert_eq!(
        update["previousQualificationPackage"],
        predecessor["finalQualificationPackage"]
    );
    assert_eq!(
        predecessor["resolver"]["fixedPointSha256"],
        predecessor["finalQualificationLock"]["sha256"]
    );
    assert_eq!(
        update["resolver"]["fixedPointSha256"],
        update["finalQualificationLock"]["sha256"]
    );
    assert_eq!(update["nativeAcceptanceCredit"], false);
}

#[test]
fn current_graph_falls_through_to_the_integrated_successor() {
    let previous = super::super::load();
    let lock = fs::read(support::repository().join("pnpm-lock.yaml")).unwrap();
    let package = fs::read(support::repository().join("package.json")).unwrap();
    let update = authorize(&previous, &lock, &package).unwrap().unwrap();
    assert_eq!(
        update["kind"],
        "exact-provider-qualification-integrated-graph-successor"
    );
}
