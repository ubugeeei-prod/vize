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

    while let Some(&byte) = bytes.get(i) {
        if let Some(state @ None) = templates.last_mut() {
            // Template literal text: not code until `${` or the closing tick.
            match byte {
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

        match byte {
            quote @ (b'\'' | b'"') => {
                close_region(&mut regions, region_start, i);
                i += 1;
                while let Some(&next) = bytes.get(i) {
                    match next {
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
                while bytes.get(i).is_some_and(|&next| next != b'\n') {
                    i += 1;
                }
                region_start = i;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                close_region(&mut regions, region_start, i);
                i += 2;
                while bytes.get(i..i + 2).is_some_and(|pair| pair != b"*/") {
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
                let closed_depth = templates.last_mut().and_then(|state| match *state {
                    Some(entry_depth) if brace_depth == entry_depth + 1 => {
                        *state = None;
                        Some(entry_depth)
                    }
                    _ => None,
                });
                if let Some(entry_depth) = closed_depth {
                    // Closes the current `${...}` interpolation.
                    close_region(&mut regions, region_start, i);
                    brace_depth = entry_depth;
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

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
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
