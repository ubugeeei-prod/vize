//! JSON-RPC reply frames.

use std::io::{self, Write};

use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use serde_json::{Value, json};
use vize_canon::{
    ContentMapperDiagnostic, ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives,
    ContentMapperSemanticLink, ContentMapperSpan, ContentMapperTransform,
    ContentMapperUnusedExpectDiagnostic,
};

/// Room for `Content-Length: <u64>\r\n\r\n` ahead of the body.
const HEADER_ROOM: usize = 40;

/// TypeScript `spanmap.FeatureAll`; a five-element span tuple decodes to it.
const SPAN_FEATURES_ALL: usize = (1 << 20) - 1;

/// Ignore directive policy; the host reads a tuple's sixth element only for expect.
const DIRECTIVE_POLICY_IGNORE: usize = 0;

pub(super) fn encode_result(id: &Value, result: impl Serialize) -> io::Result<Vec<u8>> {
    // The original server's json! envelope recursively ordered every object
    // through Value. Preserve opaque nested fields as well as envelope keys.
    // Consume the original packet so its large text/maps are dropped by
    // to_value before the escaped wire buffer is allocated.
    let result = serde_json::to_value(result).map_err(io::Error::other)?;
    #[derive(Serialize)]
    struct Envelope<'a> {
        id: &'a Value,
        jsonrpc: &'static str,
        result: &'a Value,
    }
    encode_frame(&Envelope {
        id,
        jsonrpc: "2.0",
        result: &result,
    })
}

