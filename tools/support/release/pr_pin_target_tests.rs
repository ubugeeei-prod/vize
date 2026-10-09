use super::super::pr_github as github;
use super::catalog::publication_targets;
use super::tests::Repo;
use serde_json::{Value, json};

#[test]
fn target_projection_preserves_all_shipping_fields_and_unknown_kinds() {
    let shipping = json!([
        {"kind":["lib"],"crate_types":["lib"],"name":"law","test":true,"src_path":"src/lib.rs"},
        {"kind":["bin"],"crate_types":["bin"],"name":"cli","test":true},
        {"kind":["proc-macro"],"crate_types":["proc-macro"],"name":"derive"},
        {"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build"}
    ]);
    let shipping = shipping.as_array().unwrap();
    assert_eq!(publication_targets(shipping), *shipping);
    for kinds in [
        json!(["test"]),
        json!(["bench"]),
        json!(["example"]),
        json!(["test", "example"]),
    ] {
        let mut targets = shipping.clone();
        targets.push(json!({"kind":kinds,"crate_types":["bin"],"name":"observer"}));
        assert_eq!(publication_targets(&targets), *shipping);
    }
    for kinds in [
        json!(["test", "bin"]),
        json!(["example", "custom-build"]),
        json!(["future"]),
        json!(["test", "future"]),
        json!([]),
        json!(null),
        json!("test"),
        json!([null]),
    ] {
        let mut targets = shipping.clone();
        targets.push(json!({"kind":kinds,"crate_types":["bin"],"name":"unqualified"}));
        assert_eq!(publication_targets(&targets), targets);
        assert_ne!(publication_targets(&targets), *shipping);
    }
    for (field, value) in [
        ("name", json!("other")),
        ("src_path", json!("src/other.rs")),
        ("crate_types", json!(["cdylib"])),
        ("required-features", json!(["new"])),
    ] {
        let mut targets = shipping.clone();
        targets[0][field] = value;
        assert_eq!(publication_targets(&targets), targets);
        assert_ne!(publication_targets(&targets), *shipping, "{field}");
    }
}

#[test]
fn actual_h_v_eight_discovered_tests_preserve_the_complete_shipping_targets() {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/pinned-catalog-dev-targets.json"
    ))
    .unwrap();
    assert_eq!(packet["H"], "fd6241bf8ea5466794cc955138a59a9b75a8aac2");
    assert_eq!(packet["V"], "18ce7841e693550334c81584bae2f9782d27f786");
    let crates = packet["crates"].as_array().unwrap();
    assert_eq!(crates.len(), 5);
    let mut added = Vec::new();
    for package in crates {
        let before = package["H"].as_array().unwrap();
        let after = package["V"].as_array().unwrap();
        assert!(before.iter().all(|target| after.contains(target)));
        assert_eq!(
            publication_targets(before),
            publication_targets(after),
            "{}",
            package["crate"]
        );
        assert!(!publication_targets(before).is_empty());
        for target in after.iter().filter(|target| !before.contains(target)) {
            assert_eq!(target["kind"], json!(["test"]));
            added.push(target["name"].as_str().unwrap());
            for kind in ["bin", "custom-build", "proc-macro", "future"] {
                let mut changed = after.clone();
                let index = changed.iter().position(|value| value == target).unwrap();
                changed[index]["kind"] = json!(["test", kind]);
                assert_ne!(
                    publication_targets(before),
                    publication_targets(&changed),
                    "{kind}"
                );
            }
        }
    }
    added.sort_unstable();
    assert_eq!(
        added,
        [
            "browser_ssr_context",
            "css_global_ownership",
            "custom_directive_identity",
            "keyed_fragment",
            "scoped_slotted_where",
            "slot_key_mappings",
            "vapor_component_refs",
            "vapor_keyed_fragment",
        ]
    );
}

#[test]
fn normal_cached_driver_module_changes_require_new_entry_source_pin() {
    let pin = include_str!("../../commands/release/pr.rs")
        .lines()
        .find(|line| line.starts_with("//! Driver source pin:"))
        .unwrap();
    assert_eq!(pin, "//! Driver source pin: unpublished-retirement-v10.");
    let repo = Repo::new();
    let body = "#[path = \"dependency.rs\"] mod dependency;\nfn main() { println!(\"{}\", dependency::catalog_contract()); }\n";
    let original = format!("//! Driver source pin: original-catalog.\n{body}");
    repo.commit(&[
        ("driver.rs", &original),
        ("dependency.rs", "pub fn catalog_contract() -> u8 { 1 }\n"),
    ]);
    let driver = repo.work.join("driver.rs");
    let args = [driver.to_str().unwrap()];
    assert_eq!(
        github::output("rust-script", &args, &repo.work).unwrap(),
        "1"
    );
    repo.commit(&[("dependency.rs", "pub fn catalog_contract() -> u8 { 2 }\n")]);
    assert_eq!(
        github::output("rust-script", &args, &repo.work).unwrap(),
        "1"
    );
    repo.commit(&[("driver.rs", &format!("{pin}\n{body}"))]);
    assert_eq!(
        github::output("rust-script", &args, &repo.work).unwrap(),
        "2"
    );
}
