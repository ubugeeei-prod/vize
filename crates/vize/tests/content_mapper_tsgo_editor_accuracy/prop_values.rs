//! Completion and hover inside component prop values keep the child's
//! contract for a declared prop and a fallthrough attr (#12458).

use serde_json::Value;

use super::{Project, error_codes, hover, hover_text, tsgo};
use crate::content_mapper_lsp_support::{completion, position};

fn labels(response: &Value) -> Vec<&str> {
    let mut labels = response
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| response.as_array())
        .unwrap_or_else(|| panic!("{response:#}"))
        .iter()
        .filter_map(|item| item["label"].as_str())
        .collect::<Vec<_>>();
    labels.sort_unstable();
    labels
}

#[test]
fn standard_tsgo_lsp_types_component_prop_values() {
    let Some(tsgo) = tsgo() else {
        return;
    };
    let project = Project::fixture("component-prop-value-editor-12458");
    let (uri, source) = (project.uri("src/App.vue"), project.source("src/App.vue"));
    project.run(&tsgo, async |client| {
        let mode = position(source, source.find("'wide' }").expect("declared value") + 1);
        let modes = completion(client, &uri, &mode).await;
        assert_eq!(
            ["compact", "wide"].map(|label| labels(&modes).contains(&label)),
            [true, true],
            "{modes:#}"
        );

        let attr = position(source, source.find("'text'").expect("attr value") + 1);
        let types = completion(client, &uri, &attr).await;
        assert_eq!(
            ["checkbox", "text"].map(|label| labels(&types).contains(&label)),
            [true, true],
            "{types:#}"
        );

        let binding = position(source, source.find("current }").expect("binding"));
        let binding_hover = hover(client, &uri, &binding).await;
        assert_eq!(
            hover_text(&binding_hover),
            "```typescript\nconst current: \"wide\"\n```",
            "{binding_hover:#}"
        );
        assert_eq!(error_codes(client, &uri).await, Vec::<Value>::new());
    });
}
