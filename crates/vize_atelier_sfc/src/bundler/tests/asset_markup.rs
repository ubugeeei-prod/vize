//! Only real template elements own asset URLs (#10761).

use super::*;
use vize_l0::String;

fn urls(template: &str) -> Vec<String> {
    let source = ["<template>", template, "</template>"].concat();
    collect_template_asset_urls(&source, None, Some("App.vue"))
        .into_iter()
        .map(|asset| asset.url)
        .collect()
}

#[test]
fn ignores_asset_markup_in_comments() {
    assert!(urls(r#"<!-- <img src="./missing-comment.png"> --><div>ok</div>"#).is_empty());
    assert!(urls(r#"<!--<img src="./a.png">--><!-- -- <use href="./b.svg"> -->"#).is_empty());
    // The asset after the comment's real close is still an element.
    assert_eq!(
        urls(r#"<!-- <img src="./missing.png"> --><img src="./real.png">"#),
        ["./real.png"]
    );
}

#[test]
fn ignores_asset_markup_in_raw_text() {
    assert!(urls(r#"<textarea><img src="./missing-raw.png"></textarea>"#).is_empty());
    assert!(urls(r#"<title><img src="./missing-title.png"></title>"#).is_empty());
    assert_eq!(
        urls(r#"<textarea><img src="./missing.png"></textarea><img src="./real.png">"#),
        ["./real.png"]
    );
}

#[test]
fn keeps_real_assets_beside_quoted_and_nested_markup() {
    assert!(urls(r#"<div title='<img src="./quoted.png">'></div>"#).is_empty());
    assert_eq!(
        urls(r#"<template v-if="true"><img src="./nested.png"></template>"#),
        ["./nested.png"]
    );
    assert_eq!(
        urls(r#"<svg><image href="./a.svg" /><use xlink:href="./b.svg#i" /></svg>"#),
        ["./a.svg", "./b.svg#i"]
    );
    assert_eq!(
        urls(r#"<video poster="./p.png" src="./v.mp4"></video>"#),
        ["./v.mp4", "./p.png"]
    );
    // Values are the parsed attribute text the compiled render holds.
    assert_eq!(urls(r#"<img src="./a&amp;b.png">"#), ["./a&b.png"]);
}
