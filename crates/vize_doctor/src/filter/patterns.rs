use vize_l0::String;

pub(super) fn matches_enum<T: PartialEq>(accepted: &[T], value: T) -> bool {
    accepted.is_empty() || accepted.contains(&value)
}

pub(super) fn sort_dedup<T: Ord>(values: &mut Vec<T>) {
    values.sort();
    values.dedup();
}

pub(super) fn expand_top_level_commas(patterns: &mut Vec<String>) {
    let mut expanded = Vec::with_capacity(patterns.len());
    for pattern in patterns.drain(..) {
        if !has_top_level_comma(&pattern) {
            expanded.push(pattern);
            continue;
        }
        let mut depth = 0usize;
        let mut start = 0usize;
        for (index, ch) in pattern.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    if let Some(piece) = pattern.get(start..index) {
                        push_trimmed(&mut expanded, piece);
                    }
                    start = index + ch.len_utf8();
                }
                _ => {}
            }
        }
        if let Some(piece) = pattern.get(start..) {
            push_trimmed(&mut expanded, piece);
        }
    }
    *patterns = expanded;
}

fn has_top_level_comma(pattern: &str) -> bool {
    let mut depth = 0usize;
    for ch in pattern.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

fn push_trimmed(patterns: &mut Vec<String>, pattern: &str) {
    let pattern = pattern.trim();
    if !pattern.is_empty() {
        patterns.push(pattern.into());
    }
}

pub(super) fn normalize_patterns(patterns: &mut Vec<String>, path: bool) {
    for pattern in patterns.iter_mut() {
        *pattern = pattern.trim().into();
        if path && pattern.contains('\\') {
            *pattern = pattern.replace('\\', "/").into();
        }
    }
    sort_dedup(patterns);
}
