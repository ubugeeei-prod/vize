//! Inline `<script>` extraction for standalone HTML documents.
//!
//! Built-in script rules run against `<script>` blocks embedded in plain HTML
//! (e.g. petite-vue / CDN usage), not just SFCs. These byte scanners locate
//! each inline script body and its source offset so diagnostics map back to the
//! original document.

/// Extract `(script body, content offset)` pairs for every non-empty inline
/// `<script>` block in `source`.
pub(super) fn extract_inline_scripts(source: &str) -> Vec<(&str, usize)> {
    let mut scripts = Vec::new();
    let mut cursor = 0;

    while let Some(open_start) = find_script_open(source, cursor) {
        let Some(open_end) = find_tag_end(source, open_start) else {
            break;
        };

        let content_start = open_end + 1;
        let Some(close_start) = find_ascii_case_insensitive(source, "</script", content_start)
        else {
            break;
        };

        let content = &source[content_start..close_start];
        if !content.trim().is_empty() {
            scripts.push((content, content_start));
        }

        cursor = find_tag_end(source, close_start).map_or(close_start + 9, |end| end + 1);
    }

    scripts
}

fn find_script_open(source: &str, from: usize) -> Option<usize> {
    let mut cursor = from;
    while let Some(index) = find_ascii_case_insensitive(source, "<script", cursor) {
        let boundary = source.as_bytes().get(index + 7).copied();
        if matches!(
            boundary,
            None | Some(b'>' | b'/' | b' ' | b'\n' | b'\r' | b'\t' | b'\x0c')
        ) {
            return Some(index);
        }
        cursor = index + 7;
    }
    None
}

fn find_tag_end(source: &str, from: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, byte) in source.as_bytes()[from..].iter().copied().enumerate() {
        match (quote, byte) {
            (Some(current), value) if value == current => quote = None,
            (None, b'"' | b'\'') => quote = Some(byte),
            (None, b'>') => return Some(from + relative),
            _ => {}
        }
    }
    None
}

fn find_ascii_case_insensitive(source: &str, needle: &str, from: usize) -> Option<usize> {
    let haystack = source.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() || from >= haystack.len() {
        return None;
    }

    haystack[from..]
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
        .map(|index| from + index)
}

#[cfg(test)]
mod standalone_html_tests {
    use super::extract_inline_scripts;

    #[test]
    fn extracts_inline_scripts_from_standalone_html() {
        let source = r##"<!doctype html>
<html>
<head>
  <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
</head>
<body>
  <script>
Vue.createApp({ data() { return { count: 0 } } }).mount("#app")
  </script>
</body>
</html>"##;

        let scripts = extract_inline_scripts(source);
        assert_eq!(scripts.len(), 1);
        assert!(scripts[0].0.contains("Vue.createApp"));
        assert_eq!(&source[scripts[0].1..scripts[0].1 + 3], "\nVu");
    }
}
