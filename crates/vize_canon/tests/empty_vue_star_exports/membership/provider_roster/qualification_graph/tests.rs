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

mod dependency_ownership_successor;

#[test]
fn historical_create_fresco_successor_remains_the_exact_predecessor() {
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
}

mod docs_metadata_successor {
    use super::*;

    struct Inputs {
        previous: Value,
        predecessor: Value,
        predecessor_bytes: Vec<u8>,
        update: Value,
        lock: Vec<u8>,
        package: Vec<u8>,
        workspace: Vec<u8>,
        manifest: Vec<u8>,
    }

    fn inputs() -> Inputs {
        Inputs {
            previous: super::super::super::load(),
            predecessor: create_fresco_update(),
            predecessor_bytes: fs::read(support::corpus().join(create_fresco::UPDATE)).unwrap(),
            update: docs_metadata_update(),
            lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
            package: fs::read(support::repository().join("package.json")).unwrap(),
            workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
            manifest: fs::read(support::repository().join(DOCS_METADATA_MANIFEST)).unwrap(),
        }
    }

    fn verify(
        input: &Inputs,
        predecessor: &Value,
        update: &Value,
        lock: &[u8],
        manifest: &[u8],
    ) -> Checked<()> {
        let graph = DocsMetadataQualificationGraph {
            lock,
            package: &input.package,
            workspace: &input.workspace,
            manifest,
        };
        verify_docs_metadata_update(predecessor, &input.predecessor_bytes, update, &graph)
    }

    #[test]
    fn exact_docs_metadata_successor_binds_the_whole_current_graph() {
        let input = inputs();
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.lock,
                &input.manifest,
            ),
            Ok(())
        );
        assert_eq!(
            authorize(&input.previous, &input.lock, &input.package),
            Ok(Some(input.update.clone()))
        );
    }

    #[test]
    fn changed_graph_predecessor_and_fixed_point_are_refused() {
        let input = inputs();
        for changed in [
            b"foreign lock".as_slice(),
            input.lock.get(..input.lock.len() - 1).unwrap(),
        ] {
            assert_eq!(
                verify(
                    &input,
                    &input.predecessor,
                    &input.update,
                    changed,
                    &input.manifest,
                ),
                Err("exact current docs metadata qualification graph bytes")
            );
        }

        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &predecessor,
                &input.update,
                &input.lock,
                &input.manifest,
            ),
            Err("exact frozen docs metadata qualification predecessor")
        );

        let mut resolver = input.update.clone();
        resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &resolver,
                &input.lock,
                &input.manifest,
            ),
            Err("exact docs metadata resolver fixed point")
        );
    }

    #[test]
    fn foreign_manifest_and_unreviewed_bytes_are_refused() {
        let input = inputs();
        let mut changed = input.update.clone();
        changed["docsManifest"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &changed,
                &input.lock,
                &input.manifest,
            ),
            Err("exact current docs metadata qualification graph bytes")
        );

        let mut manifest = input.manifest.clone();
        manifest.push(b' ');
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.lock,
                &manifest,
            ),
            Err("exact current docs metadata qualification graph bytes")
        );
    }
}

#[test]
fn historical_root_dependency_successor_remains_the_exact_predecessor() {
    let predecessor = dependency_ownership_update();
    let predecessor_bytes = fs::read(support::corpus().join(dependency_ownership::UPDATE)).unwrap();
    let update = create_fresco_update();
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

mod create_fresco_successor {
    use super::*;

    struct Inputs {
        predecessor: Value,
        predecessor_bytes: Vec<u8>,
        update: Value,
        lock: Vec<u8>,
        package: Vec<u8>,
        workspace: Vec<u8>,
        manifest: Vec<u8>,
    }

    fn inputs() -> Inputs {
        Inputs {
            predecessor: dependency_ownership_update(),
            predecessor_bytes: fs::read(support::corpus().join(dependency_ownership::UPDATE))
                .unwrap(),
            update: create_fresco_update(),
            lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
            package: fs::read(support::repository().join("package.json")).unwrap(),
            workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
            manifest: fs::read(support::repository().join(CREATE_FRESCO_MANIFEST)).unwrap(),
        }
    }

    fn verify(
        input: &Inputs,
        predecessor: &Value,
        update: &Value,
        lock: &[u8],
        manifest: &[u8],
    ) -> Checked<()> {
        let graph = CreateFrescoQualificationGraph {
            lock,
            package: &input.package,
            workspace: &input.workspace,
            manifest,
        };
        verify_create_fresco_update(predecessor, &input.predecessor_bytes, update, &graph)
    }

    #[test]
    fn exact_create_fresco_successor_binds_the_whole_current_graph() {
        let input = inputs();
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.lock,
                &input.manifest,
            ),
            Ok(())
        );
    }

    #[test]
    fn changed_graph_predecessor_and_fixed_point_are_refused() {
        let input = inputs();
        for changed in [
            b"foreign lock".as_slice(),
            input.lock.get(..input.lock.len() - 1).unwrap(),
        ] {
            assert_eq!(
                verify(
                    &input,
                    &input.predecessor,
                    &input.update,
                    changed,
                    &input.manifest,
                ),
                Err("exact current create-fresco qualification graph bytes")
            );
        }

        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &predecessor,
                &input.update,
                &input.lock,
                &input.manifest,
            ),
            Err("exact frozen create-fresco qualification predecessor")
        );

        let mut resolver = input.update.clone();
        resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &resolver,
                &input.lock,
                &input.manifest,
            ),
            Err("exact create-fresco resolver fixed point")
        );
    }

    #[test]
    fn foreign_manifest_and_unreviewed_bytes_are_refused() {
        let input = inputs();
        let mut changed = input.update.clone();
        changed["createFrescoManifest"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &changed,
                &input.lock,
                &input.manifest,
            ),
            Err("exact current create-fresco qualification graph bytes")
        );

        let mut manifest = input.manifest.clone();
        manifest.push(b' ');
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.lock,
                &manifest,
            ),
            Err("exact current create-fresco qualification graph bytes")
        );
    }
}
