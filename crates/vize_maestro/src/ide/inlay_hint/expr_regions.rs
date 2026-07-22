//! Executable-code regions of a template expression.
//!
//! Prop-usage inlay hints must anchor only on identifier references in
//! code, never on matching text inside string literals, template-literal
//! text, or comments. `${...}` interpolations re-enter code, including
//! through nested template literals.

/// Returns the byte ranges of `expr` that are executable code.
pub(super) fn code_regions(expr: &str) -> Vec<(usize, usize)> {
    let bytes = expr.as_bytes();
    let mut regions = Vec::new();
    let mut region_start = 0usize;
    // One entry per open template literal: the interpolation brace depth,
    // or `None` while scanning that template's literal text.
    let mut templates: Vec<Option<usize>> = Vec::new();
    let mut brace_depth = 0usize;
    let mut i = 0;

    let close_region = |regions: &mut Vec<(usize, usize)>, start: usize, end: usize| {
        if end > start {
            regions.push((start, end));
        }
    };

    while i < bytes.len() {
        if let Some(state @ None) = templates.last_mut() {
            // Template literal text: not code until `${` or the closing tick.
            match bytes[i] {
                b'\\' => i = i.saturating_add(2),
                b'`' => {
                    templates.pop();
                    i += 1;
                    region_start = i;
                }
                b'$' if bytes.get(i + 1) == Some(&b'{') => {
                    *state = Some(brace_depth);
                    brace_depth += 1;
                    i += 2;
                    region_start = i;
                }
                _ => i += 1,
            }
            continue;
        }

        match bytes[i] {
            b'\'' | b'"' => {
                let quote = bytes[i];
                close_region(&mut regions, region_start, i);
                i += 1;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => i = i.saturating_add(2),
                        b if b == quote => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                region_start = i;
            }
            b'`' => {
                close_region(&mut regions, region_start, i);
                templates.push(None);
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                close_region(&mut regions, region_start, i);
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                region_start = i;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                close_region(&mut regions, region_start, i);
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                region_start = i;
            }
            b'{' => {
                brace_depth += 1;
                i += 1;
            }
            b'}' => {
                if let Some(Some(entry_depth)) = templates.last().copied()
                    && brace_depth == entry_depth + 1
                {
                    // Closes the current `${...}` interpolation.
                    close_region(&mut regions, region_start, i);
                    brace_depth = entry_depth;
                    *templates.last_mut().expect("template state") = None;
                } else {
                    brace_depth = brace_depth.saturating_sub(1);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    if templates.is_empty() || matches!(templates.last(), Some(Some(_))) {
        close_region(&mut regions, region_start, bytes.len());
    }
    regions
}

#[cfg(test)]
mod tests {
    use super::code_regions;

    fn code(expr: &str) -> Vec<&str> {
        code_regions(expr)
            .into_iter()
            .map(|(start, end)| &expr[start..end])
            .collect()
    }

    #[test]
    fn template_literal_text_is_not_code_but_interpolations_are() {
        assert_eq!(
            code("[`tag--size-${size}`, `tag--tone-${tone}`]"),
            vec!["[", "size", ", ", "tone", "]"]
        );
    }

    #[test]
    fn string_literals_and_comments_are_not_code() {
        assert_eq!(
            code("'size' + size /* size */ + \"size\""),
            vec![" + size ", " + "]
        );
        assert_eq!(code("size // size"), vec!["size "]);
    }

    #[test]
    fn nested_templates_reenter_code_only_inside_interpolations() {
        assert_eq!(code("`a-${`b-${size}`}-c`"), vec!["size"]);
    }

    #[test]
    fn object_braces_inside_interpolations_stay_balanced() {
        assert_eq!(
            code("`x-${fn({ size })}-y` + size"),
            vec!["fn({ size })", " + size"]
        );
    }

    #[test]
    fn unterminated_template_text_stays_excluded() {
        assert_eq!(code("`tag--size-"), Vec::<&str>::new());
        assert_eq!(code("`tag--${size"), vec!["size"]);
    }
}
