//! JavaScript identifier lookup at an authored UTF-8 cursor offset.

/// Extract the complete identifier even when the cursor is inside a multi-byte scalar.
pub(crate) fn at_offset(content: &str, offset: usize) -> Option<String> {
    content
        .get(span_at_offset(content, offset)?)
        .map(str::to_string)
}

/// The authored byte span of the identifier at `offset`.
pub(crate) fn span_at_offset(content: &str, offset: usize) -> Option<std::ops::Range<usize>> {
    let mut cursor = offset.min(content.len());
    while !content.is_char_boundary(cursor) {
        cursor -= 1;
    }

    let part = oxc_syntax::identifier::is_identifier_part;
    let selected = if content
        .get(cursor..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(part)
    {
        cursor
    } else {
        let (previous, ch) = content.get(..cursor)?.char_indices().next_back()?;
        if !part(ch) {
            return None;
        }
        previous
    };

    let mut start = selected;
    for (index, ch) in content.get(..selected)?.char_indices().rev() {
        if !part(ch) {
            break;
        }
        start = index;
    }
    if !content
        .get(start..)?
        .chars()
        .next()
        .is_some_and(oxc_syntax::identifier::is_identifier_start)
    {
        return None;
    }

    let mut end = selected;
    for (index, ch) in content.get(selected..)?.char_indices() {
        if !part(ch) {
            break;
        }
        end = selected + index + ch.len_utf8();
    }
    Some(start..end)
}

/// The receiver of a member name: `params` for `route.params.page`, empty
/// for `call().page`. A spread (`...page`) still references a binding.
pub(crate) fn member_receiver(content: &str, offset: usize) -> Option<&str> {
    let before = content
        .get(..span_at_offset(content, offset)?.start)?
        .trim_end();
    if before.ends_with("...") {
        return None;
    }
    let receiver = before.strip_suffix('.')?;
    let receiver = receiver.strip_suffix('?').unwrap_or(receiver).trim_end();
    let start = receiver
        .char_indices()
        .rev()
        .take_while(|(_, ch)| oxc_syntax::identifier::is_identifier_part(*ch))
        .last()
        .map_or(receiver.len(), |(index, _)| index);
    receiver.get(start..)
}

#[cfg(test)]
mod tests {
    use super::{at_offset, member_receiver};

    #[test]
    fn member_names_are_distinct_from_binding_references() {
        let source = "const page = route.params.page ?? this?.page ?? route?.params\n  .page; f(...page, g().page)";
        let receivers = source
            .match_indices("page")
            .map(|(offset, _)| member_receiver(source, offset + 1))
            .collect::<Vec<_>>();
        assert_eq!(
            receivers,
            [
                None,
                Some("params"),
                Some("this"),
                Some("params"),
                None,
                Some("")
            ]
        );
    }

    #[test]
    fn cjk_and_accented_identifiers_keep_full_utf8_span() {
        let source = "const 名前 = café;";
        let cjk = source.find("名前").unwrap();
        for offset in cjk..=cjk + "名前".len() {
            assert_eq!(at_offset(source, offset).as_deref(), Some("名前"));
        }
        let accented = source.find("café").unwrap();
        assert_eq!(
            at_offset(source, accented + "café".len()).as_deref(),
            Some("café")
        );
        assert_eq!(at_offset(source, source.find('=').unwrap()), None);
    }
}
