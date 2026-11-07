//! Template uses preserve setup-binding identity and declaration hover (#11636, #11638).

use serde_json::{Value, json};

use super::{Project, error_codes, hover_text, tsgo};
use crate::content_mapper_lsp_support::{
    contains_location_range, contains_text_edit, definition, hover, position, position_range,
    references, rename,
};

const NAME: &str = "renoteCollapsed";

#[test]
fn standard_tsgo_lsp_finds_template_uses_of_setup_bindings() {
    let Some(tsgo) = tsgo() else {
        return;
    };
    let project = Project::fixture("template-binding-uses-11636");
    let uri = project.uri("src/App.vue");
    let source = project.source("src/App.vue");
    let wrong = project.uri("src/Wrong.vue");
    // A `v-for` alias of the same name owns its own uses.
    let loop_start = source.find("<li v-for").unwrap();
    let loop_end = source.find("</li>").unwrap();
    let (authored, alias): (Vec<_>, Vec<_>) = source
        .match_indices(NAME)
        .map(|(offset, _)| offset)
        .partition(|offset| !(loop_start..loop_end).contains(offset));
    assert_eq!((authored.len(), alias.len()), (7, 3));
    let ranges = |offsets: &[usize]| {
        offsets
            .iter()
            .map(|offset| position_range(source, *offset, NAME.len()))
            .collect::<Vec<_>>()
    };
    let (authored_ranges, alias_ranges) = (ranges(&authored), ranges(&alias));
    let declaration = position(source, authored[0]);
    let template_use = position(source, source.find("v-if=\"renoteCollapsed").unwrap() + 6);
    let items_declaration = position(source, source.find("const items").unwrap() + 6);
    let items_source = position_range(source, source.find("in items").unwrap() + 3, 5);
    project.run(&tsgo, async |client| {
        for (label, at) in [("script", &declaration), ("template", &template_use)] {
            let found = references(client, &uri, at).await;
            assert_locations(&found, &uri, &authored_ranges, label);
            for (start, end) in &alias_ranges {
                assert!(
                    !contains_location_range(&found, &uri, start, end),
                    "{label}: the v-for alias is another binding:\n{found:#}"
                );
            }
        }
        let edit = rename(client, &uri, &template_use, "collapsed").await;
        for (start, end) in &authored_ranges {
            assert!(
                contains_text_edit(&edit, &uri, start, end, "collapsed"),
                "{edit:#}"
            );
        }
        assert_eq!(edit_count(&edit), authored_ranges.len(), "{edit:#}");
        let target = definition(client, &uri, &template_use).await;
        let (start, end) = &authored_ranges[0];
        assert!(
            contains_location_range(&target, &uri, start, end),
            "{target:#}"
        );
        let appear_note = position(
            source,
            source.find("{{ appearNote.text").unwrap() + "{{ ".len(),
        );
        let appear_note_hover = hover(client, &uri, &appear_note).await;
        assert_eq!(
            hover_text(&appear_note_hover),
            "const appearNote: Note",
            "{appear_note_hover:#}"
        );
        let ref_hover = hover(client, &uri, &template_use).await;
        assert_eq!(
            hover_text(&ref_hover),
            "const renoteCollapsed: boolean",
            "{ref_hover:#}"
        );
        // A `v-for` source reads the scope around its loop.
        let items = references(client, &uri, &items_declaration).await;
        assert!(
            contains_location_range(&items, &uri, &items_source.0, &items_source.1),
            "{items:#}"
        );
        // The template still sees the unwrapped value, and its errors stay.
        assert_eq!(error_codes(client, &uri).await, Vec::<Value>::new());
        assert_eq!(error_codes(client, &wrong).await, [json!(2345)]);
    });
}

fn assert_locations(found: &Value, uri: &str, expected: &[(Value, Value)], label: &str) {
    let locations = found.as_array().unwrap_or_else(|| panic!("{found:#}"));
    assert_eq!(locations.len(), expected.len(), "{label}:\n{found:#}");
    for (start, end) in expected {
        assert!(
            contains_location_range(found, uri, start, end),
            "{label}: missing {start} - {end}:\n{found:#}"
        );
    }
}

fn edit_count(edit: &Value) -> usize {
    edit["changes"]
        .as_object()
        .map(|changes| {
            changes
                .values()
                .filter_map(Value::as_array)
                .map(Vec::len)
                .sum()
        })
        .unwrap_or_default()
}
