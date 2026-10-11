//! Locate the genuine module source, excluding quoted imported binding names.

pub(super) fn module_specifier_end(source: &str) -> Option<usize> {
    let after = source.strip_prefix("import")?;
    if !after.starts_with(|c: char| c.is_whitespace() || matches!(c, '{' | '*' | '\'' | '"' | '/'))
    {
        return None;
    }
    let mut cursor = super::skip_trivia(source, "import".len());
    let mut depth = 0usize;
    let mut first = true;
    while let Some(rest) = source.get(cursor..) {
        let ch = rest.chars().next()?;
        if matches!(ch, '\'' | '"') {
            let end = cursor + quoted_end(rest)?;
            if first {
                return Some(end);
            }
            cursor = end;
            continue;
        }
        if depth == 0
            && let Some(after) = rest.strip_prefix("from")
            && after.starts_with(|c: char| c.is_whitespace() || matches!(c, '\'' | '"' | '/'))
        {
            let start = super::skip_trivia(source, cursor + "from".len());
            if source.get(start..)?.starts_with(['\'', '"']) {
                return Some(start + quoted_end(source.get(start..)?)?);
            }
        }
        first = false;
        if rest.starts_with("//") || rest.starts_with("/*") {
            cursor = super::skip_trivia(source, cursor);
        } else {
            match ch {
                '{' => depth += 1,
                '}' => depth = depth.checked_sub(1)?,
                ';' => return None,
                _ => {}
            }
            cursor += ch.len_utf8();
        }
    }
    None
}

fn quoted_end(source: &str) -> Option<usize> {
    let quote = source.chars().next()?;
    let mut escaped = false;
    for (offset, ch) in source.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Some(offset + ch.len_utf8());
        } else if matches!(ch, '\n' | '\r') {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn quoted_import_names_are_not_module_sources() {
        for source in [
            "import { 'ref' as ref } from 'vue';",
            "import { 'r\\\'ef' as ref } from 'vue';",
            "import type { 'ref' as ref } from 'vue';",
        ] {
            assert_eq!(
                super::module_specifier_end(source),
                source.find("'vue'").map(|start| start + "'vue'".len())
            );
        }
    }
}
