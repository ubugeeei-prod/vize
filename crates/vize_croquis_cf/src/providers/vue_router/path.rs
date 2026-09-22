//! Vue Router path syntax: params and nested-path joining.
//!
//! The one implementation of the path grammar the provider and Maestro's
//! editor helpers share: `:name`, an optional custom regexp `(…)` (nested
//! parentheses and `\` escapes skipped), then an optional modifier — `?`
//! optional, `+` repeatable, `*` both. A `\:` is a literal colon.

use vize_carton::CompactString;

/// One param a route path declares.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteParam {
    /// The param name.
    pub name: CompactString,
    /// `?` or `*`: the param may be absent.
    pub optional: bool,
    /// `+` or `*`: the param takes a list of segments.
    pub repeatable: bool,
}

impl RouteParam {
    /// The type Vue Router accepts for this param in a navigation
    /// (`RouteParamValueRaw` shaped by the modifiers).
    #[must_use]
    pub fn accepted_type(&self) -> &'static str {
        match (self.optional, self.repeatable) {
            (false, false) => "string | number",
            (true, false) => "string | number | null | undefined",
            (false, true) => "(string | number)[]",
            (true, true) => "(string | number)[] | null | undefined",
        }
    }
}

/// Every param `path` declares, in order, first occurrence of a name only.
#[must_use]
pub fn parse_route_params(path: &str) -> Vec<RouteParam> {
    let bytes = path.as_bytes();
    let mut params: Vec<RouteParam> = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative) = path[cursor..].find(':') {
        let colon = cursor + relative;
        if colon > 0 && bytes[colon - 1] == b'\\' {
            cursor = colon + 1;
            continue;
        }
        let name_start = colon + 1;
        let name_end = name_start
            + bytes[name_start..]
                .iter()
                .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_')
                .count();
        if name_end == name_start {
            cursor = name_start;
            continue;
        }
        let mut modifier_at = skip_custom_regexp(bytes, name_end);
        let modifier = bytes.get(modifier_at).copied();
        if matches!(modifier, Some(b'?' | b'+' | b'*')) {
            modifier_at += 1;
        }
        let name = &path[name_start..name_end];
        if !params.iter().any(|param| param.name == name) {
            params.push(RouteParam {
                name: CompactString::new(name),
                optional: matches!(modifier, Some(b'?' | b'*')),
                repeatable: matches!(modifier, Some(b'+' | b'*')),
            });
        }
        cursor = modifier_at;
    }
    params
}

fn skip_custom_regexp(bytes: &[u8], at: usize) -> usize {
    if bytes.get(at).copied() != Some(b'(') {
        return at;
    }
    let mut depth = 1usize;
    let mut pos = at + 1;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            b'(' => {
                depth += 1;
                pos += 1;
            }
            b')' => {
                depth -= 1;
                pos += 1;
                if depth == 0 {
                    return pos;
                }
            }
            _ => pos += 1,
        }
    }
    pos.min(bytes.len())
}

/// The full path of a child record: an absolute child path stands alone,
/// a relative one is appended to its parent's with one `/` (Vue Router's
/// `normalizeRouteRecord` join; an empty child path is its parent's path).
#[must_use]
pub fn join_route_path(parent: Option<&str>, child: &str) -> CompactString {
    let Some(parent) = parent else {
        return CompactString::new(child);
    };
    if child.starts_with('/') {
        return CompactString::new(child);
    }
    let mut joined = CompactString::new(parent);
    if !child.is_empty() {
        if !parent.ends_with('/') {
            joined.push('/');
        }
        joined.push_str(child);
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::{RouteParam, join_route_path, parse_route_params};
    use vize_carton::CompactString;

    /// `name`, then `?`/`+`/`*` for the modifiers, space-separated.
    fn shape(path: &str) -> CompactString {
        let mut out = CompactString::default();
        for param in parse_route_params(path) {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&param.name);
            match (param.optional, param.repeatable) {
                (true, true) => out.push('*'),
                (true, false) => out.push('?'),
                (false, true) => out.push('+'),
                (false, false) => {}
            }
        }
        out
    }

    #[test]
    fn modifiers_regexps_and_escapes_are_exact() {
        assert_eq!(
            shape("/articles/:id(\\d+)/:tab?/:chapters+/:pathMatch(.*)*"),
            "id tab? chapters+ pathMatch*"
        );
        assert_eq!(shape("/time/12\\:30/:at((a|b)(c))"), "at");
        assert_eq!(shape("/static/:/x"), "");
        assert_eq!(shape("/:id/:id"), "id");
    }

    #[test]
    fn joining_follows_vue_router_normalization() {
        assert_eq!(join_route_path(None, "/users"), "/users");
        assert_eq!(
            join_route_path(Some("/users/:id"), "posts"),
            "/users/:id/posts"
        );
        assert_eq!(join_route_path(Some("/users/:id"), ""), "/users/:id");
        assert_eq!(join_route_path(Some("/"), "about"), "/about");
        assert_eq!(join_route_path(Some("/users/:id"), "/admin"), "/admin");
    }

    #[test]
    fn accepted_types_follow_the_modifiers() {
        let types = parse_route_params("/:a/:b?/:c+/:d*")
            .iter()
            .map(RouteParam::accepted_type)
            .collect::<Vec<_>>();
        assert_eq!(
            types,
            [
                "string | number",
                "string | number | null | undefined",
                "(string | number)[]",
                "(string | number)[] | null | undefined",
            ]
        );
    }
}
