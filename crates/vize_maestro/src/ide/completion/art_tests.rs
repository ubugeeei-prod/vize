//! Completion inside `*.art.vue` files and inline `<art>` blocks, routed
//! through `CompletionService::complete` by cursor position.

use std::fs;

use tower_lsp::lsp_types::{CompletionItem, CompletionResponse, InsertTextFormat, Url};

use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;

const ART_FILE: &str = r#"<script setup lang="ts">
import Button from './Button.vue'
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Primary</Button>
  </variant>

</art>
"#;

fn complete_at(name: &str, source: &str, offset: usize) -> Vec<CompletionItem> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    fs::write(&path, source).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    let language = if name.ends_with(".art.vue") {
        "art-vue"
    } else {
        "vue"
    };
    state
        .documents
        .open(uri.clone(), source.to_owned(), 1, language.to_owned());
    state.update_virtual_docs(&uri, source);
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    match CompletionService::complete(&ctx) {
        Some(CompletionResponse::Array(items)) => items,
        Some(CompletionResponse::List(list)) => list.items,
        None => Vec::new(),
    }
}

fn labels(items: &[CompletionItem]) -> Vec<&str> {
    items.iter().map(|item| item.label.as_str()).collect()
}

fn item<'a>(items: &'a [CompletionItem], label: &str) -> &'a CompletionItem {
    items
        .iter()
        .find(|item| item.label == label)
        .unwrap_or_else(|| panic!("missing {label:?} in {:?}", labels(items)))
}

#[test]
fn art_opening_tag_offers_art_metadata_attributes() {
    let source = ART_FILE.replace(
        r#"component="./Button.vue">"#,
        r#"component="./Button.vue" >"#,
    );
    let offset = source.find(r#"Button.vue" >"#).unwrap() + r#"Button.vue" "#.len();
    let items = complete_at("Button.art.vue", &source, offset);
    let labels = labels(&items);

    for expected in [
        "title",
        "component",
        "description",
        "category",
        "tags",
        "status",
        "order",
    ] {
        assert!(labels.contains(&expected), "missing {expected}: {labels:?}");
    }
    assert!(
        !labels.contains(&"default"),
        "variant attribute leaked: {labels:?}"
    );
    let status = item(&items, "status");
    assert_eq!(status.insert_text.as_deref(), Some("status=\"$1\""));
    assert_eq!(status.insert_text_format, Some(InsertTextFormat::SNIPPET));
}

#[test]
fn variant_opening_tag_offers_variant_attributes() {
    let source = ART_FILE.replace(
        "<variant name=\"Primary\" default>",
        "<variant name=\"Primary\" >",
    );
    let offset = source.find("\"Primary\" >").unwrap() + "\"Primary\" ".len();
    let items = complete_at("Button.art.vue", &source, offset);
    let labels = labels(&items);

    for expected in ["name", "default", "args", "viewport", "skip-vrt"] {
        assert!(labels.contains(&expected), "missing {expected}: {labels:?}");
    }
    assert!(
        !labels.contains(&"title"),
        "art attribute leaked: {labels:?}"
    );
    assert_eq!(
        item(&items, "args").insert_text.as_deref(),
        Some("args='{\"$1\": $2}'")
    );
    assert_eq!(
        item(&items, "default").insert_text.as_deref(),
        Some("default")
    );
}

#[test]
fn between_variants_offers_variant_snippets_but_no_second_art_block() {
    let offset = ART_FILE.find("\n</art>").unwrap();
    let items = complete_at("Button.art.vue", ART_FILE, offset);
    let labels = labels(&items);

    assert!(labels.contains(&"variant"), "{labels:?}");
    assert!(labels.contains(&"variant with args"), "{labels:?}");
    assert!(!labels.contains(&"art"), "{labels:?}");
    assert_eq!(
        item(&items, "variant").insert_text.as_deref(),
        Some("<variant name=\"$1\">\n\t$0\n</variant>")
    );
}

#[test]
fn an_empty_art_file_offers_the_art_block_and_script_snippets() {
    let items = complete_at("Card.art.vue", "", 0);
    let labels = labels(&items);

    assert!(labels.contains(&"art"), "{labels:?}");
    assert!(labels.contains(&"script setup"), "{labels:?}");
    assert!(labels.contains(&"style"), "{labels:?}");
    assert!(!labels.contains(&"variant"), "{labels:?}");
    let art = item(&items, "art");
    assert_eq!(art.insert_text_format, Some(InsertTextFormat::SNIPPET));
    assert_eq!(
        art.insert_text.as_deref(),
        Some("<art>\n\t<variant name=\"$1\" default>\n\t\t$0\n\t</variant>\n</art>")
    );
}

#[test]
fn after_the_closing_art_tag_offers_only_block_snippets() {
    let items = complete_at("Button.art.vue", ART_FILE, ART_FILE.len());
    let labels = labels(&items);

    assert!(labels.contains(&"script setup"), "{labels:?}");
    assert!(
        !labels.contains(&"art"),
        "a second <art> is not valid: {labels:?}"
    );
    assert!(!labels.contains(&"variant"), "{labels:?}");
    assert!(!labels.contains(&"title"), "{labels:?}");
}

const INLINE_ART: &str = r#"<script setup lang="ts">
defineProps<{ label: string }>()
</script>

<template>
  <button>{{ label }}</button>
</template>

<art title="Self button">
  <variant name="Default" default>
    <Self label="Hi" />
  </variant>

</art>
"#;

#[test]
fn inline_art_between_variants_offers_variant_and_self_snippets() {
    let offset = INLINE_ART.find("\n</art>").unwrap();
    let items = complete_at("SelfButton.vue", INLINE_ART, offset);
    let labels = labels(&items);

    assert!(labels.contains(&"variant"), "{labels:?}");
    assert!(labels.contains(&"Self"), "{labels:?}");
    assert!(!labels.contains(&"art"), "{labels:?}");
    assert!(!labels.contains(&"script setup"), "{labels:?}");
    assert_eq!(
        item(&items, "Self").insert_text.as_deref(),
        Some("<Self $1>$0</Self>")
    );
}

#[test]
fn inline_art_opening_tag_offers_art_metadata_attributes() {
    let offset = INLINE_ART.find("\"Self button\"").unwrap() + "\"Self button\"".len();
    let items = complete_at("SelfButton.vue", INLINE_ART, offset);
    let labels = labels(&items);

    assert!(labels.contains(&"component"), "{labels:?}");
    assert!(labels.contains(&"status"), "{labels:?}");
    assert!(!labels.contains(&"variant"), "{labels:?}");
}

#[test]
fn an_article_tag_in_a_variant_is_not_an_art_opening_tag() {
    // `<article` starts with `<art`; its attributes must not be art metadata.
    let source = INLINE_ART.replace("<Self label=\"Hi\" />", "<article cl");
    let offset = source.find("<article cl").unwrap() + "<article cl".len();
    let items = complete_at("SelfButton.vue", &source, offset);
    let labels = labels(&items);

    assert!(!labels.contains(&"component"), "{labels:?}");
    assert!(!labels.contains(&"category"), "{labels:?}");
}

#[test]
fn a_hyphenated_variant_component_is_not_a_variant_opening_tag() {
    let source = INLINE_ART.replace("<Self label=\"Hi\" />", "<variant-card si");
    let offset = source.find("<variant-card si").unwrap() + "<variant-card si".len();
    let items = complete_at("SelfButton.vue", &source, offset);
    let labels = labels(&items);

    assert!(!labels.contains(&"skip-vrt"), "{labels:?}");
    assert!(!labels.contains(&"viewport"), "{labels:?}");
}
