use super::{ELEMENTS, lookup, lookup_cold};

const REFERENCES: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/lsp/html-element-hover-documentation/references.expected.json"
);

#[test]
fn complete_html_catalog_matches_the_existing_dom_map_and_verified_reference_links() {
    let mut admitted: Vec<_> = vize_l0::HTML_TAGS
        .iter()
        .filter(|tag| {
            crate::ide::corsa_support::native_dom_tag_info(tag)
                .is_some_and(|info| info.category == "HTML element")
        })
        .copied()
        .collect();
    admitted.sort_unstable();
    assert_eq!(ELEMENTS.len(), 110);
    assert_eq!(
        ELEMENTS.iter().map(|entry| entry.tag).collect::<Vec<_>>(),
        admitted
    );
    let references: serde_json::Value = serde_json::from_str(REFERENCES).unwrap();
    let references = references["references"].as_array().unwrap();
    assert_eq!(references.len(), ELEMENTS.len());
    for entry in ELEMENTS {
        let expected = references
            .iter()
            .find(|row| row["tag"] == entry.tag)
            .unwrap();
        assert_eq!(entry.mdn_url, expected["mdn"].as_str().unwrap());
        assert_eq!(entry.standard_url, expected["spec"].as_str().unwrap());
        assert!(!entry.description.is_empty());
        assert!(!entry.description.contains("Native DOM element"));
        assert!(std::ptr::eq(lookup(entry.tag).unwrap(), entry));
        // Common-tag direct indices and the cold path resolve the same borrowed row.
        assert!(std::ptr::eq(lookup_cold(entry.tag).unwrap(), entry));
    }
}

#[test]
fn unknown_components_vue_builtins_and_other_namespaces_have_no_html_descriptor() {
    for tag in [
        "",
        "Button",
        "my-widget",
        "template",
        "param",
        "slot",
        "teleport",
        "suspense",
        "component",
        "circle",
        "svg",
        "math",
        "mi",
        "zzzz",
    ] {
        assert!(lookup(tag).is_none(), "unexpected descriptor: {tag}");
        assert!(
            lookup_cold(tag).is_none(),
            "unexpected cold descriptor: {tag}"
        );
    }
}
