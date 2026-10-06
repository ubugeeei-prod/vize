//! Preserve adjacency inside an existing source-owned inter-block gap.

pub(super) fn is_attached_comment(gap: &str) -> bool {
    let content = gap.trim_end();
    if !content.ends_with("-->") {
        return false;
    }
    let mut remaining = content;
    loop {
        let Some((_, body)) = remaining.split_once("<!--") else {
            return false;
        };
        let Some((_, after)) = body.split_once("-->") else {
            return false;
        };
        if after.is_empty() {
            break;
        }
        remaining = after;
    }
    let tail = gap.get(content.len()..).unwrap_or_default().as_bytes();
    let mut breaks = 0;
    for (index, &byte) in tail.iter().enumerate() {
        if byte == b'\n' || (byte == b'\r' && tail.get(index + 1) != Some(&b'\n')) {
            breaks += 1;
            if breaks > 1 {
                return false;
            }
        }
    }
    true
}
