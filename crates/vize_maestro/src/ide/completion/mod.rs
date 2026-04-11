//! Completion provider for Vue SFC files.
//!
//! Provides context-aware completions for:
//! - Template expressions and directives
//! - Script bindings and imports
//! - CSS properties and Vue-specific selectors
//! - Real completions from Corsa (when available)
//!
//! Uses vize_croquis for accurate scope analysis and type information.
#![allow(clippy::disallowed_methods)]

mod items;
mod script;
mod service;
mod style;
mod template;

/// Completion service for providing context-aware completions.
pub struct CompletionService;

/// Completion trigger characters for Vue SFC.
pub const TRIGGER_CHARACTERS: &[char] = &[
    '<',  // HTML tags
    '.',  // Object property access
    ':',  // v-bind shorthand
    '@',  // v-on shorthand
    '#',  // v-slot shorthand
    '"',  // Attribute values
    '\'', // Attribute values
    '/',  // Closing tags
    ' ',  // Space for attribute completion
];

/// Get trigger characters as strings.
pub fn trigger_characters() -> Vec<String> {
    TRIGGER_CHARACTERS.iter().map(|c| c.to_string()).collect()
}

// =============================================================================
// Context detection helpers
// =============================================================================

/// Check if cursor offset is inside an HTML comment (`<!-- ... -->`).
fn is_inside_html_comment(content: &str, offset: usize) -> bool {
    let before = &content[..offset.min(content.len())];
    if let Some(comment_start) = before.rfind("<!--") {
        let after_start = &before[comment_start + 4..];
        !after_start.contains("-->")
    } else {
        false
    }
}

/// Check if cursor is inside <art ...> opening tag.
fn is_inside_art_tag(before: &str) -> bool {
    if let Some(art_start) = before.rfind("<art") {
        let after_art = &before[art_start..];
        !after_art.contains('>')
    } else {
        false
    }
}

/// Check if cursor is inside <variant ...> opening tag.
fn is_inside_variant_tag(before: &str) -> bool {
    if let Some(variant_start) = before.rfind("<variant") {
        let after_variant = &before[variant_start..];
        !after_variant.contains('>')
    } else {
        false
    }
}

/// Check if we should suggest <art> block at root level.
fn should_suggest_art_block(before: &str) -> bool {
    !before.contains("<art")
        && (before.trim().is_empty() || before.ends_with('\n') || before.ends_with('<'))
}

/// Check if we should suggest <variant> block inside <art>.
fn should_suggest_variant_block(before: &str) -> bool {
    if let Some(art_start) = before.rfind("<art") {
        let after_art = &before[art_start..];
        after_art.contains('>') && !after_art.contains("</art>")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{is_inside_html_comment, items, script, style, template, trigger_characters};
    use tower_lsp::lsp_types::{CompletionItemKind, InsertTextFormat};
    use vize_relief::BindingType;

    #[test]
    fn test_directive_completions() {
        let items = template::directive_completions();
        assert!(!items.is_empty());

        let v_if = items.iter().find(|i| i.label == "v-if");
        assert!(v_if.is_some());
        assert_eq!(v_if.unwrap().kind, Some(CompletionItemKind::KEYWORD));
    }

    #[test]
    fn test_composition_api_completions() {
        let items = script::composition_api_completions();
        assert!(!items.is_empty());

        let ref_item = items.iter().find(|i| i.label == "ref");
        assert!(ref_item.is_some());
        assert_eq!(ref_item.unwrap().kind, Some(CompletionItemKind::FUNCTION));
    }

    #[test]
    fn test_macro_completions() {
        let items = script::macro_completions();
        assert!(!items.is_empty());

        let define_props = items.iter().find(|i| i.label == "defineProps");
        assert!(define_props.is_some());
    }

    #[test]
    fn test_vue_css_completions() {
        let items = style::vue_css_completions();
        assert_eq!(items.len(), 4);

        let deep = items.iter().find(|i| i.label == ":deep");
        assert!(deep.is_some());
    }

    #[test]
    fn test_trigger_characters() {
        let chars = trigger_characters();
        assert!(chars.contains(&"<".to_string()));
        assert!(chars.contains(&":".to_string()));
        assert!(chars.contains(&"@".to_string()));
    }

    #[test]
    fn test_binding_type_to_completion_info() {
        let (kind, detail, _) = items::binding_type_to_completion_info(BindingType::SetupRef);
        assert_eq!(kind, CompletionItemKind::VARIABLE);
        insta::assert_snapshot!(detail);

        let (kind, detail, _) = items::binding_type_to_completion_info(BindingType::SetupConst);
        assert_eq!(kind, CompletionItemKind::CONSTANT);
        insta::assert_snapshot!(detail);

        let (kind, detail, _) = items::binding_type_to_completion_info(BindingType::Props);
        assert_eq!(kind, CompletionItemKind::PROPERTY);
        insta::assert_snapshot!(detail);
    }

    #[test]
    fn test_vize_directive_completions() {
        let items = template::vize_directive_completions();
        assert_eq!(items.len(), 9);

        for item in &items {
            assert_eq!(item.kind, Some(CompletionItemKind::KEYWORD));
        }

        let todo = items.iter().find(|i| i.label == "@vize:todo");
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.insert_text_format, Some(InsertTextFormat::SNIPPET));
        assert_eq!(todo.insert_text, Some("@vize:todo $1 ".to_string()));

        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"@vize:todo"));
        assert!(labels.contains(&"@vize:fixme"));
        assert!(labels.contains(&"@vize:expected"));
        assert!(labels.contains(&"@vize:docs"));
        assert!(labels.contains(&"@vize:ignore-start"));
        assert!(labels.contains(&"@vize:ignore-end"));
        assert!(labels.contains(&"@vize:level(warn)"));
        assert!(labels.contains(&"@vize:deprecated"));
        assert!(labels.contains(&"@vize:dev-only"));
    }

    #[test]
    fn test_is_inside_html_comment() {
        assert!(is_inside_html_comment("<!-- @vize:", 11));
        assert!(is_inside_html_comment("<!-- ", 5));
        assert!(is_inside_html_comment("<div><!-- hello", 15));

        assert!(!is_inside_html_comment("<div>", 5));
        assert!(!is_inside_html_comment("", 0));

        assert!(!is_inside_html_comment("<!-- done -->", 13));
        assert!(!is_inside_html_comment("<!-- done --> text", 18));

        assert!(is_inside_html_comment("<!-- a --> <!-- b", 17));
        assert!(!is_inside_html_comment("<!-- a --> <!-- b --> after", 26));
    }
}
