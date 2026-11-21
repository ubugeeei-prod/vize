//! Both arms run strict TypeScript with identical virtual file identities,
//! resolving the new calls exclusively through the published ambient set.
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use vize_canon::{batch::JsxDocumentVirtualTs, virtual_ts::ProjectionMapping};
use vize_l0::{Span, line_index::utf16_offset};
use vize_l4::targets::ts::{COMPONENT_HELPERS, TsDocument};

pub(super) struct Comparison {
    name: String,
    source: String,
    original: JsxDocumentVirtualTs,
    projected: TsDocument,
}

impl Comparison {
    pub(super) fn new(
        name: &str,
        source: String,
        original: JsxDocumentVirtualTs,
        projected: TsDocument,
    ) -> Self {
        Self {
            name: name.into(),
            source,
            original,
            projected,
        }
    }
}

fn check(inputs: Vec<Value>, helpers: &[&str]) -> Value {
    let mut child = Command::new("node")
        .args([
            "--input-type=module",
            "-e",
            include_str!("../../../../tests/tooling/support/native-jsx-module-typescript.ts"),
        ])
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&json!({"cases":inputs,"helpers":helpers})).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub(super) fn compare_checker(documents: Vec<Comparison>) -> Vec<Vec<u32>> {
    let inputs = |original: bool| {
        documents
            .iter()
            .enumerate()
            .map(|(index, doc)| {
                let text = if original {
                    doc.original.code.as_str()
                } else {
                    doc.projected.document().as_str()
                };
                json!({"id": index, "projected": text})
            })
            .collect()
    };
    let before = check(inputs(true), &[]);
    let after = check(inputs(false), COMPONENT_HELPERS);
    for helper in after["helpers"].as_array().unwrap() {
        assert_eq!(
            helper["declarations"],
            json!(["npm/cli/virtual-ts-globals.d.ts"])
        );
    }
    assert_eq!(before["cases"].as_array().unwrap().len(), documents.len());
    assert_eq!(after["cases"].as_array().unwrap().len(), documents.len());
    documents
        .into_iter()
        .enumerate()
        .map(|(i, doc)| {
            let code = doc.original.code;
            let mapping = ProjectionMapping::from_spans(doc.original.mappings);
            let original = &before["cases"][i];
            let candidate = &after["cases"][i];
            assert_eq!(original["id"], i);
            assert_eq!(candidate["id"], i);
            let old = mapped(original, &doc.source, |start, length| {
                let left = utf16_offset(&code, start).unwrap();
                let right = utf16_offset(&code, start + length).unwrap();
                let (a, b) = mapping
                    .diagnostic_range_to_authored(left, right)
                    .unwrap_or_else(|| panic!("{}: Canon mapping {start}/{length}", doc.name));
                Span::new(u32::try_from(a).unwrap(), u32::try_from(b).unwrap())
            });
            let new = mapped(candidate, &doc.source, |start, length| {
                doc.projected
                    .map_utf16(start, length)
                    .unwrap_or_else(|e| panic!("{}: {e:?} at {start}/{length}", doc.name))
            });
            assert_eq!(old, new, "{}", doc.name);
            old.iter()
                .map(|d| u32::try_from(d["code"].as_u64().unwrap()).unwrap())
                .collect()
        })
        .collect()
}

fn mapped(case: &Value, source: &str, mut map: impl FnMut(u32, u32) -> Span) -> Vec<Value> {
    case["diagnostics"].as_array().unwrap().iter().map(|diagnostic| {
        let start = u32::try_from(diagnostic["start"].as_u64().unwrap()).unwrap();
        let length = u32::try_from(diagnostic["length"].as_u64().unwrap()).unwrap();
        let span = map(start, length);
        let authored = source.get(span.start as usize..span.end as usize).unwrap();
        json!({
            "file": case["id"], "code": diagnostic["code"], "category": diagnostic["category"],
            "message": diagnostic["message"], "start": span.start, "end": span.end, "text": authored,
        })
    }).collect()
}
