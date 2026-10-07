#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use serde_json::json;

#[path = "support/lsp_authored_rename.rs"]
mod authored;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use authored::{capture, edit, expected, location, observe, project};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/lsp/slot-rename/8011/original/",
            $name,
            ".vue.txt"
        ))
    };
}

macro_rules! control {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/lsp/slot-rename/8011/controls/",
            $name,
            ".vue.txt"
        ))
    };
}

#[test]
fn quoted_hyphen_slot_keys_preserve_payload_scope_foreign_owners_and_dynamic_names() {
    let mut observed = Vec::new();
    let mut wanted = Vec::new();
    for origin in ["declaration", "outlet", "consumer"] {
        for newline in ["\n", "\r\n"] {
            let files = [
                ("Card.vue", control!("QuotedCard").replace('\n', newline)),
                (
                    "CardUser.vue",
                    control!("QuotedUser").replace('\n', newline),
                ),
                ("Other.vue", control!("Other").replace('\n', newline)),
            ];
            let goldens = [
                (
                    "Card.vue",
                    control!("QuotedCardRenamed").replace('\n', newline),
                ),
                (
                    "CardUser.vue",
                    control!("QuotedUserRenamed").replace('\n', newline),
                ),
                ("Other.vue", control!("Other").replace('\n', newline)),
            ];
            let (mut fixture, uris) = project(&files);
            let [card_uri, user_uri, _] = &uris;
            let card = files[0].1.as_str();
            let user = files[1].1.as_str();
            let references = json!([
                location(card_uri, card, "item-row\":", 8),
                location(card_uri, card, "item-row\" title", 8),
                location(user_uri, user, "item-row=\"", 8),
            ]);
            let rename = json!({"changes": {
                card_uri: [
                    edit(card, "item-row\":", 8, "heading"),
                    edit(card, "item-row\" title", 8, "heading"),
                ],
                user_uri: [edit(user, "item-row=\"", 8, "heading")],
            }});
            let request = match origin {
                "declaration" => (card_uri.as_str(), card, "tem-row\":", "heading"),
                "outlet" => (card_uri.as_str(), card, "tem-row\" title", "heading"),
                "consumer" => (user_uri.as_str(), user, "tem-row=\"", "heading"),
                _ => unreachable!(),
            };
            let context = format!("authored slot controls {origin} {newline:?}");
            let golden = expected(&goldens, references, rename);
            let actual = observe(&mut fixture, &files, &uris, request, &goldens, &context);
            capture(&fixture, &files, &context, &golden, &actual);
            wanted.push(golden);
            observed.push(actual);
        }
    }
    assert_eq!(json!(observed), json!(wanted));
}

#[test]
fn original_slot_declaration_outlet_and_consumer_share_the_complete_rename() {
    let mut observed = Vec::new();
    let mut wanted = Vec::new();
    for origin in ["declaration", "outlet", "consumer"] {
        for newline in ["\n", "\r\n"] {
            let files = [
                ("Card.vue", source!("Card").replace('\n', newline)),
                ("CardUser.vue", source!("CardUser").replace('\n', newline)),
            ];
            let goldens = [
                ("Card.vue", source!("CardRenamed").replace('\n', newline)),
                (
                    "CardUser.vue",
                    source!("CardUserRenamed").replace('\n', newline),
                ),
            ];
            let (mut fixture, uris) = project(&files);
            let [card_uri, user_uri] = &uris;
            let card = files[0].1.as_str();
            let user = files[1].1.as_str();
            // Whole authored tokens and repaired files are fixed before any query.
            let references = json!([
                location(card_uri, card, "header(props", 6),
                location(card_uri, card, "header\" title", 6),
                location(user_uri, user, "header=\"", 6),
            ]);
            let rename = json!({"changes": {
                card_uri: [
                    edit(card, "header(props", 6, "heading"),
                    edit(card, "header\" title", 6, "heading"),
                ],
                user_uri: [edit(user, "header=\"", 6, "heading")],
            }});
            let request = match origin {
                "declaration" => (card_uri.as_str(), card, "eader(props", "heading"),
                "outlet" => (card_uri.as_str(), card, "eader\" title", "heading"),
                "consumer" => (user_uri.as_str(), user, "eader=\"", "heading"),
                _ => unreachable!(),
            };
            let context = format!("original #8011 slot {origin} {newline:?}");
            let golden = expected(&goldens, references, rename);
            let actual = observe(&mut fixture, &files, &uris, request, &goldens, &context);
            capture(&fixture, &files, &context, &golden, &actual);
            wanted.push(golden);
            observed.push(actual);
        }
    }
    assert_eq!(json!(observed), json!(wanted));
}
