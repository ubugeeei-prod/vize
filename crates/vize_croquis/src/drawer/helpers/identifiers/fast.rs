use super::IdentifierRef;
use vize_carton::CompactString;

/// Fast string-based identifier extraction for simple expressions.
#[inline]
pub(super) fn extract_identifiers_fast(expr: &str) -> Vec<CompactString> {
    extract_identifier_refs_fast(expr)
        .into_iter()
        .map(|identifier| identifier.name)
        .collect()
}

#[inline]
pub(super) fn extract_identifier_refs_fast(expr: &str) -> Vec<IdentifierRef> {
    extract_identifier_refs_fast_with_base(expr, 0)
}

fn extract_identifier_refs_fast_with_base(expr: &str, base_offset: u32) -> Vec<IdentifierRef> {
    let mut identifiers = Vec::with_capacity(4);
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let c = bytes[i];

        // Skip single-quoted strings
        if c == b'\'' {
            i += 1;
            while i < len && bytes[i] != b'\'' {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < len {
                i += 1;
            }
            continue;
        }

        // Skip double-quoted strings
        if c == b'"' {
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < len {
                i += 1;
            }
            continue;
        }

        // Handle template literals
        if c == b'`' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'`' {
                    i += 1;
                    break;
                }
                if bytes[i] == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
                    i += 2;
                    let interp_start = i;
                    let mut brace_depth = 1;
                    while i < len && brace_depth > 0 {
                        match bytes[i] {
                            b'{' => brace_depth += 1,
                            b'}' => brace_depth -= 1,
                            _ => {}
                        }
                        if brace_depth > 0 {
                            i += 1;
                        }
                    }
                    if interp_start < i {
                        let interp_content = &expr[interp_start..i];
                        for ident in extract_identifier_refs_fast_with_base(
                            interp_content,
                            base_offset + interp_start as u32,
                        ) {
                            identifiers.push(ident);
                        }
                    }
                    if i < len {
                        i += 1;
                    }
                    continue;
                }
                i += 1;
            }
            continue;
        }

        if c == b'/' && i + 1 < len {
            match bytes[i + 1] {
                b'/' => {
                    i += 2;
                    while i < len && bytes[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                b'*' => {
                    i += 2;
                    while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    i = (i + 2).min(len);
                    continue;
                }
                _ => {}
            }
        }

        // Start of identifier
        if c.is_ascii_alphabetic() || c == b'_' || c == b'$' {
            let start = i;
            i += 1;
            while i < len
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }

            // Check if preceded by '.' (property access)
            let is_property_access = if start > 0 {
                let mut j = start - 1;
                loop {
                    let prev = bytes[j];
                    if prev == b' ' || prev == b'\t' || prev == b'\n' || prev == b'\r' {
                        if j == 0 {
                            break false;
                        }
                        j -= 1;
                    } else {
                        break prev == b'.';
                    }
                }
            } else {
                false
            };

            if !is_property_access {
                identifiers.push(IdentifierRef::new(
                    &expr[start..i],
                    base_offset + start as u32,
                ));
            }
        } else {
            i += 1;
        }
    }

    identifiers
}
