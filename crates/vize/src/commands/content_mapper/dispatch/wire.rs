//! JSON-RPC reply frames.

use std::io::{self, Write};

use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};
use serde_json::{Value, json};
use vize_canon::{
    ContentMapperDiagnostic, ContentMapperDiagnosticDirectives, ContentMapperSemanticLink,
    ContentMapperTransform, ContentMapperUnusedExpectDiagnostic,
};

/// Room for `Content-Length: <u64>\r\n\r\n` ahead of the body.
const HEADER_ROOM: usize = 40;

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
    #[derive(Serialize)]
    struct Envelope<'a> {
        id: &'a Value,
        jsonrpc: &'static str,
        result: Sorted<'a, ContentMapperTransform>,
    }
    // Reserve the escaped text and mapping rows once instead of doubling a
    // buffer the size of a large module.
    let capacity = transform.text.len() + transform.text.len() / 8 + transform.mappings.len() * 64;
    encode_frame_with_capacity(
        &Envelope {
            id,
            jsonrpc: "2.0",
            result: Sorted(transform),
        },
        capacity,
    )
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

impl Serialize for Sorted<'_, ContentMapperTransform> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ContentMapperTransform {
            text,
            extension,
            mappings,
            semantic_links,
            diagnostic_directives,
            diagnostics,
        } = self.0;
        let mut out = serializer.serialize_struct("ContentMapperTransform", 6)?;
        if let Some(directives) = diagnostic_directives {
            out.serialize_field("diagnosticDirectives", &Sorted(directives))?;
        }
        out.serialize_field("diagnostics", &SortedSeq(diagnostics))?;
        out.serialize_field("extension", extension)?;
        out.serialize_field("mappings", mappings)?;
        if !semantic_links.is_empty() {
            out.serialize_field("semanticLinks", &SortedSeq(semantic_links))?;
        }
        out.serialize_field("text", text)?;
        out.end()
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

impl Serialize for Sorted<'_, ContentMapperDiagnosticDirectives> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ContentMapperDiagnosticDirectives {
            unused_expect_directive_diagnostics,
            directives,
        } = self.0;
        let mut out = serializer.serialize_struct("ContentMapperDiagnosticDirectives", 2)?;
        out.serialize_field("directives", directives)?;
        out.serialize_field(
            "unusedExpectDirectiveDiagnostics",
            &SortedSeq(unused_expect_directive_diagnostics),
        )?;
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
            text: "const a = \"雪\\n\";\u{2028}🌸".into(),
            extension: ".tsx",
            mappings: vec![ContentMapperSpan([0, 5, 10, 5, 0, 7]); 3],
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
                directives: vec![ContentMapperDiagnosticDirective([1, 2, 3, 4, 1, 0])],
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
                    encode_result(&id, transform(full)).unwrap(),
                );
            }
        }
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
