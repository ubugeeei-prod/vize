//! Helper functions for inline script compilation.
//!
//! Provides utility functions for comment stripping and const name extraction
//! used during script parsing.

use vize_carton::String;
/// Strip comments from a line for bracket/paren counting.
/// Removes `// ...` line comments and `/* ... */` block comments while preserving string content.
pub(crate) fn strip_comments_for_counting(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = b'"';

    while let Some(&byte) = bytes.get(i) {
        if in_string {
            let escaped = i.checked_sub(1).and_then(|prev| bytes.get(prev)) == Some(&b'\\');
            if byte == string_char && !escaped {
                in_string = false;
            }
            result.push(byte as char);
            i += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' | b'`' => {
                in_string = true;
                string_char = byte;
                result.push(byte as char);
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                // Line comment: skip rest of line
                break;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                // Block comment: skip until */
                i += 2;
                while bytes.get(i..i + 2).is_some_and(|pair| pair != b"*/") {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2; // skip */
                }
            }
            _ => {
                result.push(byte as char);
                i += 1;
            }
        }
    }
    result
}
