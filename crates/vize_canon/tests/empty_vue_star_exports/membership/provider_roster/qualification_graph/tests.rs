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

mod dependency_ownership_successor {
    use super::*;

    struct Inputs {
        predecessor: Value,
        predecessor_bytes: Vec<u8>,
        update: Value,
        lock: Vec<u8>,
        package: Vec<u8>,
        workspace: Vec<u8>,
        owner_bytes: Vec<Vec<u8>>,
    }

    fn historical_owner(path: &str) -> Vec<u8> {
        let bytes = fs::read(support::repository().join(path)).unwrap();
        if path != "npm/cli/package.json" {
            return bytes;
        }
        let current = String::from_utf8(bytes).unwrap();
        current
            .replace("    \"virtual-ts-checker-strict.d.ts\",\n", "")
            .replace(
                "    \"./virtual-ts-checker-strict\": {\n      \"types\": \"./virtual-ts-checker-strict.d.ts\"\n    },\n",
                "",
            )
            .into_bytes()
    }

    fn inputs() -> Inputs {
        Inputs {
            predecessor: fresco_update(),
            predecessor_bytes: fs::read(support::corpus().join(FRESCO_UPDATE)).unwrap(),
            update: dependency_ownership_update(),
            lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
            package: fs::read(support::repository().join("package.json")).unwrap(),
            workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
            owner_bytes: OWNER_MANIFESTS
                .iter()
                .map(|path| historical_owner(path))
                .collect(),
        }
    }

    fn verify(
        input: &Inputs,
        predecessor: &Value,
        update: &Value,
        lock: &[u8],
        owners: &[Vec<u8>],
    ) -> Checked<()> {
        let owners = OWNER_MANIFESTS
            .iter()
            .zip(owners)
            .map(|(path, bytes)| (*path, bytes.as_slice()))
            .collect::<Vec<_>>();
        let graph = OwnershipQualificationGraph {
            lock,
            package: &input.package,
            workspace: &input.workspace,
            owners: &owners,
        };
        verify_dependency_ownership_update(predecessor, &input.predecessor_bytes, update, &graph)
    }

    #[test]
    fn exact_root_dependency_successor_binds_the_whole_current_graph() {
        let input = inputs();
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.lock,
                &input.owner_bytes,
            ),
            Ok(())
        );
    }

    #[test]
    fn historical_fresco_successor_remains_the_exact_predecessor() {
        let input = inputs();
        assert_eq!(support::hash(&input.predecessor_bytes), FRESCO_UPDATE_HASH);
        assert_eq!(
            input.update["previousProviderQualificationRecord"]["sha256"],
            FRESCO_UPDATE_HASH
        );
        assert_eq!(
            input.update["previousQualificationLock"],
            input.predecessor["finalQualificationLock"]
        );
        assert_eq!(
            input.update["previousQualificationPackage"],
            input.predecessor["finalQualificationPackage"]
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
                    &input.owner_bytes,
                ),
                Err("exact current root dependency qualification graph bytes")
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
                &input.owner_bytes,
            ),
            Err("exact frozen root dependency ownership predecessor")
        );

        let mut resolver = input.update.clone();
        resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &resolver,
                &input.lock,
                &input.owner_bytes,
            ),
            Err("exact root dependency resolver fixed point")
        );
    }

    #[test]
    fn foreign_owner_manifests_and_unreviewed_bytes_are_refused() {
        let input = inputs();
        let mut changed = input.update.clone();
        changed["ownerManifests"][6]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &changed,
                &input.lock,
                &input.owner_bytes,
            ),
            Err("exact current owner manifest bytes")
        );

        let mut owners = input.owner_bytes.clone();
        owners[10].push(b' ');
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &input.update,
                &input.lock,
                &owners,
            ),
            Err("exact current owner manifest bytes")
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
            lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
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
