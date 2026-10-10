use std::fs;

use serde_json::json;

use super::{Case, assert_operation, custody, packets, profile, request, run_with};

#[test]
#[cfg(unix)]
fn retained_html_and_root_plan_are_used_once_after_owned_disk_mutation() {
    let case: Case = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lint/oxlint-original-html-operation-7903/mutation-case.json"
    )))
    .unwrap();
    for host in [
        profile::HostProfile::Oxlint178,
        profile::HostProfile::Oxlint186,
    ] {
        let mut owned = custody::Owned::setup(&case, host);
        let before = custody::snapshot(owned.temporary.path());
        let mut calls = Vec::new();
        let actual = run_with(
            request(&case, &owned.root, &owned.config, host),
            case.root.as_bytes(),
            |linter, source, filename| {
                calls.push(json!({"filename":filename,"source_bytes":source.as_bytes()}));
                if calls.len() == 1 {
                    // Deliberate mutation is confined to this owned fixture.
                    // It witnesses retained data, rather than claiming ordinary
                    // concurrent-filesystem admission or immutable external state.
                    fs::write(&owned.config, b"{invalid-after-original-decode\n").unwrap();
                    custody::write(
                        &owned.root.join("a/Live.html"),
                        b"<p>Replaced first input</p>\n",
                    );
                    custody::write(
                        &owned.root.join("b/Live.html"),
                        b"<p>Replaced later input</p>\n",
                    );
                }
                // The genuine existing HTML engine consumes the exact callback
                // source. No fake diagnostic or provider replacement is used.
                linter.lint_standalone_html(source, filename)
            },
        );
        let mut observation = packets::observation(&actual);
        observation["actual_callbacks"] = json!(calls);
        observation["owned_mutations"] = json!([
            {"path":owned.config,"bytes":b"{invalid-after-original-decode\n"},
            {"path":owned.root.join("a/Live.html"),"bytes":b"<p>Replaced first input</p>\n"},
            {"path":owned.root.join("b/Live.html"),"bytes":b"<p>Replaced later input</p>\n"},
        ]);
        owned.observe(&before, &observation);
        let after = custody::snapshot(owned.temporary.path());
        owned.receipt(&after);
        let actual = actual.unwrap();
        assert_operation(&case, host, &owned.root, &actual);
        assert_eq!(
            calls,
            vec![
                json!({"filename":owned.root.join("a/Live.html"),"source_bytes":custody::source_bytes("@Warning.html")}),
                json!({"filename":owned.root.join("b/Live.html"),"source_bytes":custody::source_bytes("@Warning.html")}),
            ],
            "whole ordered actual call packets and once-per-original execution"
        );
        let mut expected_after = before;
        for (path, bytes) in [
            (
                "repo/.oxlintrc.json",
                b"{invalid-after-original-decode\n".as_slice(),
            ),
            (
                "repo/a/Live.html",
                b"<p>Replaced first input</p>\n".as_slice(),
            ),
            (
                "repo/b/Live.html",
                b"<p>Replaced later input</p>\n".as_slice(),
            ),
        ] {
            expected_after.insert(path.into(), custody::Entry::File(bytes.to_vec()));
        }
        assert_eq!(
            after, expected_after,
            "only the three authored owned mutations occurred"
        );
    }
}
