use tower_lsp::lsp_types::CompletionItem;

use super::{Document, edit, hover_documentation, item};

fn labels(items: &[CompletionItem]) -> Vec<&str> {
    items.iter().map(|item| item.label.as_str()).collect()
}

#[test]
fn nested_rules_in_braced_preprocessors_complete_properties() {
    for lang in ["scss", "less"] {
        let items = Document::lang(lang, ".card { .title { font-fa| } }").complete();
        assert_eq!(
            edit(item(&items, "font-family")).new_text,
            "font-family: ",
            "{lang}"
        );
    }
}

#[test]
fn indented_preprocessors_complete_properties_and_values() {
    for lang in ["sass", "stylus"] {
        let items = Document::lang(lang, ".card\n  font-fa|").complete();
        assert_eq!(
            edit(item(&items, "font-family")).new_text,
            "font-family: ",
            "{lang}"
        );

        let items = Document::lang(lang, ".card\n  display: fl|").complete();
        assert_eq!(edit(item(&items, "flex")).new_text, "flex", "{lang}");
    }
    let items = Document::lang("stylus", ".card\n  display fl|").complete();
    assert_eq!(edit(item(&items, "flex")).new_text, "flex");
}

#[test]
fn indented_preprocessors_hover_properties() {
    for lang in ["sass", "stylus"] {
        let document = Document::lang(lang, ".card\n  dis|play: flex");
        let hover = super::super::hover(&document.context(), 0).expect("property hover");
        let docs = hover_documentation(&hover);
        assert!(docs.contains("**display**"), "{lang}: {docs}");
    }
}

#[test]
fn indented_selector_lines_offer_no_property_completions() {
    let items = Document::lang("sass", "car|").complete();
    assert_eq!(
        labels(&items),
        vec!["v-bind", ":deep", ":slotted", ":global"]
    );
}
