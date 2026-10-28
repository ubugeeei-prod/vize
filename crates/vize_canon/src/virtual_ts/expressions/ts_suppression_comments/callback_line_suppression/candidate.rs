#[derive(Clone, Copy)]
pub(super) enum CandidateGate {
    Lexical,
    Preserve,
    Parse,
}

/// The allocation-free gate keeps the common single-arrow path lexical. A
/// direct function that needs no rewrite still returns `Preserve`, preventing
/// a nested body arrow from becoming the lexical fallback's target.
pub(super) fn candidate_gate(value: &str) -> CandidateGate {
    candidate(value).unwrap_or(CandidateGate::Preserve)
}

#[cfg(test)]
pub(super) fn has_callback_parser_candidate(value: &str) -> bool {
    matches!(candidate_gate(value), CandidateGate::Parse)
}

fn candidate(value: &str) -> Option<CandidateGate> {
    let bytes = value.as_bytes();
    let (mut index, mut quote, mut escaped) = (0usize, None, false);
    let (mut line_comment, mut block_comment) = (false, false);
    let (mut saw_line_break, mut saw_function, mut saw_generic) = (false, false, false);
    let (mut function_header, mut function_needs_parse) = (false, false);
    let (mut function_params, mut function_paren_depth) = (false, 0usize);
    let mut parenthesis_depth = 0usize;
    let mut separator_depth = None;
    let mut shallowest_arrow = None;
    let (mut shallowest_count, mut branch_prefix) = (0u8, false);
    let mut expression_start = true;
    while let Some(&current) = bytes.get(index) {
        let next = index.checked_add(1);
        saw_line_break |= matches!(current, b'\n' | b'\r');
        function_needs_parse |= function_header && matches!(current, b'\n' | b'\r');
        if line_comment {
            line_comment = !matches!(current, b'\n' | b'\r');
        } else if block_comment {
            if current == b'*' && next.and_then(|next| bytes.get(next)) == Some(&b'/') {
                block_comment = false;
                index = next?.checked_add(1)?;
                continue;
            }
        } else if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if current == b'\\' {
                escaped = true;
            } else if current == delimiter {
                quote = None;
            }
        } else if current == b'/' && next.and_then(|next| bytes.get(next)) == Some(&b'/') {
            line_comment = true;
            index = next?.checked_add(1)?;
            continue;
        } else if current == b'/' && next.and_then(|next| bytes.get(next)) == Some(&b'*') {
            block_comment = true;
            index = next?.checked_add(1)?;
            continue;
        } else if matches!(current, b'\'' | b'"' | b'`') {
            quote = Some(current);
            expression_start = false;
        } else if current == b'=' && next.and_then(|next| bytes.get(next)) == Some(&b'>') {
            match shallowest_arrow {
                None => {
                    shallowest_arrow = Some(parenthesis_depth);
                    shallowest_count = 1;
                    branch_prefix = separator_depth == Some(parenthesis_depth);
                }
                Some(depth) if parenthesis_depth < depth => {
                    shallowest_arrow = Some(parenthesis_depth);
                    shallowest_count = 1;
                    branch_prefix = separator_depth == Some(parenthesis_depth);
                }
                Some(depth) if parenthesis_depth == depth => {
                    shallowest_count = shallowest_count.saturating_add(1).min(2);
                    branch_prefix |= separator_depth == Some(depth);
                }
                Some(_) => {}
            }
            expression_start = true;
            index = next?.checked_add(1)?;
            continue;
        } else if current == b'<' && (expression_start || function_header) {
            saw_generic |= generic_head(bytes, next?).unwrap_or(false);
            expression_start = false;
        } else if current == b'_' || current == b'$' || current.is_ascii_alphabetic() {
            let end = identifier_end(bytes, index)?;
            let token = bytes.get(index..end)?;
            if expression_start && token == b"function" {
                saw_function = true;
                function_needs_parse |= saw_line_break;
                function_header = true;
                expression_start = false;
            } else if !(expression_start && token == b"async") {
                expression_start = false;
            }
            index = end;
            continue;
        } else if current.is_ascii_whitespace() {
            // Trivia keeps the current expression-start state.
        } else if current == b'(' {
            parenthesis_depth = parenthesis_depth.checked_add(1)?;
            if function_header {
                function_params = true;
                function_paren_depth = function_paren_depth.checked_add(1)?;
            }
            expression_start = true;
        } else if current == b')' {
            parenthesis_depth = parenthesis_depth.checked_sub(1)?;
            if function_header && function_params {
                function_paren_depth = function_paren_depth.checked_sub(1)?;
                if function_paren_depth == 0 {
                    function_header = false;
                }
            }
            expression_start = false;
        } else if matches!(current, b',' | b'?') {
            if separator_depth.is_none_or(|depth| parenthesis_depth < depth) {
                separator_depth = Some(parenthesis_depth);
            }
            expression_start = true;
        } else if current == b':' && separator_depth == Some(parenthesis_depth) {
            expression_start = true;
        } else {
            expression_start = false;
        }
        index = index.checked_add(1)?;
    }
    if line_comment || block_comment || quote.is_some() {
        return Some(CandidateGate::Preserve);
    }
    if !saw_line_break {
        return Some(CandidateGate::Lexical);
    }
    if saw_generic || function_needs_parse || (branch_prefix && shallowest_count > 1) {
        Some(CandidateGate::Parse)
    } else if saw_function {
        Some(CandidateGate::Preserve)
    } else {
        Some(CandidateGate::Lexical)
    }
}