/// [`encode_result`] for a transform, without copying its text and mappings
/// into a `Value` tree first. The sorted keys `Value` produced are spelled out
/// here; exhaustive patterns make a new protocol field a compile error.
pub(super) fn encode_transform(
    id: &Value,
    transform: &ContentMapperTransform,
) -> io::Result<Vec<u8>> {
    let ContentMapperTransform {
        text,
        extension,
        mappings,
        semantic_links,
        diagnostic_directives,
        diagnostics,
    } = transform;
    // Reserve the escaped text and mapping rows once instead of doubling a
    // buffer the size of a large module.
    let capacity = text.len() + text.len() / 8 + mappings.len() * 64;
    let mut frame = Vec::with_capacity(HEADER_ROOM + capacity);
    frame.resize(HEADER_ROOM, 0);
    frame.extend_from_slice(br#"{"id":"#);
    serde_json::to_writer(&mut frame, id).map_err(io::Error::other)?;
    frame.extend_from_slice(br#","jsonrpc":"2.0","result":{"#);
    if let Some(directives) = diagnostic_directives {
        frame.extend_from_slice(br#""diagnosticDirectives":"#);
        push_directives(&mut frame, directives)?;
        frame.push(b',');
    }
    frame.extend_from_slice(br#""diagnostics":"#);
    serde_json::to_writer(&mut frame, &SortedSeq(diagnostics)).map_err(io::Error::other)?;
    frame.extend_from_slice(br#","extension":"#);
    push_json_str(&mut frame, extension);
    frame.extend_from_slice(br#","mappings":"#);
    push_spans(&mut frame, mappings);
    if !semantic_links.is_empty() {
        frame.extend_from_slice(br#","semanticLinks":"#);
        serde_json::to_writer(&mut frame, &SortedSeq(semantic_links)).map_err(io::Error::other)?;
    }
    frame.extend_from_slice(br#","text":"#);
    push_json_str(&mut frame, text);
    frame.extend_from_slice(b"}}");
    finish_frame(frame)
}

/// `serde_json`'s escaping, eight bytes per step: projections are long
/// plain runs between newlines, which serde scans and copies byte by byte.
fn push_json_str(out: &mut Vec<u8>, text: &str) {
    const ONES: u64 = u64::from_ne_bytes([1; 8]);
    const HIGHS: u64 = u64::from_ne_bytes([0x80; 8]);
    let bytes = text.as_bytes();
    out.reserve(bytes.len() + 2);
    out.push(b'"');
    let mut run = 0;
    let mut index = 0;
    while let Some(chunk) = bytes.get(index..).and_then(<[u8]>::first_chunk::<8>) {
        let word = u64::from_le_bytes(*chunk);
        let quote = word ^ (ONES * u64::from(b'"'));
        let backslash = word ^ (ONES * u64::from(b'\\'));
        // Exact per word, and the lowest flagged byte is the first special one.
        let special = (word.wrapping_sub(ONES * 0x20) & !word
            | quote.wrapping_sub(ONES) & !quote
            | backslash.wrapping_sub(ONES) & !backslash)
            & HIGHS;
        if special == 0 {
            index += 8;
            continue;
        }
        index += (special.trailing_zeros() / 8) as usize;
        push_escaped(out, bytes, &mut run, index);
        index += 1;
    }
    while index < bytes.len() {
        push_escaped(out, bytes, &mut run, index);
        index += 1;
    }
    out.extend_from_slice(bytes.get(run..).unwrap_or_default());
    out.push(b'"');
}

/// Flushes the plain run before `index` and escapes the byte there, if it
/// needs escaping.
fn push_escaped(out: &mut Vec<u8>, bytes: &[u8], run: &mut usize, index: usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let Some(&byte) = bytes.get(index) else {
        return;
    };
    let short: &[u8] = match byte {
        b'"' => br#"\""#,
        b'\\' => br"\\",
        b'\n' => br"\n",
        b'\r' => br"\r",
        b'\t' => br"\t",
        0x08 => br"\b",
        0x0c => br"\f",
        0x00..=0x1f => &[],
        _ => return,
    };
    out.extend_from_slice(bytes.get(*run..index).unwrap_or_default());
    *run = index + 1;
    if short.is_empty() {
        let digit = |nibble: u8| HEX.get(usize::from(nibble)).copied().unwrap_or(b'0');
        out.extend_from_slice(&[
            b'\\',
            b'u',
            b'0',
            b'0',
            digit(byte >> 4),
            digit(byte & 0x0f),
        ]);
    } else {
        out.extend_from_slice(short);
    }
}

/// `[[a,b,c,d,e,f],...]` without a serializer call per number. A span with
/// every feature drops its mask, as `spanmap.Marshal` writes it.
fn push_spans(out: &mut Vec<u8>, spans: &[ContentMapperSpan]) {
    out.push(b'[');
    for (row, ContentMapperSpan(span)) in spans.iter().enumerate() {
        if row > 0 {
            out.push(b',');
        }
        let width = if span[5] == SPAN_FEATURES_ALL { 5 } else { 6 };
        push_row(out, span.get(..width).unwrap_or(span));
    }
    out.push(b']');
}

/// Directive table with sorted keys. An ignore tuple drops its unused-expect
/// index, which the host never reads for that policy.
fn push_directives(
    out: &mut Vec<u8>,
    directives: &ContentMapperDiagnosticDirectives,
) -> io::Result<()> {
    let ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics,
        directives,
    } = directives;
    out.extend_from_slice(br#"{"directives":["#);
    for (row, ContentMapperDiagnosticDirective(tuple)) in directives.iter().enumerate() {
        if row > 0 {
            out.push(b',');
        }
        let width = if tuple[4] == DIRECTIVE_POLICY_IGNORE {
            5
        } else {
            6
        };
        push_row(out, tuple.get(..width).unwrap_or(tuple));
    }
    out.extend_from_slice(br#"],"unusedExpectDirectiveDiagnostics":"#);
    serde_json::to_writer(&mut *out, &SortedSeq(unused_expect_directive_diagnostics))
        .map_err(io::Error::other)?;
    out.push(b'}');
    Ok(())
}

fn push_row(out: &mut Vec<u8>, row: &[usize]) {
    out.push(b'[');
    for (column, &value) in row.iter().enumerate() {
        if column > 0 {
            out.push(b',');
        }
        push_usize(out, value);
    }
    out.push(b']');
}

fn push_usize(out: &mut Vec<u8>, mut value: usize) {
    let mut digits = [0u8; 20];
    let mut start = digits.len();
    for slot in digits.iter_mut().rev() {
        *slot = b'0' + (value % 10) as u8;
        start -= 1;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    out.extend_from_slice(digits.get(start..).unwrap_or_default());
}

pub(super) fn encode_error(id: &Value, code: i32, message: &str) -> io::Result<Vec<u8>> {
    encode_frame(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message},
    }))
}

/// Serialize the body once, behind reserved header room, instead of copying
/// it into a second frame buffer.
pub(super) fn encode_frame(value: &impl Serialize) -> io::Result<Vec<u8>> {
    encode_frame_with_capacity(value, 0)
}

fn encode_frame_with_capacity(value: &impl Serialize, capacity: usize) -> io::Result<Vec<u8>> {
    let mut frame = Vec::with_capacity(HEADER_ROOM + capacity);
    frame.resize(HEADER_ROOM, 0);
    serde_json::to_writer(&mut frame, value).map_err(io::Error::other)?;
    finish_frame(frame)
}

/// Writes the header into the room reserved ahead of the body.
fn finish_frame(mut frame: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut header = Vec::with_capacity(HEADER_ROOM);
    write!(
        &mut header,
        "Content-Length: {}\r\n\r\n",
        frame.len() - HEADER_ROOM
    )?;
    let start = HEADER_ROOM - header.len();
    frame
        .get_mut(start..HEADER_ROOM)
        .ok_or_else(|| io::Error::other("content-mapper frame header overflow"))?
        .copy_from_slice(&header);
    frame.drain(..start);
    Ok(frame)
}

/// A protocol value serialized with lexicographically sorted keys.
struct Sorted<'a, T>(&'a T);

struct SortedSeq<'a, T>(&'a [T]);

impl<'a, T> Serialize for SortedSeq<'a, T>
where
    Sorted<'a, T>: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(Sorted))
    }
}

impl Serialize for Sorted<'_, ContentMapperDiagnostic> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ContentMapperDiagnostic {
            message_text,
            start,
            length,
            code,
        } = self.0;
        let mut out = serializer.serialize_struct("ContentMapperDiagnostic", 4)?;
        out.serialize_field("code", code)?;
        out.serialize_field("length", length)?;
        out.serialize_field("messageText", message_text)?;
        out.serialize_field("start", start)?;
        out.end()
    }
}

