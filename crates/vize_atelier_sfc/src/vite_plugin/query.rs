/// Borrowed view of a Vite module ID split at the first query marker.
pub struct SplitRequest<'a> {
    /// The path segment before `?`.
    pub path: &'a str,
    /// The query segment without the leading `?`.
    pub query: &'a str,
    /// The original query suffix including `?`, or an empty string.
    pub query_suffix: &'a str,
}

/// Splits a Vite module ID without allocating.
pub fn split_request(id: &str) -> SplitRequest<'_> {
    if let Some(query_start) = id.find('?') {
        SplitRequest {
            path: &id[..query_start],
            query: &id[query_start + 1..],
            query_suffix: &id[query_start..],
        }
    } else {
        SplitRequest {
            path: id,
            query: "",
            query_suffix: "",
        }
    }
}

/// Returns true when the raw query contains `key` as a standalone key.
pub fn query_has_key(query: &str, key: &str) -> bool {
    query.split('&').any(|part| query_part_key(part) == key)
}

/// Returns the raw value for `key` in a Vite query string.
pub fn query_value<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query.split('&').find_map(|part| {
        let (part_key, value) = part.split_once('=').unwrap_or((part, ""));
        (part_key == key).then_some(value)
    })
}

/// Returns true when `key` exists and exactly matches `expected`.
pub fn query_value_is(query: &str, key: &str, expected: &str) -> bool {
    query_value(query, key).is_some_and(|value| value == expected)
}

/// Parses an unsigned style block index.
pub fn parse_u32(value: &str) -> Option<u32> {
    value.parse().ok()
}

fn query_part_key(part: &str) -> &str {
    part.split_once('=').map_or(part, |(key, _)| key)
}
