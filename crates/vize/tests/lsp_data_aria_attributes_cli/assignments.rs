//! Exact offered edits preserve complete authored assignments in both modes.
#![expect(
    clippy::string_slice,
    reason = "checked source spans and edit application"
)]

use serde_json::{Value, json};

use super::{
    ARIA, DATA,
    controls::{changed, edited_bank},
    controls_project::Project,
    position, with_edit,
};

const DECLARED: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/Declared.vue.txt"
);
const DEFAULTS: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/defaults.expected.json"
);

#[test]
fn assigned_names_preserve_suffixes_values_binding_utf16_disk_and_restore() {
    let data: Value = serde_json::from_str(DATA).unwrap();
    let aria: Vec<Value> = serde_json::from_str(ARIA).unwrap();
    let defaults: Value = serde_json::from_str(DEFAULTS).unwrap();
    for native in [false, true] {
        let original = changed("div", "data-role=\"x\"");
        let Some(mut project) = Project::new(&original, DECLARED, native) else {
            return;
        };
        for tag in ["div", "Unknown"] {
            for binding in ["", ":", "v-bind:"] {
                for assignment in ["=\"x\"", " = \"x\""] {
                    for (name, prefix, item) in [
                        ("data-role", "dat", data["data-role"].clone()),
                        (
                            "aria-hidden",
                            "aria-hi",
                            aria.iter()
                                .find(|item| item["label"] == "aria-hidden")
                                .unwrap()
                                .clone(),
                        ),
                    ] {
                        let token = format!("{binding}{name}");
                        let source = changed(tag, &format!("{token}{assignment}"));
                        project.change(&source);
                        for length in [prefix.len(), name.len()] {
                            let cursor = &source[source.find(name).unwrap() + length..];
                            let typed = &name[..length];
                            let expected = if binding.is_empty() {
                                json!([with_edit(
                                    item.clone(),
                                    position(&source, &token),
                                    position(&source, cursor),
                                    typed
                                )])
                            } else {
                                // Preserve the original complete bind bank. Only
                                // this contextual name appends one exact candidate.
                                let mut bank =
                                    defaults["native"]["bound"].as_array().unwrap().clone();
                                let mut contextual = item.clone();
                                contextual["insertText"] = json!(typed);
                                bank.push(contextual);
                                edited_bank(&source, &token, cursor, bank)
                            };
                            let actual = project.assert_response(&source, cursor, expected);
                            apply_identity(&source, actual.as_array().unwrap().last().unwrap());
                        }
                        project.assert_disk(&original, DECLARED);
                    }
                }
            }
        }
        for tag in ["div", "Unknown"] {
            let source = changed(tag, "aria-label=\"wide\"");
            project.change(&source);
            let label = defaults["native"]["bare"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["label"] == "aria-label")
                .unwrap()
                .clone();
            // The existing common item owns this complete legacy payload. This
            // extension must not replace it with new contextual documentation.
            let expected = with_edit(
                label,
                position(&source, "aria-label"),
                position(&source, "label=\"wide\""),
                "aria-label=\"$1\"",
            );
            project.assert_response(&source, "label=\"wide\"", json!([expected]));
            project.assert_disk(&original, DECLARED);
        }
        project.change(&original);
        let actual = with_edit(
            data["data-role"].clone(),
            position(&original, "data-role"),
            position(&original, "a-role=\"x\""),
            "dat",
        );
        project.assert_response(&original, "a-role=\"x\"", json!([actual]));
        project.assert_disk(&original, DECLARED);
        project.shutdown();
    }
}

fn apply_identity(source: &str, item: &Value) {
    let edit: lsp_types::CompletionItem = serde_json::from_value(item.clone()).unwrap();
    let Some(lsp_types::CompletionTextEdit::Edit(edit)) = edit.text_edit else {
        panic!("one authored TextEdit");
    };
    let start = byte_offset(source, edit.range.start);
    let end = byte_offset(source, edit.range.end);
    let mut applied = source.to_owned();
    applied.replace_range(start..end, &edit.new_text);
    assert_eq!(applied, source, "suffix and assignment preserved");
}

fn byte_offset(source: &str, position: lsp_types::Position) -> usize {
    let start = source
        .match_indices('\n')
        .take(position.line as usize)
        .last()
        .map_or(0, |(index, _)| index + 1);
    let mut utf16 = 0;
    for (offset, ch) in source[start..].char_indices() {
        if utf16 == position.character {
            return start + offset;
        }
        utf16 += ch.len_utf16() as u32;
    }
    assert_eq!(utf16, position.character);
    source.len()
}
