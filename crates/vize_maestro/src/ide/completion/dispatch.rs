//! Completion entry types, trigger characters, and context-detection helpers.
//!
//! Hosts the public `CompletionService` handle and the trigger-character
//! surface alongside the shared cursor-context predicates used by the
//! block-specific completion handlers.

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
pub(crate) fn is_inside_html_comment(content: &str, offset: usize) -> bool {
    let before = &content[..offset.min(content.len())];
    if let Some(comment_start) = before.rfind("<!--") {
        let after_start = &before[comment_start + 4..];
        !after_start.contains("-->")
    } else {
        false
    }
}

/// Check if cursor is inside <art ...> opening tag.
pub(crate) fn is_inside_art_tag(before: &str) -> bool {
    if let Some(art_start) = before.rfind("<art") {
        let after_art = &before[art_start..];
        !after_art.contains('>')
    } else {
        false
    }
}

/// Check if cursor is inside <variant ...> opening tag.
pub(crate) fn is_inside_variant_tag(before: &str) -> bool {
    if let Some(variant_start) = before.rfind("<variant") {
        let after_variant = &before[variant_start..];
        !after_variant.contains('>')
    } else {
        false
    }
}

/// Check if we should suggest <art> block at root level.
pub(crate) fn should_suggest_art_block(before: &str) -> bool {
    !before.contains("<art")
        && (before.trim().is_empty() || before.ends_with('\n') || before.ends_with('<'))
}

/// Check if we should suggest <variant> block inside <art>.
pub(crate) fn should_suggest_variant_block(before: &str) -> bool {
    if let Some(art_start) = before.rfind("<art") {
        let after_art = &before[art_start..];
        after_art.contains('>') && !after_art.contains("</art>")
    } else {
        false
    }
}
