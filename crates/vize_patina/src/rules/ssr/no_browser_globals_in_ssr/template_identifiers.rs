use super::{NoBrowserGlobalsInSsr, type_ranges};

impl NoBrowserGlobalsInSsr {
    /// Extract identifiers from an expression string.
    ///
    /// This method is aware of JavaScript syntax to avoid false positives:
    /// - Skips content inside string literals ('...', "...", `...`)
    /// - Skips content inside comments and regex literals (`/window/`)
    /// - Skips property access after `.` (e.g., `obj.top` → only `obj`)
    /// - Skips object property keys (e.g., `{ top: 0 }` → skips `top`)
    /// - Skips direct `typeof window` guards that are safe in SSR
    fn extract_identifiers(expr: &str) -> Vec<(&str, usize)> {
        let mut identifiers = Vec::new();
        let bytes = expr.as_bytes();
        let len = bytes.len();
        let at = |index: usize| bytes.get(index).copied();
        let mut i = 0;
        // Track whether the previous token was a `.` (property access)
        let mut after_dot = false;
        let mut after_typeof = false;
        let mut can_start_regex = true;

        while let Some(b) = at(i) {
            // Skip comments without changing the surrounding expression state.
            if b == b'/' && at(i + 1) == Some(b'/') {
                i += 2;
                while at(i).is_some_and(|c| !matches!(c, b'\n' | b'\r')) {
                    i += 1;
                }
                continue;
            }
            if b == b'/' && at(i + 1) == Some(b'*') {
                i += 2;
                while i + 1 < len {
                    if at(i) == Some(b'*') && at(i + 1) == Some(b'/') {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            if b == b'/' && can_start_regex {
                i += 1;
                let mut in_character_class = false;
                while let Some(c) = at(i) {
                    if c == b'\\' {
                        i += 2;
                        continue;
                    }
                    if c == b'[' {
                        in_character_class = true;
                        i += 1;
                        continue;
                    }
                    if c == b']' {
                        in_character_class = false;
                        i += 1;
                        continue;
                    }
                    if c == b'/' && !in_character_class {
                        i += 1;
                        while at(i).is_some_and(|c| c.is_ascii_alphabetic()) {
                            i += 1;
                        }
                        break;
                    }
                    i += 1;
                }
                after_dot = false;
                after_typeof = false;
                can_start_regex = false;
                continue;
            }

            // Skip string literals
            if b == b'\'' || b == b'"' || b == b'`' {
                after_typeof = false;
                let quote = b;
                i += 1;
                while let Some(c) = at(i) {
                    if c == b'\\' {
                        i += 2; // skip escaped character
                        continue;
                    }
                    if c == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                after_dot = false;
                can_start_regex = false;
                continue;
            }

            // Track dot for property access
            if b == b'.' {
                after_dot = true;
                can_start_regex = false;
                i += 1;
                continue;
            }

            // Identifier start
            if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
                let start = i;
                i += 1;
                while at(i).is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'$') {
                    i += 1;
                }
                let ident = expr.get(start..i).unwrap_or_default();

                // Skip if it's a property access (after `.`)
                if after_dot {
                    after_dot = false;
                    after_typeof = false;
                    can_start_regex = false;
                    continue;
                }

                // Skip if it's an object property key (identifier followed by `:`)
                // Look ahead past whitespace for `:`
                let mut j = i;
                while at(j).is_some_and(|c| c.is_ascii_whitespace()) {
                    j += 1;
                }
                if at(j) == Some(b':') && at(j + 1) != Some(b':') {
                    // This is an object key like `{ top: 0 }`, skip it
                    after_dot = false;
                    after_typeof = false;
                    can_start_regex = true;
                    continue;
                }

                if ident == "typeof" {
                    after_typeof = true;
                    after_dot = false;
                    can_start_regex = true;
                    continue;
                }

                if after_typeof {
                    after_typeof = false;
                    let mut next = i;
                    while at(next).is_some_and(|c| c.is_ascii_whitespace()) {
                        next += 1;
                    }
                    if !matches!(at(next), Some(b'.' | b'[')) {
                        after_dot = false;
                        can_start_regex = false;
                        continue;
                    }
                }

                identifiers.push((ident, start));
                after_dot = false;
                can_start_regex = false;
                continue;
            }

            // Skip digits (number literals)
            if b.is_ascii_digit() {
                after_typeof = false;
                i += 1;
                while at(i).is_some_and(|c| c.is_ascii_alphanumeric() || c == b'.') {
                    i += 1;
                }
                after_dot = false;
                can_start_regex = false;
                continue;
            }

            // Any other character
            if !b.is_ascii_whitespace() {
                after_dot = false;
                if !matches!(b, b'(' | b')') {
                    after_typeof = false;
                }
                can_start_regex = matches!(
                    b,
                    b'(' | b'['
                        | b'{'
                        | b','
                        | b':'
                        | b';'
                        | b'?'
                        | b'='
                        | b'!'
                        | b'&'
                        | b'|'
                        | b'+'
                        | b'-'
                        | b'*'
                        | b'%'
                        | b'~'
                        | b'^'
                        | b'<'
                        | b'>'
                );
            }
            i += 1;
        }

        identifiers
    }

    fn runtime_identifiers(expr: &str) -> Vec<&str> {
        let identifiers = Self::extract_identifiers(expr);
        if !identifiers
            .iter()
            .any(|(name, _)| Self::is_browser_global_static(name))
        {
            return identifiers.into_iter().map(|(name, _)| name).collect();
        }

        let type_ranges = type_ranges::type_ranges(expr);
        identifiers
            .into_iter()
            .filter(|(_, offset)| {
                !type_ranges
                    .iter()
                    .any(|(start, end)| *offset >= *start && *offset < *end)
            })
            .map(|(name, _)| name)
            .collect()
    }
}
