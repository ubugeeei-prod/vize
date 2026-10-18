use super::*;

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/release/integration-body-448/original-catalog-bodies.json"
);
const SETTLED: &str =
    include_str!("../../../tests/_fixtures/release/integration-body-448/settled-body.txt");

fn policy() -> String {
    generated(
        8793,
        "bd8755ef9df6b3678f08a4153a7cc9b69e315b8d",
        "2817a0aa13518dd46c16bc8bf56ac831f68c24dc",
        "v0.448.0",
    )
}
fn legacy_policy() -> String {
    policy().replacen(QUEUE_POLICY, LEGACY_POLICY, 1)
}

#[test]
fn actual_448_catalog_snapshots_and_settled_bot_formats_keep_the_same_policy() {
    let original: Value = serde_json::from_str(ORIGINAL).unwrap();
    let event = original["eventBody"].as_str().unwrap();
    let current = original["currentBody"].as_str().unwrap();
    assert_eq!(event, legacy_policy());
    assert_ne!(event, policy());
    assert!(
        qualify(event, &policy(), "ubugeeei-prod/vize", 8794)
            .unwrap()
            .is_empty()
    );
    assert!(current.ends_with("<!-- /codesmith:footer -->"));
    assert!(!current.ends_with('\n'));
    assert!(!current.contains("Autofix is disabled."));
    for body in [current.to_owned(), format!("{current}\n")] {
        assert_eq!(
            qualify(&body, &policy(), "ubugeeei-prod/vize", 8794).unwrap(),
            ["codesmith"]
        );
    }
    assert_eq!(
        qualify(SETTLED, &policy(), "ubugeeei-prod/vize", 8794).unwrap(),
        ["codesmith", "coderabbit"]
    );
    for (body, expected) in [
        (current, vec!["codesmith"]),
        (SETTLED, vec!["codesmith", "coderabbit"]),
    ] {
        let suffix = body.strip_prefix(&legacy_policy()).unwrap();
        assert_eq!(
            qualify(
                &format!("{}{suffix}", policy()),
                &policy(),
                "ubugeeei-prod/vize",
                8794,
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn current_bot_formats_still_refuse_foreign_links_forged_pins_and_incomplete_envelopes() {
    for (before, after) in [
        ("/pr/8794?", "/pr/8795?"),
        ("pr_number=8794", "pr_number=8795"),
        ("<!-- codesmith:autofix:disabled -->", "<!-- unknown -->"),
        ("<!-- /codesmith:footer -->", "<!-- missing -->"),
        ("all five full H gates", "only selected gates"),
        ("- **Release**", "- **<script>**"),
        (
            "  - Updated Vize packages",
            "  - <script>Updated Vize packages",
        ),
    ] {
        assert!(SETTLED.contains(before));
        assert!(
            qualify(
                &SETTLED.replace(before, after),
                &policy(),
                "ubugeeei-prod/vize",
                8794,
            )
            .is_err(),
            "{before}"
        );
    }
    assert!(
        qualify(
            &format!("{SETTLED}\n\nunknown suffix"),
            &policy(),
            "ubugeeei-prod/vize",
            8794,
        )
        .is_err()
    );
}
