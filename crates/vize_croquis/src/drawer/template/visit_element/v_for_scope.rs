use oxc_syntax::identifier::is_identifier_part;

use crate::drawer::helpers::VForScopeAliases;
use crate::scope::ParamNames;
use vize_carton::{CompactString, SmallVec};
use vize_relief::ExpressionNode;

pub(super) fn v_for_scope_bindings(aliases: &VForScopeAliases) -> ParamNames {
    let mut bindings = aliases.value_bindings.clone();
    if let Some(key) = &aliases.key_alias {
        bindings.push(key.clone());
    }
    if let Some(index) = &aliases.index_alias {
        bindings.push(index.clone());
    }
    bindings
}

pub(super) fn v_for_alias_declaration_offsets(
    exp: &ExpressionNode<'_>,
    aliases: &VForScopeAliases,
    template_source: &str,
) -> SmallVec<[(CompactString, u32); 4]> {
    let (content, base_offset) = expression_content_and_offset(exp, template_source);
    let Some((alias_start, alias_end)) = v_for_alias_range(content) else {
        return SmallVec::new();
    };
    let alias_text = &content[alias_start..alias_end];
    let alias_base = base_offset + alias_start as u32;

    let mut offsets = SmallVec::new();
    for name in v_for_scope_bindings(aliases) {
        if let Some(relative) = find_identifier_token(alias_text, name.as_str()) {
            offsets.push((name, alias_base + relative as u32));
        }
    }
    offsets
}

pub(super) fn v_for_source_offset(
    exp: &ExpressionNode<'_>,
    aliases: &VForScopeAliases,
    template_source: &str,
) -> Option<u32> {
    let (content, base_offset) = expression_content_and_offset(exp, template_source);
    source_offset_in_expression(content, base_offset, aliases.source.as_str())
}

fn source_offset_in_expression(content: &str, base_offset: u32, source: &str) -> Option<u32> {
    content
        .rfind(source)
        .map(|relative| base_offset + relative as u32)
}

fn expression_content_and_offset<'a>(
    exp: &'a ExpressionNode<'_>,
    source: &'a str,
) -> (&'a str, u32) {
    let loc = exp.loc();
    let content = match exp {
        ExpressionNode::Simple(simple) => simple.content.as_str(),
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    };
    (content, loc.span.start)
}

fn v_for_alias_range(expr: &str) -> Option<(usize, usize)> {
    let leading = expr.len() - expr.trim_start().len();
    let trimmed = expr.trim();
    let separator = find_v_for_separator(trimmed)?;
    let alias = &trimmed[..separator];
    let alias_leading = alias.len() - alias.trim_start().len();
    let alias_end = alias.trim_end().len();
    Some((leading + alias_leading, leading + alias_end))
}

fn find_v_for_separator(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut index = 0;
    while index + 4 <= bytes.len() {
        if bytes[index] == b' '
            && ((bytes[index + 1] == b'i' && bytes[index + 2] == b'n')
                || (bytes[index + 1] == b'o' && bytes[index + 2] == b'f'))
            && bytes[index + 3] == b' '
        {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// The offset at which `name` is declared inside a `v-for` alias list.
///
/// Canon's `pattern_identifier_offset` mirrors this to anchor the generated
/// alias identifiers, so both sides must pick the same token.
fn find_identifier_token(text: &str, name: &str) -> Option<usize> {
    text.match_indices(name)
        .find(|(index, _)| is_declaration_token(text, *index, name.len()))
        .map(|(index, _)| index)
}

/// Whether the `[at, at + len)` slice of `text` declares a binding.
///
/// It must not be part of a longer identifier — bounded by ECMAScript's
/// `IdentifierPart`, so `it` matches neither inside `éit` the way a byte-wise
/// ASCII test would nor inside `a\u{301}it` or `it\u{200C}tail`, which a
/// `char::is_alphanumeric` test would miss — and must not be the property of a
/// member access (`{ kind = other.it, it }`), which references another binding
/// instead of declaring this one. A rest element (`[first, ...rest]`) is a
/// declaration and stays eligible.
fn is_declaration_token(text: &str, at: usize, len: usize) -> bool {
    let leading = &text[..at];
    let before = leading.chars().next_back();
    let after = text[at + len..].chars().next();
    if before.is_some_and(is_identifier_part) || after.is_some_and(is_identifier_part) {
        return false;
    }
    before != Some('.') || leading.ends_with("..")
}

#[cfg(test)]
mod tests {
    use super::{find_identifier_token, source_offset_in_expression};

    #[test]
    fn alias_tokens_are_declarations_not_references() {
        assert_eq!(find_identifier_token("({ id, name }, i)", "name"), Some(7));
        // A longer identifier never yields its suffix, ASCII or not.
        assert_eq!(find_identifier_token("item", "it"), None);
        assert_eq!(find_identifier_token("{ éit, it }", "it"), Some(8));
        // A combining mark and a zero-width joiner continue an identifier too.
        assert_eq!(find_identifier_token("{ a\u{301}it, it }", "it"), Some(9));
        assert_eq!(
            find_identifier_token("{ it\u{200c}tail, it }", "it"),
            Some(13)
        );
        // A default value's member access references another binding.
        assert_eq!(
            find_identifier_token("{ kind = other.it, it }", "it"),
            Some(19)
        );
        // A rest element declares its binding.
        assert_eq!(find_identifier_token("[first, ...it]", "it"), Some(11));
    }

    #[test]
    fn source_offsets_keep_whitespace_and_choose_the_final_duplicate() {
        assert_eq!(
            source_offset_in_expression("  item in item  ", 40, "item"),
            Some(50)
        );
        assert_eq!(
            source_offset_in_expression(
                "({ id }, key) of rows.filter(row => row.id)",
                7,
                "rows.filter(row => row.id)"
            ),
            Some(24)
        );
    }
}
