use super::super::eslint_vue_scorecard::{
    EslintVueScorecardQualificationGraph, update as eslint_vue_scorecard_update,
    verify as verify_eslint_vue_scorecard_update,
};
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
        predecessor: js_tooling_refresh_update(),
        predecessor_bytes: fs::read(support::corpus().join(js_tooling_refresh::UPDATE)).unwrap(),
        update: eslint_vue_scorecard_update(),
        lock: fs::read(support::repository().join("pnpm-lock.yaml")).unwrap(),
        package: fs::read(support::repository().join("package.json")).unwrap(),
        workspace: fs::read(support::repository().join(WORKSPACE)).unwrap(),
        manifest: fs::read(support::repository().join(eslint_vue_scorecard::MANIFEST)).unwrap(),
    }
}

fn verify(input: &Inputs, update: &Value, lock: &[u8]) -> Checked<()> {
    let graph = EslintVueScorecardQualificationGraph {
        lock,
        package: &input.package,
        workspace: &input.workspace,
        manifest: &input.manifest,
    };
    verify_eslint_vue_scorecard_update(&input.predecessor, &input.predecessor_bytes, update, &graph)
}

#[test]
fn exact_scorecard_successor_reconstructs_the_frozen_lock_predecessor() {
    let input = inputs();
    assert_eq!(
        support::hash(&input.predecessor_bytes),
        js_tooling_refresh::UPDATE_HASH
    );
    assert_eq!(verify(&input, &input.update, &input.lock), Ok(()));
}

#[test]
fn changed_scorecard_graph_and_insertions_are_refused() {
    let input = inputs();
    assert_eq!(
        verify(&input, &input.update, b"foreign lock"),
        Err("exact current scorecard qualification graph bytes")
    );

    let mut insertion = input.update.clone();
    insertion["lockInsertions"][0]["addedUtf8"] = json!("foreign insertion");
    assert_eq!(
        verify(&input, &insertion, &input.lock),
        Err("complete scorecard lock insertion")
    );
}
