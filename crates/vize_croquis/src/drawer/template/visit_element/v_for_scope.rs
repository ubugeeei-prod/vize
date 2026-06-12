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
) -> SmallVec<[(CompactString, u32); 4]> {
    let (content, base_offset) = expression_content_and_offset(exp);
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

fn expression_content_and_offset<'a>(exp: &'a ExpressionNode<'_>) -> (&'a str, u32) {
    let loc = exp.loc();
    let content = match exp {
        ExpressionNode::Simple(simple) => simple.content.as_str(),
        ExpressionNode::Compound(compound) => compound.loc.source.as_str(),
    };
    (content, loc.start.offset)
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

fn find_identifier_token(text: &str, name: &str) -> Option<usize> {
    text.match_indices(name).find_map(|(index, _)| {
        let before = index
            .checked_sub(1)
            .and_then(|prev| text.as_bytes().get(prev))
            .is_none_or(|byte| !is_identifier_continue(*byte));
        let after = text
            .as_bytes()
            .get(index + name.len())
            .is_none_or(|byte| !is_identifier_continue(*byte));
        (before && after).then_some(index)
    })
}

fn is_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric()
}
