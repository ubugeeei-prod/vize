//! Hover and completion inside component prop values keep reading the
//! child's contract through the owned per-prop form (#12458).

use serde_json::Value;

use super::{Project, error_codes, hover, hover_text, tsgo};
use crate::content_mapper_lsp_support::{completion, position};

#[test]
fn standard_tsgo_lsp_types_component_prop_values() {
    let Some(tsgo) = tsgo() else {
        return;
    };
    let project = Project::fixture("component-prop-value-editor-12458");
    let (uri, source) = (project.uri("src/App.vue"), project.source("src/App.vue"));
    project.run(&tsgo, async |client| {
        // The contextually typed copy answers beside the inference copy.
        let key = position(source, source.find("mode: 'wide'").expect("declared value"));
        let key_hover = hover(client, &uri, &key).await;
        assert_eq!(
            hover_text(&key_hover),
            "(property) mode: string\n\n(property) mode: \"compact\" | \"wide\"",
            "{key_hover:#}"
        );

        let binding = position(source, source.find("current }").expect("binding"));
        let binding_hover = hover(client, &uri, &binding).await;
        assert_eq!(
            hover_text(&binding_hover),
            "const current: \"wide\"",
            "{binding_hover:#}"
        );

        let expression = position(source, source.find("current }").expect("binding") + 3);
        let completions = completion(client, &uri, &expression).await;
        let current = completions["items"]
            .as_array()
            .unwrap_or_else(|| panic!("{completions:#}"))
            .iter()
            .filter(|item| item["label"] == "current")
            .count();
        assert_eq!(current, 1, "{completions:#}");

        assert_eq!(error_codes(client, &uri).await, Vec::<Value>::new());
    });
}
