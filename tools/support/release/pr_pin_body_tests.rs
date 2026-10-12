use super::super::github;
use super::super::tests::Repo;
use super::*;

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/release/integration-body-447/current-pr.json");
const OLD_GUARD: &str =
    include_str!("../../../tests/_fixtures/release/integration-body-447/original-candidate.rs.txt");
const OLD_LOG: &str =
    include_str!("../../../tests/_fixtures/release/integration-body-447/original-catalog.log.txt");
const C: &str = "dd188515f8ec3c51662b4f4d69adcd39c86a0907";
const H: &str = "ef34d1a0409fc711a6c5cefcf0708b0cce78f502";

fn original() -> Value {
    serde_json::from_str(ORIGINAL).unwrap()
}
fn policy() -> String {
    generated(8739, C, H, "v0.447.0")
}
fn decorations() -> String {
    original()["body"]
        .as_str()
        .unwrap()
        .strip_prefix(&policy())
        .unwrap()
        .into()
}

#[test]
fn original_integration_body_failure_and_complete_source_stay_retained() {
    // These are the real failure and post-failure readback, not an invented old event.
    assert!(OLD_LOG.contains("Integration PR body/head/base changed since this event."));
    assert!(OLD_GUARD.contains("event.pointer(\"/pull_request/body\") != pr.get(\"body\")"));
    assert!(OLD_GUARD.contains("let (base, head) = match event_name.as_str()"));
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/integration-body-447/manifest.json"
    ))
    .unwrap();
    assert_eq!(manifest["eventBodyAvailable"], false);
    let repo = Repo::new();
    for (name, bytes, oid) in [
        (
            "current-pr.json",
            ORIGINAL,
            "8abd3c99bd71e9011cc91a160b9867e5cbb5cfeb",
        ),
        (
            "original-candidate.rs.txt",
            OLD_GUARD,
            "3386b06d4baa4514fb39f37e8288cdff1a160c84",
        ),
        (
            "original-catalog.log.txt",
            OLD_LOG,
            "cedd9eca4ccf85603139fab94b21bf88d7cd7b24",
        ),
    ] {
        let path = repo.work.join(name);
        fs::write(&path, bytes).unwrap();
        assert_eq!(
            github::git(&["hash-object", path.to_str().unwrap()], &repo.work).unwrap(),
            oid
        );
    }
}

