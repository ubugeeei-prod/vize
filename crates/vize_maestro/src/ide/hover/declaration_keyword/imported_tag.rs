//! Keyword alignment for marker-less imported component tags (#3937).

use super::import_binds_word;

/// Rewrite a quick-info block opening with `var {word}` to `(property) {word}`
/// when the hover sits on a component tag (`<Word`) and an import binds the
/// word — the Volar 2.2.10 oracle shape for tags backed by non-.vue modules
/// (#3937). The type body is left untouched; it already matches.
pub(super) fn imported_tag_property_keyword(
    markdown: &str,
    content: &str,
    offset: usize,
    script_setup: &str,
    lang: Option<&str>,
    word: &str,
) -> Option<String> {
    if word.is_empty() {
        return None;
    }
    let fence_start = markdown.find("```")?;
    let after_fence = markdown[fence_start..].find('\n')? + fence_start + 1;
    let synthesized = format!("var {word}");
    if !markdown[after_fence..].starts_with(&synthesized) {
        return None;
    }
    let following = markdown[after_fence + synthesized.len()..].chars().next();
    if following.is_some_and(|c| c == '_' || c == '$' || c.is_alphanumeric()) {
        return None;
    }
    if !hover_is_on_tag(content, offset, word) {
        return None;
    }
    if !import_binds_word(script_setup, lang, word) {
        return None;
    }
    let mut rewritten = String::with_capacity(markdown.len() + "(property)".len());
    rewritten.push_str(&markdown[..after_fence]);
    rewritten.push_str("(property)");
    rewritten.push_str(&markdown[after_fence + "var".len()..]);
    Some(rewritten)
}

/// Whether `offset` sits inside an occurrence of `word` that is written as a
/// tag name — immediately preceded by `<`. A hover on the same identifier in
/// an expression keeps the checker's answer.
///
/// The identifier scan is Unicode-aware: a non-ASCII letter such as the `Å` of
/// `<ÅButton` continues the name rather than ending it, and `index` advances by
/// the separator's full UTF-8 width so the resulting byte offset stays on a
/// char boundary.
fn hover_is_on_tag(content: &str, offset: usize, word: &str) -> bool {
    let clamped = offset.min(content.len());
    let Some(before) = content.get(..clamped) else {
        return false;
    };
    let word_start = before
        .char_indices()
        .rev()
        .find(|(_, c)| !(c.is_alphanumeric() || *c == '_' || *c == '$' || *c == '-'))
        .map_or(0, |(index, c)| index + c.len_utf8());
    content
        .get(word_start..)
        .is_some_and(|rest| rest.starts_with(word))
        && before[..word_start].ends_with('<')
}
