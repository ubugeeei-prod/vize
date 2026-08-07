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

    while i < bytes.len() {
        if in_string {
            if bytes[i] == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }

        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                in_string = true;
                string_char = bytes[i];
                result.push(bytes[i] as char);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // Line comment: skip rest of line
                break;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // Block comment: skip until */
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2; // skip */
                }
            }
            _ => {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
    }
    result
}
