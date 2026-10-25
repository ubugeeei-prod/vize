use super::{
    ContentMapping, EncodedSourceFile, MAX_SOURCE_FILE_PROTOCOL_VERSION,
    MIN_SOURCE_FILE_PROTOCOL_VERSION, NO_STRUCTURED_DATA, SpanMapFeature, SpanMapKind, TextRange,
};
use crate::{DiagnosticDirectivePolicy, EncodedPayload, SpanMapFidelity};

const VIRTUAL_TEXT: &str = "export const count: number = 1;\n";
const ORIGINAL_TEXT: &str = "<script>\nconst count = 1;\n</script>\n";

/// Builds payloads in the layout `tsc/internal/api/encoder/encoder.go` writes.
#[derive(Default)]
struct SourceFileEncoder {
    offsets: Vec<u32>,
    file_text: String,
    other_strings: String,
    structured: Vec<u8>,
}

impl SourceFileEncoder {
    fn new(text: &str) -> Self {
        Self {
            file_text: text.to_owned(),
            ..Default::default()
        }
    }

    fn add_range(&mut self, start: usize, end: usize) -> u32 {
        let index = self.offsets.len() as u32;
        self.offsets.push(start as u32);
        self.offsets.push(end as u32);
        index
    }

    /// Adds the file text itself, which the encoder always records first.
    fn add_file_text(&mut self) -> u32 {
        self.add_range(0, self.file_text.len())
    }

    /// Appends a string that is not a slice of the file text.
    fn add_string(&mut self, value: &str) -> u32 {
        let start = self.file_text.len() + self.other_strings.len();
        self.other_strings.push_str(value);
        self.add_range(start, start + value.len())
    }

    fn add_structured(&mut self, bytes: &[u8]) -> u32 {
        let offset = self.structured.len() as u32;
        self.structured.extend_from_slice(bytes);
        offset
    }

    fn finish(self, extended: [u32; 19]) -> Vec<u8> {
        self.finish_with_version(MAX_SOURCE_FILE_PROTOCOL_VERSION, extended)
    }

    fn finish_with_version(self, version: u8, extended: [u32; 19]) -> Vec<u8> {
        let string_offsets = 44;
        let string_data = string_offsets + self.offsets.len() * 4;
        let extended_data = string_data + self.file_text.len() + self.other_strings.len();
        let structured_data = extended_data + extended.len() * 4;
        let nodes = structured_data + self.structured.len();

        let mut payload = Vec::new();
        payload.extend_from_slice(&(u32::from(version) << 24).to_le_bytes());
        payload.extend_from_slice(&[0; 16]); // content hash
        payload.extend_from_slice(&0_u32.to_le_bytes()); // parse options
        for offset in [
            string_offsets,
            string_data,
            extended_data,
            structured_data,
            nodes,
        ] {
            payload.extend_from_slice(&(offset as u32).to_le_bytes());
        }
        for offset in &self.offsets {
            payload.extend_from_slice(&offset.to_le_bytes());
        }
        payload.extend_from_slice(self.file_text.as_bytes());
        payload.extend_from_slice(self.other_strings.as_bytes());
        for field in extended {
            payload.extend_from_slice(&field.to_le_bytes());
        }
        payload.extend_from_slice(&self.structured);
        // Nil sentinel node, then the root SourceFile node pointing at extended
        // data offset 0.
        payload.extend_from_slice(&[0; 28]);
        for field in [
            308_u32, // kind (SourceFile)
            0,       // pos
            0,       // end
            0,       // next sibling
            0,       // parent
            2 << 30, // extended data type, offset 0
            0,       // flags
        ] {
            payload.extend_from_slice(&field.to_le_bytes());
        }
        payload
    }
}

fn msgpack_array(length: usize) -> Vec<u8> {
    assert!(length <= 0x0f, "tests only build small arrays");
    vec![0x90 | length as u8]
}

fn msgpack_uint(value: u32) -> Vec<u8> {
    if value <= 0x7f {
        vec![value as u8]
    } else if value <= 0xff {
        vec![0xcc, value as u8]
    } else if value <= 0xffff {
        vec![0xcd, (value >> 8) as u8, value as u8]
    } else {
        vec![
            0xce,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ]
    }
}

