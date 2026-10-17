//! Authored local CSS selector names shared by typechecking and completion.
//!
//! This bounded query does not resolve imports or claim a closed module shape.
//! Unsupported selector syntax refuses instead of guessing names.

use std::collections::BTreeSet;

use vize_carton::{String as CompactString, ToCompactString};

/// Extract simple authored class selectors from selector preludes. Declarations
/// are never scanned, so values such as `url(./asset.png)` cannot become fake
/// module keys. Comments and string contents are blanked before inspection.
pub fn authored_local_css_classes(css: &str) -> Option<BTreeSet<CompactString>> {
    let bytes = css.as_bytes();
    let mut classes = BTreeSet::new();
    let mut prelude = Vec::new();
    let mut index = 0usize;
    let mut quote = None;

    while let Some(&byte) = bytes.get(index) {
        if let Some(open_quote) = quote {
            if byte == b'\\' {
                index = (index + 2).min(bytes.len());
                continue;
            }
            if byte == open_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            let relative_end = css.get(index + 2..)?.find("*/")?;
            index += relative_end + 4;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        match byte {
            b'{' => {
                collect_classes_from_selector_prelude(&prelude, &mut classes)?;
                prelude.clear();
            }
            b';' | b'}' => prelude.clear(),
            _ => prelude.push(byte),
        }
        index += 1;
    }
    quote.is_none().then_some(classes)
}

fn collect_classes_from_selector_prelude(
    prelude: &[u8],
    classes: &mut BTreeSet<CompactString>,
) -> Option<()> {
    let trimmed = prelude.trim_ascii_start();
    if trimmed.starts_with(b"@") {
        return Some(());
    }
    if trimmed.contains(&b'\\') || !trimmed.is_ascii() {
        return None;
    }

    let mut index = 0usize;
    let mut local = true;
    // Functions restore the surrounding scope when they close. A bare global
    // scope survives commas inside a selector function; only a top-level comma
    // starts a fresh local selector.
    let mut functions = Vec::new();
    let mut attribute_depth = 0usize;
    while let Some(&byte) = trimmed.get(index) {
        if byte == b'[' {
            attribute_depth += 1;
        } else if byte == b']' {
            attribute_depth = attribute_depth.checked_sub(1)?;
        } else if attribute_depth == 0 {
            let scope = [(b":global".as_slice(), false), (b":local".as_slice(), true)]
                .into_iter()
                .find(|(keyword, _)| {
                    trimmed
                        .get(index..)
                        .is_some_and(|rest| rest.starts_with(keyword))
                        && trimmed
                            .get(index + keyword.len())
                            .is_none_or(|&next| next == b'(' || next.is_ascii_whitespace())
                });
            if let Some((keyword, mode)) = scope {
                index += keyword.len();
                if trimmed.get(index) == Some(&b'(') {
                    functions.push(local);
                    index += 1;
                }
                local = mode;
                continue;
            }
            match byte {
                b'(' => functions.push(local),
                b')' => local = functions.pop()?,
                b',' if functions.is_empty() => local = true,
                b'.' if local
                    && trimmed
                        .get(index + 1)
                        .is_some_and(|&b| is_css_class_start(b)) =>
                {
                    let start = index + 1;
                    let mut end = start + 1;
                    while trimmed.get(end).is_some_and(|&b| is_css_class_continue(b)) {
                        end += 1;
                    }
                    let class_name = std::str::from_utf8(trimmed.get(start..end)?).ok()?;
                    classes.insert(class_name.to_compact_string());
                    index = end;
                    continue;
                }
                _ => {}
            }
        }
        index += 1;
    }
    if attribute_depth != 0 || !functions.is_empty() {
        return None;
    }
    Some(())
}

fn is_css_class_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'-')
}

fn is_css_class_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
