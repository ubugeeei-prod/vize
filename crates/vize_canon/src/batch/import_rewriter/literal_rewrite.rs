//! Encode changed module identities at the existing AST literal boundary.

use vize_carton::String;

/// Returns the emitted content length for the import map's atomic interval.
/// The AST spans exclude delimiters. A safe path keeps the original quote and
/// needs no temporary output buffer; only unsafe content uses the shared encoder.
pub(super) fn replace(
    output: &mut String,
    source: &str,
    start: u32,
    end: u32,
    path: &str,
) -> Option<u32> {
    let start = start as usize;
    let end = end as usize;
    let quote_start = start.checked_sub(1)?;
    let quote = *source.as_bytes().get(quote_start)?;
    if !matches!(quote, b'\'' | b'"')
        || source.as_bytes().get(end) != Some(&quote)
        || source.get(start..end).is_none()
    {
        return None;
    }
    if !path
        .bytes()
        .any(|byte| byte == quote || byte == b'\\' || byte < 0x20)
        && !path.contains(['\u{2028}', '\u{2029}'])
    {
        output.replace_range(start..end, path);
        return Some(path.len() as u32);
    }
    let mut encoded = String::with_capacity(path.len() + 2);
    crate::virtual_ts::push_ts_string_literal(&mut encoded, path);
    let content_length = encoded.len() - 2;
    output.replace_range(quote_start..end + 1, &encoded);
    Some(content_length as u32)
}

#[cfg(test)]
mod tests;
