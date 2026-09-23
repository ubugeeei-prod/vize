use vize_croquis::Croquis;

pub(super) fn is_member_root_occurrence(summary: &Croquis, offset: u32, name: &str) -> bool {
    occurrence_tail(summary, offset, name).is_some_and(|tail| {
        tail.starts_with('.') || tail.starts_with("?.") || tail.starts_with('[')
    })
}

pub(super) fn is_call_root_occurrence(summary: &Croquis, offset: u32, name: &str) -> bool {
    occurrence_tail(summary, offset, name)
        .is_some_and(|tail| tail.starts_with('(') || tail.starts_with("?.("))
}

fn occurrence_tail<'a>(summary: &'a Croquis, offset: u32, name: &str) -> Option<&'a str> {
    for expr in &summary.template_expressions {
        if offset < expr.start {
            continue;
        }
        let local = (offset - expr.start) as usize;
        let source = expr.content.as_str();
        if local + name.len() > source.len() || source.get(local..local + name.len()) != Some(name)
        {
            continue;
        }
        return source.get(member_tail_start(source, local, name.len())..);
    }
    None
}

fn member_tail_start(source: &str, local: usize, name_len: usize) -> usize {
    let mut tail = skip_js_trivia_forward(source, local + name_len);
    let mut wrappers = parenthesized_wrapper_count(source, local);
    while wrappers > 0 && source.get(tail..).is_some_and(|rest| rest.starts_with(')')) {
        tail = skip_js_trivia_forward(source, tail + 1);
        wrappers -= 1;
    }
    tail
}

fn parenthesized_wrapper_count(source: &str, local: usize) -> usize {
    let mut end = local;
    let mut first_open = None;
    let mut count = 0;
    loop {
        end = skip_js_trivia_backward(source, end);
        if !source
            .get(..end)
            .is_some_and(|before| before.ends_with('('))
        {
            break;
        }
        end -= 1;
        first_open = Some(end);
        count += 1;
    }
    if first_open.is_some_and(|open| has_call_like_prefix(source, open)) {
        0
    } else {
        count
    }
}

fn has_call_like_prefix(source: &str, open: usize) -> bool {
    let prefix = skip_js_trivia_backward(source, open);
    let Some(ch) = source
        .get(..prefix)
        .and_then(|before| before.chars().next_back())
    else {
        return false;
    };
    ch == ')' || ch == ']' || ch == '\'' || ch == '"' || ch == '`' || is_identifier_part(ch)
}

fn is_identifier_part(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn skip_js_trivia_forward(source: &str, mut index: usize) -> usize {
    loop {
        let Some(rest) = source.get(index..) else {
            return index;
        };
        let trimmed = rest.trim_start();
        index += rest.len() - trimmed.len();
        if let Some(comment) = trimmed.strip_prefix("//") {
            index += 2 + comment.find('\n').unwrap_or(comment.len());
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix("/*") {
            let Some(end) = comment.find("*/") else {
                return source.len();
            };
            index += end + 4;
            continue;
        }
        return index;
    }
}

fn skip_js_trivia_backward(source: &str, mut end: usize) -> usize {
    loop {
        let Some(before) = source.get(..end) else {
            return end;
        };
        let trimmed = before.trim_end();
        end = trimmed.len();
        if let Some(start) = trimmed.strip_suffix("*/").and_then(|body| body.rfind("/*")) {
            end = start;
            continue;
        }
        return end;
    }
}