impl Serialize for Sorted<'_, ContentMapperSemanticLink> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ContentMapperSemanticLink {
            source_start,
            source_length,
            target_start,
            target_length,
            kind,
        } = self.0;
        let mut out = serializer.serialize_struct("ContentMapperSemanticLink", 5)?;
        out.serialize_field("kind", kind)?;
        out.serialize_field("sourceLength", source_length)?;
        out.serialize_field("sourceStart", source_start)?;
        out.serialize_field("targetLength", target_length)?;
        out.serialize_field("targetStart", target_start)?;
        out.end()
    }
}

impl Serialize for Sorted<'_, ContentMapperUnusedExpectDiagnostic> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ContentMapperUnusedExpectDiagnostic { code, message_text } = self.0;
        let mut out = serializer.serialize_struct("ContentMapperUnusedExpectDiagnostic", 2)?;
        out.serialize_field("code", code)?;
        out.serialize_field("messageText", message_text)?;
        out.end()
    }
}

/// A transform as its reply decodes: serde's value with the five-element
/// tuples TypeScript reads as `FeatureAll` spans and ignore directives.
#[cfg(test)]
pub(in crate::commands::content_mapper) fn wire_value(transform: &ContentMapperTransform) -> Value {
    let mut value = serde_json::to_value(transform).unwrap_or_default();
    let narrow = |rows: Option<&mut Vec<Value>>, column: usize, compact: usize| {
        for row in rows.into_iter().flatten() {
            if let Some(tuple) = row.as_array_mut()
                && tuple.get(column) == Some(&json!(compact))
            {
                tuple.truncate(5);
            }
        }
    };
    narrow(
        value.get_mut("mappings").and_then(Value::as_array_mut),
        5,
        SPAN_FEATURES_ALL,
    );
    narrow(
        value
            .get_mut("diagnosticDirectives")
            .and_then(|table| table.get_mut("directives"))
            .and_then(Value::as_array_mut),
        4,
        DIRECTIVE_POLICY_IGNORE,
    );
    value
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vize_canon::{
        ContentMapperDiagnostic, ContentMapperDiagnosticDirective,
        ContentMapperDiagnosticDirectives, ContentMapperSemanticLink, ContentMapperSpan,
        ContentMapperTransform, ContentMapperUnusedExpectDiagnostic,
    };

    use super::{encode_error, encode_result, encode_transform};

    fn transform(full: bool) -> ContentMapperTransform {
        ContentMapperTransform {
            text: "const a = \"雪\\n\";\u{2028}🌸\t\r\u{1}\u{8}\u{c}\u{1f}\u{7f}\n".into(),
            extension: ".tsx",
            mappings: vec![
                ContentMapperSpan([0, 5, 10, 5, 0, 7]),
                ContentMapperSpan([5, 1, 15, 1, 1, super::SPAN_FEATURES_ALL]),
                ContentMapperSpan([6, 2, 16, 2, 0, 0]),
            ],
            semantic_links: if full {
                vec![ContentMapperSemanticLink {
                    source_start: 1,
                    source_length: 2,
                    target_start: 3,
                    target_length: 4,
                    kind: "vueSetupTemplateRefUnwrap",
                }]
            } else {
                Vec::new()
            },
            diagnostic_directives: full.then(|| ContentMapperDiagnosticDirectives {
                unused_expect_directive_diagnostics: vec![ContentMapperUnusedExpectDiagnostic {
                    code: 2578,
                    message_text: "Unused '@ts-expect-error' directive.".into(),
                }],
                directives: vec![
                    ContentMapperDiagnosticDirective([0, 0, 0, 1, 0, 0]),
                    ContentMapperDiagnosticDirective([1, 2, 3, 4, 1, 0]),
                ],
            }),
            diagnostics: vec![ContentMapperDiagnostic {
                message_text: "bad \"value\"".into(),
                start: 9,
                length: 2,
                code: 100_002,
            }],
        }
    }

    #[test]
    fn transform_frames_match_the_sorted_value_envelope() {
        for full in [true, false] {
            for id in [json!(7), json!("req-1"), json!(null)] {
                assert_eq!(
                    encode_transform(&id, &transform(full)).unwrap(),
                    encode_result(&id, super::wire_value(&transform(full))).unwrap(),
                );
            }
        }
    }

    #[test]
    fn every_feature_spans_and_ignore_directives_drop_their_sixth_element() {
        let frame = encode_transform(&json!(1), &transform(true)).unwrap();
        let text = std::str::from_utf8(&frame).unwrap();
        let (_, body) = text.split_once("\r\n\r\n").unwrap();
        let value: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            value["result"]["mappings"],
            json!([[0, 5, 10, 5, 0, 7], [5, 1, 15, 1, 1], [6, 2, 16, 2, 0, 0]])
        );
        assert_eq!(
            value["result"]["diagnosticDirectives"]["directives"],
            json!([[0, 0, 0, 1, 0], [1, 2, 3, 4, 1, 0]])
        );
    }

    #[test]
    fn json_strings_match_serde_json() {
        let alphabet: Vec<char> = (0u8..0x20)
            .map(char::from)
            .chain([
                '"', '\\', '/', '\u{7f}', 'a', ' ', 'é', '雪', '🌸', '\u{2028}',
            ])
            .collect();
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for length in 0..48 {
            for _ in 0..64 {
                let text: vize_l0::String = (0..length)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        let plain = !state.is_multiple_of(4);
                        if plain {
                            'x'
                        } else {
                            alphabet[(state >> 8) as usize % alphabet.len()]
                        }
                    })
                    .collect();
                let mut out = Vec::new();
                super::push_json_str(&mut out, &text);
                assert_eq!(
                    std::str::from_utf8(&out).unwrap(),
                    serde_json::to_string(&text).unwrap()
                );
            }
        }
    }

    #[test]
    fn spans_match_serde_json() {
        let spans = vec![
            ContentMapperSpan([0, 9, 10, 99, 100, usize::MAX]),
            ContentMapperSpan([1, 12_345, 7, 0, 65_536, 4_294_967_296]),
        ];
        let mut out = Vec::new();
        super::push_spans(&mut out, &spans);
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            serde_json::to_string(&spans).unwrap()
        );
        let mut empty = Vec::new();
        super::push_spans(&mut empty, &[]);
        assert_eq!(empty, b"[]");
    }

    #[test]
    fn frames_carry_the_exact_body_length() {
        let frame = encode_error(&json!(1), -32603, "雪").unwrap();
        let text = std::str::from_utf8(&frame).unwrap();
        let (header, body) = text.split_once("\r\n\r\n").unwrap();
        assert_eq!(header, vize_l0::cstr!("Content-Length: {}", body.len()));
        assert_eq!(
            body,
            r#"{"error":{"code":-32603,"message":"雪"},"id":1,"jsonrpc":"2.0"}"#
        );
    }
}