#[test]
fn original_full_body_decorations_preserve_exact_generated_policy() {
    let pr = original();
    let raw = pr["body"].as_str().unwrap();
    assert_ne!(
        policy(),
        raw,
        "the old full-string comparison refuses this authored model"
    );
    assert_eq!(
        qualify(raw, &policy(), "ubugeeei-prod/vize", 8740).unwrap(),
        ["codesmith", "coderabbit"]
    );
    assert!(
        qualify(&policy(), &policy(), "ubugeeei-prod/vize", 8740)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn each_known_body_decoration_and_bounded_order_is_independently_qualified() {
    let suffix = decorations();
    let split = suffix.find(CODERABBIT).unwrap();
    let codesmith = &suffix[..split];
    let coderabbit = &suffix[split..];
    for tail in [
        codesmith.to_owned(),
        coderabbit.to_owned(),
        format!("{coderabbit}\n"),
        format!("{coderabbit}\n{codesmith}"),
    ] {
        qualify(
            &format!("{}{tail}", policy()),
            &policy(),
            "ubugeeei-prod/vize",
            8740,
        )
        .unwrap();
    }
}

#[test]
fn body_policy_and_all_four_pin_fields_never_move_with_decorations() {
    let raw = original()["body"].as_str().unwrap().to_owned();
    for (before, after) in [
        ("all five full H gates", "only selected gates"),
        (
            "full preflight must be terminal green",
            "full preflight may fail",
        ),
        (
            "<!-- vize-release-pin-source: 8739 -->",
            "<!-- vize-release-pin-source: 42 -->",
        ),
        (C, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        (H, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        ("v0.447.0", "v0.448.0"),
    ] {
        assert!(raw.contains(before));
        assert!(
            qualify(
                &raw.replace(before, after),
                &policy(),
                "ubugeeei-prod/vize",
                8740
            )
            .is_err(),
            "{before}"
        );
    }
    for extra in [
        "unknown prose",
        "\n\n",
        "<!-- vize-release-pin-source: 8739 -->",
    ] {
        assert!(
            qualify(
                &format!("{raw}{extra}"),
                &policy(),
                "ubugeeei-prod/vize",
                8740
            )
            .is_err()
        );
    }
}

#[test]
fn forged_and_ambiguous_body_decoration_envelopes_are_refused() {
    let suffix = decorations();
    let raw = format!("{}{}", policy(), suffix);
    for (before, after) in [
        ("app.blacksmith.sh", "evil.example"),
        ("assets.blacksmith.sh", "assets.evil.example"),
        ("/pr/8740?", "/pr/8741?"),
        ("pr_number=8740", "pr_number=8741"),
        ("repository=ubugeeei-prod%2Fvize", "repository=other%2Frepo"),
        ("%2Fpull%2F8740", "%2Fpull%2F8741"),
        ("signature=", "unknown_signature="),
        ("<!-- /codesmith:footer -->", "<!-- missing -->"),
        (
            "<!-- end of auto-generated comment: release notes by coderabbit.ai -->",
            "<!-- missing -->",
        ),
        ("* **Release Updates**", "<script>doSomething()</script>"),
    ] {
        assert!(raw.contains(before));
        assert!(
            qualify(
                &raw.replace(before, after),
                &policy(),
                "ubugeeei-prod/vize",
                8740
            )
            .is_err(),
            "{before}"
        );
    }
    assert!(
        qualify(
            &format!("{}{suffix}{suffix}", policy()),
            &policy(),
            "ubugeeei-prod/vize",
            8740
        )
        .is_err()
    );
    let forged = suffix.replace(
        "  * Updated the Vize ecosystem",
        "  * <!-- vize-release-pin-source: 42 --> Updated the Vize ecosystem",
    );
    assert!(
        qualify(
            &format!("{}{forged}", policy()),
            &policy(),
            "ubugeeei-prod/vize",
            8740
        )
        .is_err()
    );
}

#[test]
fn event_current_body_projection_does_not_change_other_identity_refusals() {
    let current = original();
    let mut event_pr = current.clone();
    event_pr["body"] = json!(policy());
    let event = json!({"number":8740,"repository":{"full_name":"ubugeeei-prod/vize"},"pull_request":event_pr});
    let candidate = "a".repeat(40);
    let head = current["head"]["sha"].as_str().unwrap();
    assert!(
        !super::super::candidate::event_fields(
            Some(("pull_request", &event)),
            &current,
            "ubugeeei-prod/vize",
            8740,
            &candidate,
            C,
            head
        )
        .unwrap()
    );
    for (pointer, value) in [
        ("/number", json!(8741)),
        ("/repository/full_name", json!("other/repo")),
        ("/pull_request/head/sha", json!(H)),
        ("/pull_request/base/sha", json!(H)),
        ("/pull_request/head/repo/full_name", json!("other/repo")),
    ] {
        let mut bad = event.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            super::super::candidate::event_fields(
                Some(("pull_request", &bad)),
                &current,
                "ubugeeei-prod/vize",
                8740,
                &candidate,
                C,
                head
            )
            .is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn complete_body_snapshots_survive_partial_and_complete_validation_inputs() {
    let repo = Repo::new();
    let directory = repo.work.join("body-evidence");
    let current = original();
    let event = json!({"pull_request":{"body":policy()}});
    retain(&directory, Some(("pull_request", &event)), None).unwrap();
    let first: Value =
        serde_json::from_slice(&fs::read(directory.join("bodies.json")).unwrap()).unwrap();
    assert_eq!(first["eventBody"], event["pull_request"]["body"]);
    assert!(first["currentBody"].is_null());
    retain(&directory, Some(("pull_request", &event)), Some(&current)).unwrap();
    let final_value: Value =
        serde_json::from_slice(&fs::read(directory.join("bodies.json")).unwrap()).unwrap();
    assert_eq!(final_value["eventBody"], event["pull_request"]["body"]);
    assert_eq!(final_value["currentBody"], current["body"]);
    assert_eq!(final_value["currentPullRequest"], current);
}