fn msgpack_string(value: &str) -> Vec<u8> {
    assert!(value.len() <= 0x1f, "tests only build short strings");
    let mut bytes = vec![0xa0 | value.len() as u8];
    bytes.extend_from_slice(value.as_bytes());
    bytes
}

/// One encoded span map segment: virtual start/length, original start/length,
/// kind, and the optional feature mask.
type EncodedSegment = (u32, u32, u32, u32, u8, Option<u32>);

/// `[virtualStart, virtualLength, originalStart, originalLength, kind, features?]`
fn span_map_blob(segments: &[EncodedSegment]) -> Vec<u8> {
    let mut bytes = msgpack_array(segments.len());
    for &(virtual_start, virtual_length, original_start, original_length, kind, features) in
        segments
    {
        bytes.extend(msgpack_array(if features.is_some() { 6 } else { 5 }));
        for value in [
            virtual_start,
            virtual_length,
            original_start,
            original_length,
            u32::from(kind),
        ] {
            bytes.extend(msgpack_uint(value));
        }
        if let Some(features) = features {
            bytes.extend(msgpack_uint(features));
        }
    }
    bytes
}

/// `[originalStart, originalLength, virtualStart, virtualLength, policy, unusedCode]`
fn directives_blob(directives: &[(u32, u32, u32, u32, u32, u32)]) -> Vec<u8> {
    let mut bytes = msgpack_array(directives.len());
    for &directive in directives {
        bytes.extend(msgpack_array(6));
        for value in [
            directive.0,
            directive.1,
            directive.2,
            directive.3,
            directive.4,
            directive.5,
        ] {
            bytes.extend(msgpack_uint(value));
        }
    }
    bytes
}

fn string_array_blob(values: &[&str]) -> Vec<u8> {
    let mut bytes = msgpack_array(values.len());
    for value in values {
        bytes.extend(msgpack_string(value));
    }
    bytes
}

/// A file no content mapper touched.
fn plain_source_file() -> Vec<u8> {
    let mut encoder = SourceFileEncoder::new(VIRTUAL_TEXT);
    let text = encoder.add_file_text();
    let file_name = encoder.add_string("/workspace/src/index.ts");
    let path = encoder.add_string("/workspace/src/index.ts");
    encoder.finish([
        text,
        file_name,
        path,
        0, // languageVariant
        3, // scriptKind
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        0,
        text, // originalText is the same string
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
    ])
}

/// A `.vue`-style file a mapper turned into virtual TypeScript.
fn mapped_source_file() -> Vec<u8> {
    let mut encoder = SourceFileEncoder::new(VIRTUAL_TEXT);
    let text = encoder.add_file_text();
    let file_name = encoder.add_string("/workspace/src/App.vue");
    let path = encoder.add_string("/workspace/src/app.vue");
    let original_text = encoder.add_string(ORIGINAL_TEXT);
    let content_mapper = encoder.add_string("vue-mapper@1.2.3");
    let virtual_file_name = encoder.add_string("/workspace/src/App.vue.ts");
    let span_map = encoder.add_structured(&span_map_blob(&[
        (0, 13, 9, 13, 0, None),
        (13, 8, 22, 3, 1, Some(SpanMapFeature::HOVER.bits())),
    ]));
    let directives = encoder.add_structured(&directives_blob(&[(9, 4, 0, 6, 1, 2578)]));
    let supplemental = encoder.add_structured(&string_array_blob(&["/workspace/src/App.vue.0.ts"]));
    encoder.finish([
        text,
        file_name,
        path,
        0,
        3,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        0,
        original_text,
        span_map,
        supplemental,
        NO_STRUCTURED_DATA,
        content_mapper,
        virtual_file_name,
        directives,
    ])
}

#[test]
fn plain_files_decode_without_content_mapper_state() {
    let decoded = EncodedSourceFile::decode(&plain_source_file()).expect("payload decodes");

    assert_eq!(decoded.protocol_version, MAX_SOURCE_FILE_PROTOCOL_VERSION);
    assert_eq!(decoded.file_name, "/workspace/src/index.ts");
    assert_eq!(decoded.text, VIRTUAL_TEXT);
    assert_eq!(decoded.original_text, VIRTUAL_TEXT);
    assert_eq!(decoded.script_kind, 3);
    assert!(!decoded.is_content_mapped());
    assert_eq!(decoded.content_mapping(), None);
    assert_eq!(decoded.span_map(), None);
}

