use super::super::node_forge_security::{
    NodeForgeSecurityQualificationGraph, update as node_forge_security_update,
    verify as verify_node_forge_security_update,
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
    let current_lock = fs::read(support::repository().join("pnpm-lock.yaml")).unwrap();
    let current_workspace = fs::read(support::repository().join(WORKSPACE)).unwrap();
    Inputs {
        predecessor: eslint_vue_scorecard::update(),
        predecessor_bytes: fs::read(support::corpus().join(eslint_vue_scorecard::UPDATE)).unwrap(),
        update: node_forge_security_update(),
        lock: runtime_security::reconstruct_lock_predecessor(&current_lock).unwrap(),
        package: fs::read(support::repository().join("package.json")).unwrap(),
        workspace: runtime_security::reconstruct_workspace_predecessor(&current_workspace).unwrap(),
    }
}

fn verify(input: &Inputs, update: &Value, lock: &[u8], workspace: &[u8]) -> Checked<()> {
    let graph = NodeForgeSecurityQualificationGraph {
        lock,
        package: &input.package,
        workspace,
    };
    verify_node_forge_security_update(&input.predecessor, &input.predecessor_bytes, update, &graph)
}

#[test]
fn exact_security_successor_reconstructs_both_frozen_predecessors() {
    let input = inputs();
    assert_eq!(
        support::hash(&input.predecessor_bytes),
        eslint_vue_scorecard::UPDATE_HASH
    );
    assert_eq!(
        verify(&input, &input.update, &input.lock, &input.workspace),
        Ok(())
    );
}

#[test]
fn changed_graph_replacements_and_verification_are_refused() {
    let input = inputs();
    assert_eq!(
        verify(&input, &input.update, b"foreign lock", &input.workspace),
        Err("exact current node-forge security graph bytes")
    );

    let mut replacement = input.update.clone();
    replacement["lockReplacements"][0]["after"] = json!("foreign successor");
    assert_eq!(
        verify(&input, &replacement, &input.lock, &input.workspace),
        Err("one exact node-forge successor spelling")
    );

    let mut verification = input.update.clone();
    verification["verification"]["packageSha256"] = json!(support::hash(b"foreign"));
    assert_eq!(
        verify(&input, &verification, &input.lock, &input.workspace),
        Err("exact node-forge security verification")
    );
}
