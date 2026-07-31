//! Completion entry types, trigger characters, and context-detection helpers.
//!
//! Hosts the public `CompletionService` handle and the trigger-character
//! surface alongside the shared cursor-context predicates used by the
//! block-specific completion handlers.

/// Completion service for providing context-aware completions.
pub struct CompletionService;

/// Completion trigger characters for Vue SFC.
///
/// This is `@vue/language-server` 3.3.8's list in its order, plus `'`. Every
/// character here has a handler path that answers with something at that
/// position — a trigger that can only ever open an empty list is worse than no
/// trigger, because the user sees the editor react and offer nothing.
///
/// Two deliberate differences from the reference server:
///
/// - `'` is kept. A single-quoted attribute value is legal Vue and Maestro
///   answers inside one exactly as it does inside `"`; dropping the trigger
///   would silently stop the list from opening there.
/// - ` ` (space) is **not** here. It opened the completion list on every space
///   typed in a template, including inside text nodes and prose. Attribute
///   names still complete as they are typed: those are word characters, which
///   editors suggest on without a trigger character.
// Grouped one purpose per line; rustfmt would otherwise pull each group's
// comment onto the end of the previous line, where it reads as documenting the
// wrong characters.
#[rustfmt::skip]
pub const TRIGGER_CHARACTERS: &[char] = &[
    // Attribute values.
    '"', '\'',
    // v-bind / v-on shorthands, and property access or a `.prop` modifier.
    ':', '@', '.',
    // Tag start, the value position `=` opens before any quote is typed,
    // closing tags, and the text node that follows the end of a start tag.
    '<', '=', '/', '>',
    // Expression syntax inside a directive value or an interpolation, plus the
    // `#` v-slot shorthand and `$`-prefixed bindings (`$style`, `$attrs`).
    '+', '^', '*', '(', ')', '#', '[', ']', '$',
    // kebab-case component and prop names are one hyphenated word, so the list
    // has to refresh mid-word.
    '-',
    // Mustache interpolation `{{`.
    '{', '}',
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

#[cfg(test)]
mod tests {
    use super::trigger_characters;

    #[test]
    fn trigger_characters_match_the_reference_server_plus_single_quote() {
        // `@vue/language-server` 3.3.8 advertises exactly this list in this
        // order, minus `'`. Maestro used to advertise `. : @ # < / " ' SPACE`:
        // thirteen of the reference server's triggers were missing, so
        // `:title=`, `{{`, `(`, `[` and kebab-case names never opened the list
        // on their own, and SPACE opened it on every space typed in a template.
        assert_eq!(
            trigger_characters(),
            vec![
                "\"", "'", ":", "@", ".", "<", "=", "/", ">", "+", "^", "*", "(", ")", "#", "[",
                "]", "$", "-", "{", "}",
            ]
        );
    }
}
