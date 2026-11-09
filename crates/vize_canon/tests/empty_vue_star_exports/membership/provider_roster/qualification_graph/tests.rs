use super::js_tooling_refresh::{
    JsToolingRefreshQualificationGraph, update as js_tooling_refresh_update,
    verify as verify_js_tooling_refresh_update,
};
use super::*;

fn current_binding(path: &str, body: &[u8]) -> Value {
    json!({
        "path": path,
        "bytes": body.len(),
        "sha256": support::hash(body),
    })
}

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

mod docs_metadata_successor {
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

    fn bind_current_graph(input: &Inputs, update: &mut Value) {
        update["finalQualificationLock"] = current_binding("pnpm-lock.yaml", &input.lock);
        update["finalQualificationPackage"] = current_binding("package.json", &input.package);
        update["workspaceManifest"] = current_binding("pnpm-workspace.yaml", &input.workspace);
        update["docsManifest"] = current_binding(DOCS_METADATA_MANIFEST, &input.manifest);
        update["resolver"]["fixedPointSha256"] = json!(support::hash(&input.lock));
    }

    #[test]
    fn rebound_docs_metadata_successor_satisfies_structural_rules() {
        let input = inputs();
        let mut update = input.update.clone();
        bind_current_graph(&input, &mut update);
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &update,
                &input.lock,
                &input.manifest,
            ),
            Ok(())
        );
    }

    #[test]
    fn changed_graph_predecessor_and_fixed_point_are_refused() {
        let input = inputs();
        let mut update = input.update.clone();
        bind_current_graph(&input, &mut update);
        for changed in [
            b"foreign lock".as_slice(),
            input.lock.get(..input.lock.len() - 1).unwrap(),
        ] {
            assert_eq!(
                verify(
                    &input,
                    &input.predecessor,
                    &update,
                    changed,
                    &input.manifest,
                ),
                Err("exact current docs metadata qualification graph bytes")
            );
        }

        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(&input, &predecessor, &update, &input.lock, &input.manifest,),
            Err("exact frozen docs metadata qualification predecessor")
        );

        let mut resolver = update;
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
        bind_current_graph(&input, &mut changed);
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
        let mut manifest_update = input.update.clone();
        bind_current_graph(&input, &mut manifest_update);
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &manifest_update,
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

    fn bind_current_graph(input: &Inputs, update: &mut Value) {
        update["finalQualificationLock"] = current_binding("pnpm-lock.yaml", &input.lock);
        update["finalQualificationPackage"] = current_binding("package.json", &input.package);
        update["workspaceManifest"] = current_binding("pnpm-workspace.yaml", &input.workspace);
        update["createFrescoManifest"] = current_binding(CREATE_FRESCO_MANIFEST, &input.manifest);
        update["resolver"]["fixedPointSha256"] = json!(support::hash(&input.lock));
    }

    #[test]
    fn exact_create_fresco_successor_binds_the_whole_current_graph() {
        let input = inputs();
        let mut update = input.update.clone();
        bind_current_graph(&input, &mut update);
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &update,
                &input.lock,
                &input.manifest,
            ),
            Ok(())
        );
    }

    #[test]
    fn changed_graph_predecessor_and_fixed_point_are_refused() {
        let input = inputs();
        let mut update = input.update.clone();
        bind_current_graph(&input, &mut update);
        for changed in [
            b"foreign lock".as_slice(),
            input.lock.get(..input.lock.len() - 1).unwrap(),
        ] {
            assert_eq!(
                verify(
                    &input,
                    &input.predecessor,
                    &update,
                    changed,
                    &input.manifest,
                ),
                Err("exact current create-fresco qualification graph bytes")
            );
        }

        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(&input, &predecessor, &update, &input.lock, &input.manifest,),
            Err("exact frozen create-fresco qualification predecessor")
        );

        let mut resolver = update;
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
        bind_current_graph(&input, &mut changed);
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
        let mut manifest_update = input.update.clone();
        bind_current_graph(&input, &mut manifest_update);
        assert_eq!(
            verify(
                &input,
                &input.predecessor,
                &manifest_update,
                &input.lock,
                &manifest,
            ),
            Err("exact current create-fresco qualification graph bytes")
        );
    }
}

mod js_tooling_refresh_successor {
    use super::*;

    struct Inputs {
        previous: Value,
        predecessor: Value,
        predecessor_bytes: Vec<u8>,
        update: Value,
        lock: Vec<u8>,
        package: Vec<u8>,
        workspace: Vec<u8>,
        unplugin_manifest: Vec<u8>,
        example_manifest: Vec<u8>,
    }

    fn inputs() -> Inputs {
        Inputs {
            previous: super::super::super::load(),
            predecessor: docs_metadata_update(),
            predecessor_bytes: fs::read(support::corpus().join(docs_metadata::UPDATE)).unwrap(),
            update: js_tooling_refresh_update(),
            lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
            package: fs::read(support::repository().join("package.json")).unwrap(),
            workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
            unplugin_manifest: fs::read(
                support::repository().join("npm/builder/unplugin/package.json"),
            )
            .unwrap(),
            example_manifest: fs::read(
                support::repository().join("npm/builder/unplugin/example/package.json"),
            )
            .unwrap(),
        }
    }

    fn verify(input: &Inputs, predecessor: &Value, update: &Value, lock: &[u8]) -> Checked<()> {
        let graph = JsToolingRefreshQualificationGraph {
            lock,
            package: &input.package,
            workspace: &input.workspace,
            unplugin_manifest: &input.unplugin_manifest,
            example_manifest: &input.example_manifest,
        };
        verify_js_tooling_refresh_update(predecessor, &input.predecessor_bytes, update, &graph)
    }

    #[test]
    fn exact_refresh_successor_binds_the_whole_current_graph() {
        let input = inputs();
        assert_eq!(
            support::hash(&input.predecessor_bytes),
            docs_metadata::UPDATE_HASH
        );
        assert_eq!(
            verify(&input, &input.predecessor, &input.update, &input.lock),
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
        assert_eq!(
            verify(&input, &input.predecessor, &input.update, b"foreign lock",),
            Err("exact current JavaScript tooling refresh graph bytes")
        );

        let mut predecessor = input.predecessor.clone();
        predecessor["finalQualificationLock"]["sha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(&input, &predecessor, &input.update, &input.lock),
            Err("exact frozen JavaScript tooling refresh predecessor")
        );

        let mut resolver = input.update.clone();
        resolver["resolver"]["fixedPointSha256"] = json!(support::hash(b"foreign"));
        assert_eq!(
            verify(&input, &input.predecessor, &resolver, &input.lock),
            Err("exact JavaScript tooling refresh resolver fixed point")
        );
    }
}
