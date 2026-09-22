//! Document links read the resident descriptor (P5-6c). A request and a
//! hover on the same buffer share one parse per revision.

use tower_lsp::lsp_types::{DocumentLink, Url};
use vize_resident::DescriptorStats;

use super::DocumentLinkService;
use crate::ide::{HoverService, IdeContext};
use crate::server::ServerState;

const SFC: &str = r#"<script setup lang="ts">
import Button from './Button.vue'
const count = ref(0)
</script>

<template>
  <Button>{{ count }}</Button>
</template>

<style scoped>
@import './extra.css';
</style>
"#;

fn uri() -> Url {
    Url::parse("file:///project/src/Counter.vue").unwrap()
}

fn counts(lookups: u32, parses: u32) -> DescriptorStats {
    DescriptorStats { lookups, parses }
}

fn fingerprint(links: &[DocumentLink]) -> Vec<(u32, u32, u32, u32, String)> {
    links
        .iter()
        .map(|link| {
            (
                link.range.start.line,
                link.range.start.character,
                link.range.end.line,
                link.range.end.character,
                link.target
                    .as_ref()
                    .map(|target| target.to_string())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

fn clean_links(text: &str, document: &Url) -> Vec<DocumentLink> {
    let descriptor = vize_resident::descriptor::parse_descriptor(document.path(), text)
        .expect("the fixture parses");
    DocumentLinkService::links_from_descriptor(text, document, &descriptor)
}

fn targets(links: &[DocumentLink]) -> String {
    fingerprint(links)
        .into_iter()
        .map(|link| link.4)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn requests_share_one_parse_with_hover() {
    let state = ServerState::new();
    let uri = uri();
    let links = DocumentLinkService::get_links(&state, SFC, &uri);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 1),
        "the first request parses"
    );
    assert_eq!(fingerprint(&links), fingerprint(&clean_links(SFC, &uri)));
    let linked = targets(&links);
    assert!(
        linked.contains("Button.vue"),
        "the import stays linked, got {linked}"
    );
    assert!(
        linked.contains("extra.css"),
        "the style import stays linked, got {linked}"
    );

    let offset = SFC.find("{{ count").unwrap() + "{{ co".len();
    let ctx = IdeContext::with_content(&state, &uri, offset, String::from(SFC));
    let _hover = HoverService::hover(&ctx);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 0),
        "hover shares the document-link parse"
    );

    let edited = SFC.replace("./Button.vue", "./Other.vue");
    let edited_links = DocumentLinkService::get_links(&state, &edited, &uri);
    assert_eq!(
        state.resident.take_stats(),
        counts(1, 1),
        "an edit is a new revision"
    );
    assert_eq!(
        fingerprint(&edited_links),
        fingerprint(&clean_links(&edited, &uri))
    );
    assert_ne!(fingerprint(&edited_links), fingerprint(&links));
    let edited_targets = targets(&edited_links);
    assert!(
        edited_targets.contains("Other.vue"),
        "the edited import stays linked, got {edited_targets}"
    );
}

#[test]
fn an_unchanged_buffer_parses_nothing() {
    let state = ServerState::new();
    let uri = uri();
    let first = DocumentLinkService::get_links(&state, SFC, &uri);
    let _first_parse = state.resident.take_stats();
    let second = DocumentLinkService::get_links(&state, SFC, &uri);
    assert_eq!(state.resident.take_stats(), counts(1, 0));
    assert_eq!(fingerprint(&second), fingerprint(&first));
}

#[test]
fn a_rejected_buffer_stays_memoized() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Broken.vue").unwrap();
    let broken = "<template><div></div>";
    assert!(DocumentLinkService::get_links(&state, broken, &uri).is_empty());
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert!(DocumentLinkService::get_links(&state, broken, &uri).is_empty());
    assert_eq!(state.resident.take_stats(), counts(1, 0));

    let fixed = "<script setup lang=\"ts\">\nimport Button from './Button.vue'\n</script>\n";
    let links = DocumentLinkService::get_links(&state, fixed, &uri);
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert_eq!(fingerprint(&links), fingerprint(&clean_links(fixed, &uri)));
    assert!(targets(&links).contains("Button.vue"));
}

#[test]
fn closing_the_document_reparses() {
    let state = ServerState::new();
    let uri = uri();
    let first = DocumentLinkService::get_links(&state, SFC, &uri);
    state.close_document(&uri);
    let _closed = state.resident.take_stats();
    let reopened = DocumentLinkService::get_links(&state, SFC, &uri);
    assert_eq!(state.resident.take_stats(), counts(1, 1));
    assert_eq!(fingerprint(&reopened), fingerprint(&first));
}
