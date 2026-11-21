//! Run the strict checker on both original-file projections and compare authored diagnostics.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use vize_canon::batch::VueDocumentVirtualTs;
use vize_l0::line_index::utf16_offset;

#[derive(Serialize)]
struct Document<'a> {
    path: &'a str,
    extension: &'a str,
    text: &'a str,
}

#[derive(Deserialize)]
struct Diagnostic {
    category: u32,
    code: u32,
    message: String,
    start: u32,
    length: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct AuthoredDiagnostic<'a> {
    category: u32,
    code: u32,
    message: &'a str,
    span: Span,
    text: &'a str,
}

fn check(documents: Vec<Document<'_>>) -> Vec<Vec<Diagnostic>> {
    let mut node = Command::new("node")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/styled_projection/checker.ts"
        ))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    node.stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&documents).unwrap())
        .unwrap();
    let output = node.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let packets: Vec<Vec<Diagnostic>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(packets.len(), documents.len());
    packets
}

fn original_span(document: &VueDocumentVirtualTs, at: u32, len: u32) -> Option<Span> {
    let offset = |n| {
        let byte = u32::try_from(utf16_offset(&document.code, n)?).ok()?;
        Some(document.import_source_map.get_original_offset(byte) as usize)
    };
    let (lo, hi) = document
        .mapping
        .diagnostic_range_to_authored(offset(at)?, offset(at.checked_add(len)?)?)?;
    Some(Span::new(u32::try_from(lo).ok()?, u32::try_from(hi).ok()?))
}

fn authored<'a>(
    packet: &'a [Diagnostic],
    source: &'a str,
    path: &str,
    map: impl Fn(u32, u32) -> Option<Span>,
) -> Vec<AuthoredDiagnostic<'a>> {
    packet
        .iter()
        .map(|diagnostic| {
            let span = map(diagnostic.start, diagnostic.length).expect(path);
            AuthoredDiagnostic {
                category: diagnostic.category,
                code: diagnostic.code,
                message: &diagnostic.message,
                span,
                text: source.get(span.start as usize..span.end as usize).unwrap(),
            }
        })
        .collect()
}

pub(super) fn compare_programs(cases: &[(&str, &str)]) -> Vec<Vec<u32>> {
    let arenas: Vec<_> = cases.iter().map(|_| Allocator::default()).collect();
    let observations: Vec<_> = arenas
        .iter()
        .zip(cases)
        .map(|(a, (_, source))| lower_sfc_native(a, source, options()))
        .collect();
    let native: Vec<_> = observations
        .iter()
        .map(|o| {
            StyledVueInput::prepare(o.admitted().unwrap())
                .unwrap()
                .project()
                .unwrap()
        })
        .collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let canon: Vec<_> = cases
        .iter()
        .map(|(path, source)| {
            generate_vue_document_virtual_ts(
                &root.join(path),
                source,
                &VirtualTsOptions::default(),
                &ImportRewriter::new(),
                false,
            )
            .unwrap()
        })
        .collect();
    let before = check(
        canon
            .iter()
            .enumerate()
            .map(|(id, doc)| {
                assert_eq!(doc.virtual_suffix, ".ts");
                Document {
                    path: cases.get(id).unwrap().0,
                    extension: "ts",
                    text: &doc.code,
                }
            })
            .collect(),
    );
    let after = check(
        native
            .iter()
            .enumerate()
            .map(|(id, doc)| Document {
                path: cases.get(id).unwrap().0,
                extension: doc.source_kind().extension(),
                text: doc.document().as_str(),
            })
            .collect(),
    );
    let mut codes = Vec::new();
    let original = canon.iter().zip(&before);
    let projected = native.iter().zip(&after);
    for (&(path, source), ((canon_doc, raw_before), (native_doc, raw_after))) in
        cases.iter().zip(original.zip(projected))
    {
        let left = authored(raw_before, source, path, |at, len| {
            original_span(canon_doc, at, len)
        });
        let right = authored(raw_after, source, path, |at, len| {
            native_doc.map_utf16(at, len).ok()
        });
        assert_eq!(left, right, "{path}");
        codes.push(right.iter().map(|d| d.code).collect());
    }
    codes
}

#[test]
fn complete_typescript_diagnostics_match_canons_original_styled_sfc() {
    let cases = [
        (
            "crates/vize_canon/tests/styled_projection/leading.vue",
            "<style>.é { color: green }</style>\r\n<script setup lang='ts'>const count: number = 'no';</script><template>{{ count }}</template>",
        ),
        (
            "crates/vize_canon/tests/styled_projection/trailing.vue",
            "<script setup lang='ts'>const label: string = '雪';</script><template>{{ label }}</template><style scoped>.label{color:blue}</style>",
        ),
        (
            "crates/vize_canon/tests/styled_projection/two.vue",
            "<style>/* v-bind(missing) */</style><script setup lang='ts'>const value: boolean = 1;</script><style lang='css'>.a::before{content:'😀'}</style>",
        ),
        (
            "crates/vize_canon/tests/styled_projection/js-read.vue",
            "<style scoped>.雪 { color: red }</style><script setup>const count = 7;</script><template>{{ count.missing }}</template>",
        ),
        (
            "crates/vize_canon/tests/styled_projection/js-valid.vue",
            "<script setup>const label = '雪';</script><template>{{ label.length }}</template><style>.red{color:red}</style>",
        ),
        (
            "crates/vize_canon/tests/styled_projection/relative.vue",
            "<script setup lang='ts'>import {counter} from './relative-types'; let value=counter;</script><style>.red{color:red}</style>",
        ),
        (
            "crates/vize_canon/tests/styled_projection/relative-missing.vue",
            "<script setup lang='ts'>import {missing} from './relative-types'; let value=missing;</script><style>.red{color:red}</style>",
        ),
    ];
    assert_eq!(
        compare_programs(&cases),
        [
            vec![2322],
            vec![],
            vec![2322],
            vec![2339],
            vec![],
            vec![],
            vec![2305]
        ]
    );
}
