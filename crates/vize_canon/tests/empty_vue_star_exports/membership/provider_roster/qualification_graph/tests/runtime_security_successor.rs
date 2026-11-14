use super::super::runtime_security::{
    RuntimeSecurityQualificationGraph, update as runtime_security_update,
    verify as verify_runtime_security_update,
};
use super::*;

struct Inputs {
    predecessor: Value,
    predecessor_bytes: Vec<u8>,
    update: Value,
    lock: Vec<u8>,
    package: Vec<u8>,
    workspace: Vec<u8>,
}

fn inputs() -> Inputs {
    Inputs {
        predecessor: node_forge_security::update(),
        predecessor_bytes: fs::read(support::corpus().join(node_forge_security::UPDATE)).unwrap(),
        update: runtime_security_update(),
        lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
        package: fs::read(support::repository().join("package.json")).unwrap(),
        workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
    }
}

fn verify(input: &Inputs, update: &Value, lock: &[u8], workspace: &[u8]) -> Checked<()> {
    let graph = RuntimeSecurityQualificationGraph {
        lock,
        package: &input.package,
        workspace,
    };
    verify_runtime_security_update(&input.predecessor, &input.predecessor_bytes, update, &graph)
}

#[test]
fn exact_runtime_security_successor_reconstructs_the_frozen_graph() {
    let input = inputs();
    assert_eq!(
        support::hash(&input.predecessor_bytes),
        node_forge_security::UPDATE_HASH
    );
    assert_eq!(
        verify(&input, &input.update, &input.lock, &input.workspace),
        Ok(())
    );
    assert_eq!(
        authorize(&super::super::super::load(), &input.lock, &input.package),
        Ok(Some(input.update.clone()))
    );

    let previous_lock = runtime_security::reconstruct_lock_predecessor(&input.lock).unwrap();
    let previous_workspace =
        runtime_security::reconstruct_workspace_predecessor(&input.workspace).unwrap();
    assert_eq!(
        support::hash(&previous_lock),
        input.predecessor["finalQualificationLock"]["sha256"]
            .as_str()
            .unwrap()
    );
    assert_eq!(
        support::hash(&previous_workspace),
        input.predecessor["workspaceManifest"]["sha256"]
            .as_str()
            .unwrap()
    );
}

#[test]
fn changed_runtime_graph_replacements_and_verification_are_refused() {
    let input = inputs();
    assert_eq!(
        verify(&input, &input.update, b"foreign lock", &input.workspace),
        Err("exact current runtime security graph bytes")
    );

    let mut replacement = input.update.clone();
    replacement["lockReplacements"][0]["after"] = json!("foreign successor");
    assert_eq!(
        verify(&input, &replacement, &input.lock, &input.workspace),
        Err("one exact runtime security successor context")
    );

    let mut verification = input.update.clone();
    verification["verification"]["audit"]["moderate"] = json!(1);
    assert_eq!(
        verify(&input, &verification, &input.lock, &input.workspace),
        Err("exact runtime security verification")
    );
}
