//! Entry-line parsing for the `[remarks]` page.
//!
//! Strict everywhere the printer has one spelling: identifiers are
//! lowercase kebab-case (the pipeline grammar's `ident`), exactly one space
//! separates fields, a span is `@{start}:{end}` with `start <= end`, and a
//! repeated argument key is rejected. The one leniency is a string value's
//! escapes: any `\uXXXX` scalar is accepted and prints back as the canonical
//! character, the JSON rule.

use alloc::vec::Vec;

use vize_s0::{Span, String, cstr};

use crate::folio::FolioError;
use crate::pass::observer::{RecordedArg, RecordedRemark, RemarkArgValue, RemarkKind};

/// Parse one entry line.
pub(super) fn entry(line: &str, line_no: usize) -> Result<RecordedRemark, FolioError> {
    let fail = |message: String| FolioError::new(line_no, message);
    let (head, rest) = take_token(line);
    let (stage, pass) = head
        .split_once('.')
        .ok_or_else(|| fail(cstr!("remark origin `{head}` is not `stage.pass`")))?;
    ident(stage, "stage", line_no)?;
    ident(pass, "pass", line_no)?;
    let (kind_text, rest) = take_token(rest);
    let kind = RemarkKind::from_name(kind_text).ok_or_else(|| {
        fail(cstr!(
            "unknown remark kind `{kind_text}` (expected applied, missed, analysis)"
        ))
    })?;
    let (name, rest) = take_token(rest);
    ident(name, "remark name", line_no)?;
    let (span_text, mut rest) = take_token(rest);
    let span = parse_span(span_text).ok_or_else(|| {
        fail(cstr!(
            "remark span `{span_text}` is not `@start:end` with start <= end"
        ))
    })?;
    let mut args: Vec<RecordedArg> = Vec::new();
    while !rest.is_empty() {
        let (key, after_key) = rest
            .split_once('=')
            .ok_or_else(|| fail(cstr!("remark argument `{rest}` is missing `=`")))?;
        ident(key, "argument key", line_no)?;
        if args.iter().any(|arg| arg.key.as_str() == key) {
            return Err(fail(cstr!("duplicate remark argument `{key}`")));
        }
        let (value, after_value) = parse_value(after_key, line_no)?;
        args.push(RecordedArg {
            key: String::from(key),
            value,
        });
        rest = after_value;
    }
    Ok(RecordedRemark {
        stage: String::from(stage),
        pass: String::from(pass),
        kind,
        name: String::from(name),
        span,
        args,
    })
}

/// Split at the first space: `(token, rest after the space)`. The rest is
/// empty when there is no space.
fn take_token(text: &str) -> (&str, &str) {
    text.split_once(' ').unwrap_or((text, ""))
}

/// Reject anything but the pipeline grammar's `ident`.
fn ident(text: &str, what: &str, line_no: usize) -> Result<(), FolioError> {
    let bytes = text.as_bytes();
    let well_formed = !bytes.is_empty()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        && bytes[0] != b'-'
        && bytes[bytes.len() - 1] != b'-';
    if well_formed {
        Ok(())
    } else {
        Err(FolioError::new(
            line_no,
            cstr!("{what} `{text}` is not a lowercase kebab-case identifier"),
        ))
    }
}

fn parse_span(text: &str) -> Option<Span> {
    let (start, end) = text.strip_prefix('@')?.split_once(':')?;
    let start = parse_u32(start)?;
    let end = parse_u32(end)?;
    (start <= end).then(|| Span::new(start, end))
}

/// Canonical decimal only: no sign, no leading zero (except `0` itself).
fn parse_u32(text: &str) -> Option<u32> {
    let canonical = !text.is_empty()
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'));
    canonical.then(|| text.parse().ok()).flatten()
}

/// Parse one value; returns it plus the text after its separating space.
fn parse_value(text: &str, line_no: usize) -> Result<(RemarkArgValue, &str), FolioError> {
    if text.starts_with('"') {
        let (value, consumed) = parse_string(text, line_no)?;
        let rest = &text[consumed..];
        return match rest.strip_prefix(' ') {
            Some(after) => Ok((RemarkArgValue::Str(value), after)),
            None if rest.is_empty() => Ok((RemarkArgValue::Str(value), rest)),
            None => Err(FolioError::new(
                line_no,
                cstr!("unexpected `{rest}` after a string value"),
            )),
        };
    }
    let (token, rest) = take_token(text);
    let value = match token {
        "true" => RemarkArgValue::Bool(true),
        "false" => RemarkArgValue::Bool(false),
        number => {
            let digits = number.strip_prefix('-').unwrap_or(number);
            let canonical = !digits.is_empty()
                && digits.bytes().all(|byte| byte.is_ascii_digit())
                && (digits == "0" || !digits.starts_with('0'))
                && number != "-0";
            let parsed = canonical.then(|| number.parse::<i64>().ok()).flatten();
            RemarkArgValue::Int(parsed.ok_or_else(|| {
                FolioError::new(
                    line_no,
                    cstr!("remark value `{number}` is not a string, integer, or boolean"),
                )
            })?)
        }
    };
    Ok((value, rest))
}

/// Parse a JSON string literal at the start of `text`; returns the decoded
/// value and the byte length consumed (quotes included).
pub(super) fn parse_string(text: &str, line_no: usize) -> Result<(String, usize), FolioError> {
    let fail = |message: &str| FolioError::new(line_no, String::from(message));
    let mut out = String::default();
    let mut chars = text.char_indices().skip(1);
    while let Some((index, character)) = chars.next() {
        match character {
            '"' => return Ok((out, index + 1)),
            '\\' => {
                let Some((_, escape)) = chars.next() else {
                    return Err(fail("unterminated escape in a string value"));
                };
                match escape {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let digit = chars
                                .next()
                                .and_then(|(_, hex)| hex.to_digit(16))
                                .ok_or_else(|| fail("`\\u` needs four hex digits"))?;
                            code = code * 16 + digit;
                        }
                        let decoded = char::from_u32(code)
                            .ok_or_else(|| fail("`\\u` escape is not a scalar value"))?;
                        out.push(decoded);
                    }
                    _ => return Err(fail("unknown escape in a string value")),
                }
            }
            control if control < ' ' => {
                return Err(fail("raw control character in a string value"));
            }
            other => out.push(other),
        }
    }
    Err(fail("unterminated string value"))
}
