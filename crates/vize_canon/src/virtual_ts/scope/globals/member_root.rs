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
        return Some(&source[member_tail_start(source, local, name.len())..]);
    }
    None
}

fn member_tail_start(source: &str, local: usize, name_len: usize) -> usize {
    let mut tail = skip_js_trivia_forward(source, local + name_len);
    let mut wrappers = parenthesized_wrapper_count(source, local);
    while wrappers > 0 && source[tail..].starts_with(')') {
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
        if end == 0 || !source[..end].ends_with('(') {
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
    let Some(ch) = source[..prefix].chars().next_back() else {
        return false;
    };
    ch == ')' || ch == ']' || ch == '\'' || ch == '"' || ch == '`' || is_identifier_part(ch)
}

fn is_identifier_part(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn skip_js_trivia_forward(source: &str, mut index: usize) -> usize {
    loop {
        while index < source.len() {
            let ch = source[index..].chars().next().unwrap();
            if !ch.is_whitespace() {
                break;
            }
            index += ch.len_utf8();
        }
        if source[index..].starts_with("//") {
            index += 2;
            while index < source.len() && source.as_bytes()[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if source[index..].starts_with("/*") {
            let Some(end) = source[index + 2..].find("*/") else {
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
        while end > 0 {
            let ch = source[..end].chars().next_back().unwrap();
            if !ch.is_whitespace() {
                break;
            }
            end -= ch.len_utf8();
        }
        if end >= 2
            && source[..end].ends_with("*/")
            && let Some(start) = source[..end - 2].rfind("/*")
        {
            end = start;
            continue;
        }
        return end;
    }
}