fn generic_head(bytes: &[u8], start: usize) -> Option<bool> {
    let mut index = skip_trivia(bytes, start)?;
    if token_at(bytes, index, b"const") {
        index = skip_trivia(bytes, index.checked_add(5)?)?;
    }
    loop {
        index = skip_trivia(bytes, identifier_end(bytes, index)?)?;
        if token_at(bytes, index, b"extends") || bytes.get(index) == Some(&b'=') {
            return Some(true);
        }
        if bytes.get(index) == Some(&b'>') {
            return Some(bytes.get(skip_trivia(bytes, index.checked_add(1)?)?) == Some(&b'('));
        }
        if bytes.get(index) != Some(&b',') {
            return Some(false);
        }
        index = skip_trivia(bytes, index.checked_add(1)?)?;
        if bytes.get(index) == Some(&b'>') {
            return Some(bytes.get(skip_trivia(bytes, index.checked_add(1)?)?) == Some(&b'('));
        }
        if token_at(bytes, index, b"const") {
            index = skip_trivia(bytes, index.checked_add(5)?)?;
        }
    }
}

fn identifier_end(bytes: &[u8], start: usize) -> Option<usize> {
    let first = *bytes.get(start)?;
    if !(matches!(first, b'_' | b'$') || first.is_ascii_alphabetic() || first >= 0x80) {
        return None;
    }
    let mut index = start.checked_add(1)?;
    while bytes.get(index).is_some_and(|byte| {
        matches!(*byte, b'_' | b'$') || byte.is_ascii_alphanumeric() || *byte >= 0x80
    }) {
        index = index.checked_add(1)?;
    }
    Some(index)
}

fn token_at(bytes: &[u8], start: usize, token: &[u8]) -> bool {
    let Some(end) = start.checked_add(token.len()) else {
        return false;
    };
    bytes.get(start..end) == Some(token)
        && bytes.get(end).is_none_or(|byte| {
            !matches!(*byte, b'_' | b'$') && !byte.is_ascii_alphanumeric() && *byte < 0x80
        })
}

fn skip_trivia(bytes: &[u8], mut index: usize) -> Option<usize> {
    loop {
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            index = index.checked_add(1)?;
        }
        let next = index.checked_add(1)?;
        match (bytes.get(index), bytes.get(next)) {
            (Some(&b'/'), Some(&b'/')) => {
                index = next.checked_add(1)?;
                while bytes
                    .get(index)
                    .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
                {
                    index = index.checked_add(1)?;
                }
            }
            (Some(&b'/'), Some(&b'*')) => {
                index = next.checked_add(1)?;
                loop {
                    let comment_next = index.checked_add(1)?;
                    match (bytes.get(index), bytes.get(comment_next)) {
                        (Some(&b'*'), Some(&b'/')) => {
                            index = comment_next.checked_add(1)?;
                            break;
                        }
                        (Some(_), Some(_)) => index = index.checked_add(1)?,
                        _ => return None,
                    }
                }
            }
            _ => return Some(index),
        }
    }
}
