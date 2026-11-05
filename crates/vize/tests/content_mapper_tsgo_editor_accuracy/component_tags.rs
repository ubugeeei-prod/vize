//! A component tag names the imported SFC itself (#11635, #11637).

use serde_json::json;

use super::{Project, assert_definition_at, error_codes, hover, hover_text, tsgo};
use crate::content_mapper_lsp_support::{anchored_position, definition};

#[test]
fn standard_tsgo_lsp_answers_component_tags_with_the_sfc() {
    let Some(tsgo) = tsgo() else {
        return;
    };
    let project = Project::fixture("component-tag-sfc-export-11635");
    let app = (project.uri("src/App.vue"), project.source("src/App.vue"));
    let header = project.uri("src/MkNoteHeader.vue");
    let wrong = project.uri("src/Wrong.vue");
    project.run(&tsgo, async |client| {
        let sfc_start = json!({ "line": 0, "character": 0 });
        assert_definition_at(
            client,
            (&app.0, app.1),
            "MkNoteHeader :note",
            &header,
            (sfc_start.clone(), sfc_start),
        )
        .await;

        let tag = anchored_position(app.1, "MkNoteHeader :note", "component tag");
        let tag_hover = hover(client, &app.0, &tag).await;
        assert_eq!(
            hover_text(&tag_hover),
            "(alias) const MkNoteHeader: MkNoteHeaderComponent",
            "{tag_hover:#}"
        );

        // A native element is no component: it names no SFC.
        let native = anchored_position(app.1, "div>", "native element");
        let native_definition = definition(client, &app.0, &native).await;
        let into_header = native_definition
            .as_array()
            .into_iter()
            .flatten()
            .filter(|location| location["uri"] == header.as_str())
            .count();
        assert_eq!(into_header, 0, "{native_definition:#}");
        // The script import still resolves, and a wrong prop type still errors.
        assert_eq!(
            error_codes(client, &app.0).await,
            Vec::<serde_json::Value>::new()
        );
        assert_eq!(error_codes(client, &wrong).await, [json!(2322)]);
    });
}
