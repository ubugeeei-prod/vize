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

fn lock_without_create_fresco() -> Vec<u8> {
    let mut lock = fs::read_to_string(support::repository().join("pnpm-lock.yaml")).unwrap();
    let start = lock.find("  npm/create-fresco:\n").unwrap();
    let end = start + lock[start..].find("\n  npm/devtools:\n").unwrap() + 1;
    lock.replace_range(start..end, "");
    lock.into_bytes()
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
fn historical_strict_checker_successor_remains_the_exact_predecessor() {
    let predecessor = strict_checker_update();
    let predecessor_bytes = fs::read(support::corpus().join(strict_checker::UPDATE)).unwrap();
    let update = create_fresco_update();
    assert_eq!(
        support::hash(&predecessor_bytes),
        strict_checker::UPDATE_HASH
    );
    assert_eq!(
        update["previousProviderQualificationRecord"]["sha256"],
        strict_checker::UPDATE_HASH
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
            predecessor: strict_checker_update(),
            predecessor_bytes: fs::read(support::corpus().join(strict_checker::UPDATE)).unwrap(),
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

mod strict_checker_successor {
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
        checker_declaration: Vec<u8>,
        strict_declaration: Vec<u8>,
    }

    fn inputs() -> Inputs {
        Inputs {
            previous: super::super::super::load(),
            predecessor: dependency_ownership_update(),
            predecessor_bytes: fs::read(support::corpus().join(dependency_ownership::UPDATE))
                .unwrap(),
            update: strict_checker_update(),
            lock: lock_without_create_fresco(),
            package: fs::read(support::repository().join("package.json")).unwrap(),
            workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
            manifest: fs::read(support::repository().join(STRICT_CHECKER_MANIFEST)).unwrap(),
            checker_declaration: fs::read(support::repository().join(CHECKER_DECLARATION)).unwrap(),
            strict_declaration: fs::read(support::repository().join(STRICT_DECLARATION)).unwrap(),
        }
    }

    fn verify(
        input: &Inputs,
        predecessor: &Value,
        update: &Value,
        manifest: &[u8],
        checker_declaration: &[u8],
        strict_declaration: &[u8],
    ) -> Checked<()> {
        let graph = StrictCheckerQualificationGraph {
            lock: &input.lock,
            package: &input.package,
            workspace: &input.workspace,
            manifest,
            checker_declaration,
            strict_declaration,
        };
        verify_strict_checker_update(predecessor, &input.predecessor_bytes, update, &graph)
    }

    #[test]
    fn exact_strict_checker_successor_binds_the_whole_current_graph() {
        let input = inputs();
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.manifest,
                &input.checker_declaration,
                &input.strict_declaration,
            ),
            Ok(())
        );
        assert_eq!(
            authorize(&input.previous, &input.lock, &input.package),
            Ok(Some(input.update.clone()))
        );
    }

    #[test]
    fn changed_predecessor_and_fixed_point_are_refused() {
        let input = inputs();
        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &predecessor,
                &input.update,
                &input.manifest,
                &input.checker_declaration,
                &input.strict_declaration,
            ),
            Err("exact frozen strict checker qualification predecessor")
        );

        let mut resolver = input.update.clone();
        resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &resolver,
                &input.manifest,
                &input.checker_declaration,
                &input.strict_declaration,
            ),
            Err("exact strict checker resolver fixed point")
        );
    }

    #[test]
    fn foreign_manifest_and_policy_declarations_are_refused() {
        let input = inputs();
        for (manifest, checker, strict) in [
            (
                b"foreign".as_slice(),
                input.checker_declaration.as_slice(),
                input.strict_declaration.as_slice(),
            ),
            (
                input.manifest.as_slice(),
                b"foreign".as_slice(),
                input.strict_declaration.as_slice(),
            ),
            (
                input.manifest.as_slice(),
                input.checker_declaration.as_slice(),
                b"foreign".as_slice(),
            ),
        ] {
            assert_eq!(
                verify(
                    &input,
                    &input.predecessor,
                    &input.update,
                    manifest,
                    checker,
                    strict,
                ),
                Err("exact current strict checker qualification graph bytes")
            );
        }
    }
}