#[test]
fn mapped_files_expose_the_mapper_and_both_texts() {
    let decoded = EncodedSourceFile::decode(&mapped_source_file()).expect("payload decodes");
    let mapping = decoded
        .content_mapping()
        .expect("the file is content mapped");

    assert!(decoded.is_content_mapped());
    assert_eq!(decoded.text, VIRTUAL_TEXT);
    assert_eq!(decoded.original_text, ORIGINAL_TEXT);
    assert_eq!(mapping.content_mapper, "vue-mapper@1.2.3");
    assert_eq!(mapping.virtual_file_name, "/workspace/src/App.vue.ts");
    assert_eq!(
        mapping.supplemental_source_file_names,
        ["/workspace/src/App.vue.0.ts"]
    );
    assert_eq!(mapping.canonical_source_file_name, None);
    assert!(!mapping.is_supplemental());
}

#[test]
fn span_map_segments_decode_as_start_and_length_pairs() {
    let decoded = EncodedSourceFile::decode(&mapped_source_file()).expect("payload decodes");
    let span_map = decoded.span_map().expect("the file is content mapped");
    let segments = span_map.segments();

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].virtual_range(), TextRange::new(0, 13));
    assert_eq!(segments[0].original_range(), TextRange::new(9, 22));
    assert_eq!(segments[0].kind, SpanMapKind::Verbatim);
    assert_eq!(
        segments[0].features,
        SpanMapFeature::ALL,
        "a five-field tuple means every feature"
    );
    assert_eq!(segments[1].kind, SpanMapKind::Atom);
    assert_eq!(segments[1].features, SpanMapFeature::HOVER);
}

#[test]
fn decoded_span_maps_answer_position_queries() {
    let decoded = EncodedSourceFile::decode(&mapped_source_file()).expect("payload decodes");
    let span_map = decoded.span_map().expect("the file is content mapped");

    let mapped = span_map.virtual_to_original_position(6);

    assert_eq!(mapped.position, 15);
    assert_eq!(mapped.fidelity, SpanMapFidelity::Exact);
    assert_eq!(
        span_map
            .virtual_to_original_position_for_feature(14, SpanMapFeature::RENAME)
            .fidelity,
        SpanMapFidelity::None,
        "the second segment only opted into hover"
    );
}

#[test]
fn diagnostic_directives_decode_in_both_coordinate_spaces() {
    let decoded = EncodedSourceFile::decode(&mapped_source_file()).expect("payload decodes");
    let mapping = decoded
        .content_mapping()
        .expect("the file is content mapped");

    assert_eq!(mapping.diagnostic_directives.len(), 1);
    let directive = mapping.diagnostic_directives[0];
    assert_eq!(directive.original_range, TextRange::new(9, 13));
    assert_eq!(directive.virtual_range, TextRange::new(0, 6));
    assert_eq!(directive.policy, DiagnosticDirectivePolicy::Expect);
    assert_eq!(directive.unused_code, 2578);
}

#[test]
fn supplemental_outputs_name_their_canonical_file() {
    let mut encoder = SourceFileEncoder::new(VIRTUAL_TEXT);
    let text = encoder.add_file_text();
    let file_name = encoder.add_string("/workspace/App.vue.0.ts");
    let path = encoder.add_string("/workspace/app.vue.0.ts");
    let canonical = encoder.add_string("/workspace/App.vue");
    let content_mapper = encoder.add_string("vue-mapper@1.2.3");
    let span_map = encoder.add_structured(&span_map_blob(&[]));
    let payload = encoder.finish([
        text,
        file_name,
        path,
        0,
        3,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        0,
        text,
        span_map,
        NO_STRUCTURED_DATA,
        canonical,
        content_mapper,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
    ]);

    let decoded = EncodedSourceFile::decode(&payload).expect("payload decodes");
    let mapping = decoded
        .content_mapping()
        .expect("the file is content mapped");

    assert!(mapping.is_supplemental());
    assert_eq!(
        mapping.canonical_source_file_name.as_deref(),
        Some("/workspace/App.vue")
    );
    assert!(
        mapping.span_map.is_empty(),
        "an empty span map describes fully synthesized output"
    );
    assert_eq!(mapping.virtual_file_name, "");
}

