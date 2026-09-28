//! JavaScript identifier lookup at an authored UTF-8 cursor offset.

/// Extract the complete identifier even when the cursor is inside a multi-byte scalar.
pub(crate) fn at_offset(content: &str, offset: usize) -> Option<String> {
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
    content.get(start..end).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::at_offset;

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
