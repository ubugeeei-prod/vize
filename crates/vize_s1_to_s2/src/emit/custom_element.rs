pub(crate) fn tag_pattern_matches(pattern: &str, tag: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if pattern.bytes().all(|byte| byte == b'*') {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == tag;
    }

    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');
    let mut position = 0;
    let mut matched_any = false;

    for (index, part) in pattern
        .split('*')
        .filter(|part| !part.is_empty())
        .enumerate()
    {
        matched_any = true;
        if index == 0 && !starts_with_wildcard {
            if !tag[position..].starts_with(part) {
                return false;
            }
            position += part.len();
            continue;
        }

        let Some(found) = tag[position..].find(part) else {
            return false;
        };
        position += found + part.len();
    }

    if !matched_any {
        return false;
    }

    if !ends_with_wildcard
        && let Some(last_part) = pattern.rsplit('*').find(|part| !part.is_empty())
    {
        return tag.ends_with(last_part);
    }

    true
}