#[test]
fn payloads_decode_straight_off_the_binary_endpoint_wrapper() {
    let payload = EncodedPayload::new(mapped_source_file());

    let decoded = payload.decode_source_file().expect("payload decodes");

    assert_eq!(
        decoded.content_mapping(),
        Some(&ContentMapping {
            content_mapper: "vue-mapper@1.2.3".to_owned(),
            virtual_file_name: "/workspace/src/App.vue.ts".to_owned(),
            span_map: decoded.span_map().expect("mapped").clone(),
            diagnostic_directives: decoded
                .content_mapping()
                .expect("mapped")
                .diagnostic_directives
                .clone(),
            supplemental_source_file_names: vec!["/workspace/src/App.vue.0.ts".to_owned()],
            canonical_source_file_name: None,
        })
    );
}

#[test]
fn unsupported_protocol_versions_are_reported_rather_than_guessed() {
    let mut encoder = SourceFileEncoder::new(VIRTUAL_TEXT);
    let text = encoder.add_file_text();
    let payload = encoder.finish_with_version(MAX_SOURCE_FILE_PROTOCOL_VERSION + 1, [text; 19]);

    let error = EncodedSourceFile::decode(&payload).expect_err("a newer payload must not decode");

    assert!(
        error
            .to_string()
            .contains("unsupported encoded source file protocol version 9"),
        "unexpected error: {error}"
    );
}

#[test]
fn every_supported_protocol_version_decodes() {
    // Version 8 only changed how the node table records a `NodeList`'s
    // trailing comma; the source-file layout this decoder reads is the same.
    for version in MIN_SOURCE_FILE_PROTOCOL_VERSION..=MAX_SOURCE_FILE_PROTOCOL_VERSION {
        let mut encoder = SourceFileEncoder::new(VIRTUAL_TEXT);
        let text = encoder.add_file_text();
        let file_name = encoder.add_string("/workspace/src/index.ts");
        let payload = encoder.finish_with_version(
            version,
            [
                text,
                file_name,
                file_name,
                0,
                3,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                0,
                text,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
                NO_STRUCTURED_DATA,
            ],
        );

        let decoded = EncodedSourceFile::decode(&payload)
            .unwrap_or_else(|error| panic!("version {version} must decode: {error}"));

        assert_eq!(decoded.protocol_version, version);
        assert_eq!(decoded.text, VIRTUAL_TEXT);
    }
}

#[test]
fn truncated_payloads_are_rejected() {
    let payload = mapped_source_file();

    let error = EncodedSourceFile::decode(&payload[..20]).expect_err("a short payload must fail");

    assert!(
        error
            .to_string()
            .contains("shorter than the 44 byte header"),
        "unexpected error: {error}"
    );
}

#[test]
fn payloads_whose_root_node_is_missing_are_rejected() {
    let mut payload = mapped_source_file();
    payload.truncate(payload.len() - 28);

    let error = EncodedSourceFile::decode(&payload).expect_err("a rootless payload must fail");

    assert!(
        error.to_string().contains("no root node record"),
        "unexpected error: {error}"
    );
}

#[test]
fn unknown_span_map_kinds_surface_as_protocol_errors() {
    let mut encoder = SourceFileEncoder::new(VIRTUAL_TEXT);
    let text = encoder.add_file_text();
    let mapper = encoder.add_string("vue-mapper@1.2.3");
    let span_map = encoder.add_structured(&span_map_blob(&[(0, 4, 0, 4, 9, None)]));
    let payload = encoder.finish([
        text,
        text,
        text,
        0,
        3,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        0,
        text,
        span_map,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
        mapper,
        NO_STRUCTURED_DATA,
        NO_STRUCTURED_DATA,
    ]);

    let error = EncodedSourceFile::decode(&payload).expect_err("unknown kinds must not decode");

    assert!(
        error.to_string().contains("unknown span map kind 9"),
        "unexpected error: {error}"
    );
}
