use super::*;
use serde_json::json;

#[test]
fn retained_stable_authority_revalidates_without_network_and_refuses_missing_forged_or_extra_evidence()
 {
    let (repo, v, c, api) = super::tests::fixture();
    let cut = field(&v, "/cut").unwrap();
    let body = format!(
        "<!-- vize-release-bump: patch -->\n{}",
        super::super::markers(Some(&c))
    );
    let mut source = super::super::super::tests::source(cut, cut);
    source.stable = Some(c.clone());
    let evidence = super::super::archive::capture_with(&source, &c.policy, &repo.work, &|route| {
        api.get(route)
            .cloned()
            .ok_or("Unknown capture route".into())
    })
    .unwrap()
    .unwrap();
    assert_eq!(
        evidence,
        json!({"policy":c.policy,"blob":c.blob,"sha256":c.sha256,"responses":api})
    );
    // Remove the remote entirely: no fetch/API can supply the retained offline proof.
    github::git(&["remote", "remove", "origin"], &repo.work).unwrap();
    let check = |body: &str, evidence: Option<&Value>, main: &str| {
        super::super::from_archive(
            body,
            cut,
            "v0.443.1",
            "owner/repo",
            main,
            evidence,
            &repo.work,
        )
    };
    assert_eq!(
        check(&body, Some(&evidence), &c.policy)
            .unwrap()
            .unwrap()
            .blob,
        c.blob
    );
    assert!(check(&body, None, &c.policy).is_err());
    assert!(check("ordinary source", Some(&evidence), &c.policy).is_err());
    assert!(check(&body, Some(&evidence), cut).is_err());
    for (pointer, value) in [
        ("/policy", json!(cut)),
        ("/blob", json!("a".repeat(40))),
        ("/sha256", json!("a".repeat(64))),
        ("/responses/actions~1runs~17/conclusion", json!("failure")),
        ("/responses/pulls~199/merged", json!(false)),
    ] {
        let mut bad = evidence.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(check(&body, Some(&bad), &c.policy).is_err(), "{pointer}");
    }
    let mut extra = evidence.clone();
    extra["responses"]["unknown/route"] = json!({});
    assert!(check(&body, Some(&extra), &c.policy).is_err());
}

#[test]
fn stable_failed_operator_is_bound_to_policy_main_not_detached_shipping_cut() {
    let mut source = super::super::super::tests::source(&"a".repeat(40), &"b".repeat(40));
    let policy = "c".repeat(40);
    source.stable = Some(Stable {
        policy: policy.clone(),
        blob: "d".repeat(40),
        sha256: "e".repeat(64),
    });
    let run = json!({"id":7,"run_attempt":1,"status":"completed","conclusion":"failure","head_sha":policy,"head_branch":"main","event":"workflow_dispatch","path":".github/workflows/release-operator.yml","head_repository":{"full_name":"owner/repo"},"actor":{"login":"maintainer"},"triggering_actor":{"login":"maintainer"}});
    super::super::super::retire_evidence::original_operator(&run, &source, 7).unwrap();
    let mut bad = run;
    bad["head_sha"] = json!(source.cut);
    assert!(super::super::super::retire_evidence::original_operator(&bad, &source, 7).is_err());
}
