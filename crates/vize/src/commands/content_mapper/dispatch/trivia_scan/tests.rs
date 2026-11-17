//! The run scan against the byte-at-a-time scan it replaced.

use super::{Brace, Comment, Slash, VUE_JSX_REFERENCE, regex_end, slash_after, trivia};

type Runs = Vec<(usize, usize, Vec<Comment>)>;

/// Every run but a lone whitespace byte without a comment, which the
/// compactor leaves as it is, or `None` when the scan gave up.
fn rewritable_runs(
    scan: impl FnOnce(&mut dyn FnMut(usize, usize, &[Comment])) -> Option<()>,
) -> Option<Runs> {
    let mut runs = Vec::new();
    scan(&mut |start, end, comments| {
        if !(comments.is_empty() && end - start == 1) {
            runs.push((start, end, comments.to_vec()));
        }
    })?;
    Some(runs)
}

fn scanned(text: &str) -> Option<Runs> {
    rewritable_runs(|on_trivia| trivia(text, on_trivia))
}

fn reference(text: &str) -> Option<Runs> {
    rewritable_runs(|on_trivia| reference_trivia(text, on_trivia))
}

/// The scan before runs: one outer step per token, a predicate per byte.
fn reference_trivia(text: &str, on_trivia: &mut dyn FnMut(usize, usize, &[Comment])) -> Option<()> {
    if text.starts_with(VUE_JSX_REFERENCE) || text.starts_with("#!") {
        return None;
    }
    let bytes = text.as_bytes();
    let mut comments = Vec::new();
    let mut braces: Vec<Brace> = Vec::new();
    let mut at = 0;
    loop {
        let start = at;
        comments.clear();
        loop {
            match (bytes.get(at), bytes.get(at + 1)) {
                (Some(b' ' | b'\t' | b'\n' | b'\r'), _) => at += 1,
                (Some(b'/'), Some(b'/')) => {
                    let end = reference_line_comment_end(bytes, at)?;
                    comments.push(Comment {
                        start: at,
                        end,
                        line: true,
                    });
                    at = end;
                }
                (Some(b'/'), Some(b'*')) => {
                    let body = bytes.get(at + 2..)?;
                    let end = at + 2 + body.windows(2).position(|pair| pair == b"*/")? + 2;
                    comments.push(Comment {
                        start: at,
                        end,
                        line: false,
                    });
                    at = end;
                }
                _ => break,
            }
        }
        if at > start {
            on_trivia(start, at, &comments);
        }
        let Some(&byte) = bytes.get(at) else {
            return braces.is_empty().then_some(());
        };
        match byte {
            b'"' | b'\'' => at = reference_string_end(bytes, at)?,
            b'`' => at = reference_template_end(bytes, at + 1, &mut braces)?,
            b'/' => match slash_after(bytes, start.checked_sub(1)) {
                Slash::Regex => at = regex_end(bytes, at)?,
                Slash::Divide => at += 1,
                Slash::Unknown => return None,
            },
            b'{' => {
                braces.push(Brace::Plain);
                at += 1;
            }
            b'}' => match braces.pop()? {
                Brace::Plain => at += 1,
                Brace::Substitution => at = reference_template_end(bytes, at + 1, &mut braces)?,
            },
            b'<' if bytes.get(at + 1) == Some(&b'/') => return None,
            _ => at = skip_until(bytes, at + 1, is_special),
        }
    }
}

fn skip_until(bytes: &[u8], at: usize, stop: impl Fn(u8) -> bool) -> usize {
    bytes
        .get(at..)
        .and_then(|rest| rest.iter().position(|byte| stop(*byte)))
        .map_or(bytes.len(), |offset| at + offset)
}

fn is_special(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'"' | b'\'' | b'`' | b'{' | b'}' | b'<'
    )
}

fn reference_line_comment_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at;
    loop {
        let rest = bytes.get(index..)?;
        let Some(offset) = rest
            .iter()
            .position(|byte| matches!(byte, b'\n' | b'\r' | 0xE2))
        else {
            return Some(bytes.len());
        };
        index += offset;
        if bytes.get(index) != Some(&0xE2) {
            return Some(index);
        }
        if matches!(bytes.get(index + 1..index + 3), Some([0x80, 0xA8 | 0xA9])) {
            return None;
        }
        index += 1;
    }
}

fn reference_string_end(bytes: &[u8], at: usize) -> Option<usize> {
    let quote = *bytes.get(at)?;
    let mut index = at + 1;
    loop {
        index = skip_until(bytes, index, |byte| {
            byte == quote || matches!(byte, b'\\' | b'\n' | b'\r')
        });
        match *bytes.get(index)? {
            b'\\' => index += 2,
            b'\n' | b'\r' => return None,
            byte if byte == quote => return Some(index + 1),
            _ => index += 1,
        }
    }
}

fn reference_template_end(bytes: &[u8], mut at: usize, braces: &mut Vec<Brace>) -> Option<usize> {
    loop {
        at = skip_until(bytes, at, |byte| matches!(byte, b'\\' | b'`' | b'$'));
        match *bytes.get(at)? {
            b'\\' => at += 2,
            b'`' => return Some(at + 1),
            b'$' if bytes.get(at + 1) == Some(&b'{') => {
                braces.push(Brace::Substitution);
                return Some(at + 2);
            }
            _ => at += 1,
        }
    }
}

#[test]
fn lone_whitespace_between_code_bytes_is_not_handed_out() {
    let mut runs = Vec::new();
    assert_eq!(
        trivia("let a = b(c, d);", |start, end, _| runs.push((start, end))),
        Some(())
    );
    assert_eq!(runs, []);
}

#[test]
fn whitespace_before_a_slash_or_more_whitespace_is_handed_out() {
    for (text, expected) in [
        ("a  b", vec![(1, 3)]),
        ("a /x/", vec![(1, 2)]),
        ("a // c\nb", vec![(1, 7)]),
        ("a\n  b", vec![(1, 4)]),
        ("a /* k */ b", vec![(1, 10)]),
    ] {
        let mut runs = Vec::new();
        assert_eq!(
            trivia(text, |start, end, _| runs.push((start, end))),
            Some(()),
            "{text}"
        );
        assert_eq!(runs, expected, "{text}");
    }
}

#[test]
fn a_closing_tag_inside_a_run_still_ends_the_scan() {
    for text in ["x</div>", "x </div>", "a<//c\n", "f(a</b)"] {
        assert_eq!(scanned(text), None, "{text}");
        assert_eq!(reference(text), None, "{text}");
    }
    // `<` then whitespace then `/` is not a closing tag; both scans agree.
    assert_eq!(scanned("x < /y/.test(z)"), reference("x < /y/.test(z)"));
}

#[test]
fn the_run_scan_matches_the_byte_scan() {
    const PIECES: &[&str] = &[
        "a", "b1", "return", "typeof", "x.y", "(", ")", "[", "]", "{", "}", ";", ",", "=", "+",
        "++", "-", "--", "<", ">", "</", "/", "/x/g", "*", "\"s\"", "'q\\''", "\"n\n\"", "`t`",
        "`u${", "` ", " ", "  ", "\n", "\t", "\r\n", "// c\n", "//e", "/* k */", "/*", "#",
        "\u{2028}", "é", "$", "\\",
    ];
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % bound as u64) as usize
    };
    for round in 0..20_000 {
        let mut text = vize_l0::String::default();
        for _ in 0..next(24) + 1 {
            text.push_str(PIECES[next(PIECES.len())]);
        }
        assert_eq!(scanned(&text), reference(&text), "round {round}: {text:?}");
    }
}
