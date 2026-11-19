/// `include(:filter)* path`, split as `pug-lexer` splits it.
pub(super) fn head(text: &str) -> Result<(&str, bool), &'static str> {
    let rest = text.strip_prefix("include").unwrap_or(text);
    if !(rest.is_empty() || rest.starts_with([':', ' '])) {
        return Err("malformed include");
    }
    let filtered = rest.starts_with(':');
    let path = match rest.find(' ') {
        Some(at) if !filtered || at > 0 => rest.get(at..).unwrap_or_default(),
        _ => "",
    };
    let path = path.trim_matches(super::super::literal::is_js_space);
    if path.is_empty() {
        return Err("missing path for include");
    }
    Ok((path, filtered))
}

#[cfg(test)]
mod tests {
    use super::head;

    #[test]
    fn heads_split_like_pugs_lexer() {
        assert_eq!(head("include a/b.pug "), Ok(("a/b.pug", false)));
        assert_eq!(head("include:markdown-it doc.md"), Ok(("doc.md", true)));
        assert_eq!(head("include"), Err("missing path for include"));
        assert_eq!(head("includes x"), Err("malformed include"));
    }
}
