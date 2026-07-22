//! i18n message preview hints resolved from `<i18n>` blocks and workspace
//! JSON locale catalogs.

use std::fs;

use super::InlayHintService;
use tower_lsp::lsp_types::{InlayHintLabel, Position, Range, Url};

#[test]
fn test_i18n_message_preview_without_script_setup() {
    let content = r#"<template>
  <p>{{ $t("auth.login") }}</p>
</template>
<i18n lang="json">
{
  "en": { "auth": { "login": "Log in" } }
}
</i18n>
"#;

    let uri = Url::parse("file:///test.vue").unwrap();
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    };

    let hints = InlayHintService::get_hints(content, &uri, range);
    let labels: Vec<&str> = hints
        .iter()
        .filter_map(|hint| match &hint.label {
            InlayHintLabel::String(label) => Some(label.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(labels, vec!["= Log in"]);
}

#[test]
fn test_i18n_message_preview_from_workspace_json_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("src/components/LoginButton.vue");
    let locale_path = dir.path().join("src/locales/en.json");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::create_dir_all(locale_path.parent().unwrap()).unwrap();
    fs::write(&locale_path, r#"{ "auth": { "login": "Log in" } }"#).unwrap();

    let content = r#"<template>
  <p>{{ $t("auth.login") }}</p>
</template>
"#;
    fs::write(&source_path, content).unwrap();

    let uri = Url::from_file_path(&source_path).unwrap();
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    };

    let hints = InlayHintService::get_hints(content, &uri, range);
    let labels: Vec<&str> = hints
        .iter()
        .filter_map(|hint| match &hint.label {
            InlayHintLabel::String(label) => Some(label.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(labels, vec!["= Log in"]);
}
