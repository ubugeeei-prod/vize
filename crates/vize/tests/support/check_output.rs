use std::path::Path;

/// Replaces the per-run project root with `<project>` and every
/// `{:.2?}`-formatted `Duration` token (`12.34ms`, `1.20s`, `999.00µs`, …)
/// with `<duration>`, so captured CLI output can be pinned byte-exact.
pub(crate) fn normalize_check_output(text: &str, project_root: &Path) -> String {
    let rooted = text.replace(&project_root.display().to_string(), "<project>");
    let chars: Vec<char> = rooted.chars().collect();
    let mut out = String::with_capacity(rooted.len());
    let mut index = 0;
    while index < chars.len() {
        let at_boundary = index == 0 || !chars[index - 1].is_ascii_alphanumeric();
        if at_boundary && chars[index].is_ascii_digit() {
            let mut end = index;
            while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
                end += 1;
            }
            let unit_len = ["ns", "\u{b5}s", "ms", "s"]
                .iter()
                .filter_map(|unit| {
                    let unit_chars: Vec<char> = unit.chars().collect();
                    let follows = chars.get(end + unit_chars.len());
                    let ends_clean = follows.is_none_or(|c| !c.is_ascii_alphanumeric());
                    (chars[end..].starts_with(&unit_chars) && ends_clean)
                        .then_some(unit_chars.len())
                })
                .max();
            if let Some(unit_len) = unit_len {
                out.push_str("<duration>");
                index = end + unit_len;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}
