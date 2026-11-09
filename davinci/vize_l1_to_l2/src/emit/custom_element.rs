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
    let mut rest = tag;
    let mut matched_any = false;

    for (index, part) in pattern
        .split('*')
        .filter(|part| !part.is_empty())
        .enumerate()
    {
        matched_any = true;
        if index == 0 && !starts_with_wildcard {
            let Some(after) = rest.strip_prefix(part) else {
                return false;
            };
            rest = after;
            continue;
        }

        let Some((_, after)) = rest.split_once(part) else {
            return false;
        };
        rest = after;
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
