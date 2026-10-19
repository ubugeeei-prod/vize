//! `<style module>` class completion after `$style.` / a named module.

use std::fs;

use tower_lsp::lsp_types::{CompletionItemKind, CompletionResponse, Url};

use crate::ide::{CompletionService, IdeContext};
use crate::server::ServerState;

fn complete(source: &str, cursor: &str) -> Vec<(String, Option<CompletionItemKind>)> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Card.vue");
    fs::write(&path, source).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, source);
    let offset = source.find(cursor).unwrap() + cursor.len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let items = match CompletionService::complete(&ctx) {
        Some(CompletionResponse::Array(items)) => items,
        Some(CompletionResponse::List(list)) => list.items,
        None => Vec::new(),
    };
    items
        .into_iter()
        .map(|item| (item.label, item.kind))
        .collect()
}

fn fields(names: &[&str]) -> Vec<(String, Option<CompletionItemKind>)> {
    names
        .iter()
        .map(|name| ((*name).to_owned(), Some(CompletionItemKind::FIELD)))
        .collect()
}

const STYLES: &str = r#"
<style module>
.title { font-weight: bold }
.card, .card:hover > .icon { color: red }
@media (width > 600px) { .wide { display: grid } }
</style>

<style>
.global-only { color: blue }
</style>

<style module lang="scss">
.from-scss { color: green }
</style>

<style module lang="sass">
.from-sass
  color: green
</style>
"#;

#[test]
fn style_dot_lists_css_and_scss_module_classes_sorted() {
    let source = vize_l0::cstr!("<template>\n  <div :class=\"$style.\" />\n</template>\n{STYLES}");
    assert_eq!(
        complete(&source, "$style."),
        fields(&["card", "from-scss", "icon", "title", "wide"])
    );
}

#[test]
fn style_dot_in_an_interpolation_lists_module_classes() {
    let source = vize_l0::cstr!("<template>\n  <p>{{{{ $style. }}}}</p>\n</template>\n{STYLES}");
    assert_eq!(
        complete(&source, "$style."),
        fields(&["card", "from-scss", "icon", "title", "wide"])
    );
}

#[test]
fn a_partially_typed_class_still_lists_module_classes() {
    let source =
        vize_l0::cstr!("<template>\n  <div :class=\"$style.ti\" />\n</template>\n{STYLES}");
    assert_eq!(
        complete(&source, "$style.ti"),
        fields(&["card", "from-scss", "icon", "title", "wide"])
    );
}

#[test]
fn scss_modules_resolve_nested_parent_references() {
    let source = "<template>\n  <div :class=\"$style.\" />\n</template>\n<style module lang=\"scss\">\n.card {\n  &__title { color: red; }\n  // .commented {}\n  .icon {}\n}\n</style>\n";
    assert_eq!(
        complete(source, "$style."),
        fields(&["card", "card__title", "icon"])
    );
}

#[test]
fn a_named_module_lists_its_own_classes() {
    let source = r#"<template>
  <div :class="classes." />
</template>

<style module="classes">
.named { color: red }
</style>

<style module>
.unnamed { color: blue }
</style>
"#;
    assert_eq!(complete(source, "classes."), fields(&["named"]));
}

#[test]
fn style_dot_without_a_module_block_is_not_a_class_list() {
    let source = "<template>\n  <div :class=\"$style.\" />\n</template>\n<style>\n.plain { color: red }\n</style>\n";
    let labels = complete(source, "$style.");
    assert!(
        labels.iter().all(|(label, _)| label != "plain"),
        "{labels:?}"
    );
}
