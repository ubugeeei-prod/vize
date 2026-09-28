//! Small authored Vue snippets for template hover code fences.

/// Return the opening tag containing `offset`, preserving its authored text.
/// Attribute expressions may contain `>`, so stop only outside quoted values.
pub(crate) fn opening_tag_at(source: &str, offset: usize) -> Option<&str> {
    let before = source.get(..=offset)?;
    let start = before.rfind('<')?;
    let tail = source.get(start..)?;
    let mut quote = None;
    for (relative, ch) in tail.char_indices() {
        if relative > 1024 {
            return None;
        }
        match (quote, ch) {
            (None, '>') => return source.get(start..start + relative + 1),
            (None, '\'' | '"') => quote = Some(ch),
            (Some(open), close) if open == close => quote = None,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::opening_tag_at;

    #[test]
    fn keeps_authored_attribute_expression_in_opening_tag() {
        let source = r#"<Child :show="count > 0" @save="save">
  <span />
</Child>"#;
        let offset = source.find("count").unwrap();
        assert_eq!(
            opening_tag_at(source, offset),
            Some(r#"<Child :show="count > 0" @save="save">"#)
        );
    }
}
