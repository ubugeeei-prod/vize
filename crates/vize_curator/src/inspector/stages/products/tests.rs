#![expect(
    clippy::unwrap_used,
    reason = "read original input and compare complete serializations"
)]
use super::{InspectorSourceFile, TemplateComplexity, inspect_template_products};
use crate::inspector::{
    InspectorOptions, InspectorTarget, InspectorTemplateSyntax, build_payload, serialize_payload,
};
use std::{fs, path::Path};
use vize_l0::String;

fn original(files: &[InspectorSourceFile]) -> (serde_json::Value, Vec<Option<TemplateComplexity>>) {
    let feed = super::super::payload_stage_feed(files);
    let records = files
        .iter()
        .map(|file| {
            if file.path.ends_with(".vue") {
                TemplateComplexity::from_sfc(&file.source)
            } else {
                None
            }
        })
        .collect();
    (feed, records)
}

fn composed(files: &[InspectorSourceFile]) -> (serde_json::Value, Vec<Option<TemplateComplexity>>) {
    let mut records = Vec::new();
    let feed = inspect_template_products(files, |file, record| {
        assert!(core::ptr::eq(&files[records.len()], file));
        records.push(record);
    });
    assert_eq!(records.len(), files.len());
    (feed, records)
}

fn file(path: &str, source: &str) -> InspectorSourceFile {
    InspectorSourceFile {
        path: String::from(path),
        source: String::from(source),
    }
}

#[test]
fn original_stage_pages_remarks_and_component_records_keep_both_coordinate_frames() {
    let files = [
        file(
            "src/花.vue",
            "<script setup>const 雪 = true</script>\r\n<template><!--🌱--><section><i v-if='雪 && ok'>A</i><b v-else>B</b></section></template>",
        ),
        file(
            "src/App.vue",
            "<template><Box><template #default='x'><i v-for='y in x' :key='y'/></template></Box></template>",
        ),
        file("src/Pug.vue", "<template lang='pug'>i hello</template>"),
        file(
            "src/External.vue",
            "<template src='./part.html'></template>",
        ),
        file("src/Missing.vue", "<script>export default {}</script>"),
        file("src/Broken.vue", "<template><p>"),
        file("src/mixed.ts", "export const n=1"),
    ];
    let before = original(&files);
    let after = composed(&files);
    assert_eq!(before, after);
    let payload = build_payload(
        InspectorTarget::Dom,
        InspectorOptions {
            custom_renderer: false,
            template_syntax: InspectorTemplateSyntax::Standard,
        },
        files.to_vec(),
    );
    let published: serde_json::Value =
        serde_json::from_str(&serialize_payload(&payload).unwrap()).unwrap();
    assert_eq!(after.0, published["stageFeed"]);
    assert_eq!(
        after.1.iter().map(Option::is_some).collect::<Vec<_>>(),
        [true, true, false, false, false, false, false]
    );
}

fn inputs(mut visit: impl FnMut(InspectorSourceFile)) -> (usize, usize) {
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_INSPECTOR_TEMPLATE_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut total = 0;
    let mut invalid = 0;
    for path in manifest.split_terminator('\0') {
        total += 1;
        let bytes = fs::read(root.join(path)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        visit(file(path.strip_suffix(".txt").unwrap_or(path), source));
    }
    (total, invalid)
}

#[test]
#[ignore = "compare original inspector stage feeds and complete project complexity records"]
fn original_template_products() {
    let (mut compared, mut pages, mut remarks, mut records, mut contributors, mut bytes) =
        (0, 0, 0, 0, 0, 0);
    let (total, invalid) = inputs(|file| {
        let files = std::slice::from_ref(&file);
        let before = original(files);
        let after = composed(files);
        assert_eq!(after, before, "{}", file.path);
        pages += after.0["pages"].as_array().unwrap().len();
        remarks += after.0["remarks"].as_array().unwrap().len();
        for record in after.1.iter().flatten() {
            records += 1;
            contributors += record.contributors.len();
        }
        bytes += serde_json::to_vec(&after).unwrap().len();
        compared += 1;
    });
    assert_eq!(total, invalid + compared);
    assert!(pages > 0 && remarks > 0 && records > 0 && contributors > 0);
    eprintln!(
        "inspector_template_products total={total} invalid={invalid} compared={compared} pages={pages} remarks={remarks} records={records} contributors={contributors} bytes={bytes}"
    );
}
