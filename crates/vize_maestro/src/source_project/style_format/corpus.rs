//! Each scene uses the original stock writer and the shipped Sass adapter as
//! independent byte oracles for both the original and edited document.
use super::*;
use crate::{source_project::SourceSnapshotCache, utils::position_to_offset};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use tower_lsp::lsp_types::Url;
use vize_glyph::{EndOfLine, FormatOptions};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::container::vue::DescriptorOptions;

#[derive(Default)]
struct Counts {
    files: usize,
    selected: usize,
    refused: usize,
    checked: usize,
    changed: usize,
    uses: usize,
}

fn examine(name: &str, text: &str, counts: &mut Counts, rows: &mut Vec<Value>) {
    counts.files += 1;
    if !text.to_ascii_lowercase().contains("scss") {
        return;
    }
    let uri = Url::parse("file:///Original.vue").unwrap();
    let docs = DocumentStore::new();
    docs.open(uri.clone(), text.into(), 4, "vue".into());
    let cache = SourceSnapshotCache::default();
    let snapshot = cache.capture(&docs, &uri).unwrap();
    let arena = Allocator::default();
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    };
    let descriptor = Vue.observe_descriptor(&arena, snapshot.source(), options);
    let Ok(view) = descriptor.admitted() else {
        return;
    };
    let settings = FormatOptions {
        end_of_line: EndOfLine::Lf,
        ..Default::default()
    };
    let formatter = GlyphFormatter::new(&settings, &arena);
    for (ordinal, style) in view.styles().enumerate() {
        let Ok(original) = ScssProducts::from_style(style) else {
            continue;
        };
        counts.selected += 1;
        let result = snapshot.verified_style_format(&docs, &descriptor, ordinal, &formatter);
        if original.program().is_err()
            || !original.classes().is_some_and(|c| c.is_complete())
            || style.attrs().iter().any(|a| a.name == "src")
        {
            assert!(result.is_err(), "{name}");
            counts.refused += 1;
            continue;
        }
        let checked = result.unwrap_or_else(|error| panic!("{name}: {error:?}"));
        counts.checked += 1;
        let span = style.original_block().authored_span();
        let fragment = text.get(span.start as usize..span.end as usize).unwrap();
        let stock = formatter.format(fragment).unwrap();
        let rendered = stock.code.strip_suffix('\n').unwrap();
        let mut newer = text.to_owned();
        if let Some(edit) = checked.publish(&docs).unwrap() {
            let [OneOf::Left(edit)] = edit.edits.as_slice() else {
                panic!("one selected original style")
            };
            assert_eq!(edit.new_text, rendered, "{name}");
            let doc = docs.get(&uri).unwrap();
            let pos = doc.position_text();
            let from = position_to_offset(pos, edit.range.start).unwrap();
            let to = position_to_offset(pos, edit.range.end).unwrap();
            assert_eq!((from, to), (span.start as usize, span.end as usize));
            newer.replace_range(from..to, &edit.new_text);
            counts.changed += 1;
        } else {
            assert_eq!(fragment, rendered, "{name}");
        }
        let edited = Vue.observe_descriptor(&arena, &newer, options);
        let next = edited.admitted().unwrap().styles().nth(ordinal).unwrap();
        let after = ScssProducts::from_style(next).unwrap();
        assert_eq!(original.css().unwrap(), after.css().unwrap());
        assert_eq!(original.classes(), after.classes());
        let a = original.program().unwrap();
        let b = after.program().unwrap();
        for (left, right) in a.variable_uses().iter().zip(b.variable_uses()) {
            let d = a.definition_at(left.span.start).unwrap();
            let e = b.definition_at(right.span.start).unwrap();
            assert_eq!((d.name, d.value), (e.name, e.value), "{name}");
            counts.uses += 1;
        }
        for product in [&original, &after] {
            rows.push(json!({"name":name,"source":product.block().source(),"css":product.css().unwrap().as_str()}));
        }
    }
}

#[test]
#[ignore = "requires a real-project manifest and the pinned Sass oracle"]
fn original_edits_match_the_stock_formatter_preprocessor_and_current_editor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let input = std::fs::read(std::env::var_os("VIZE_FORMAT_SCSS_INPUTS").unwrap()).unwrap();
    let mut counts = Counts::default();
    let mut rows = Vec::new();
    for path in input.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let name = std::str::from_utf8(path).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        if let Ok(source) = std::str::from_utf8(&bytes) {
            examine(name, source, &mut counts, &mut rows);
        }
    }
    println!(
        "formatting corpus files={} selected={} refused={} certified={} edited={} bindings={}",
        counts.files, counts.selected, counts.refused, counts.checked, counts.changed, counts.uses
    );
    assert!(counts.checked > 0 && counts.refused > 0);
    let mut generated = Counts::default();
    for name in ["a", "large_one", "test-x"] {
        for color in ["red", "blue", "12px", "#abc"] {
            for body in ["color:$v", "margin:$v $v", "&.on{color:$v} width:1px"] {
                for prefix in ["\r\n", "<!--🦀-->"] {
                    let source = format!(
                        "{prefix}<template><p/></template><style module lang='scss'>$v:{color};.{name}{{{body}}}</style>"
                    );
                    examine("Generated.vue", &source, &mut generated, &mut rows);
                }
            }
        }
    }
    assert_eq!(generated.checked, 72);
    let mut child = Command::new("node")
        .arg(root.join("crates/vize_atelier_sfc/tests/observed_scss/oracle.ts"))
        .arg("--correctness-only")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&rows).unwrap())
        .unwrap();
    let reply = child.wait_with_output().unwrap();
    assert!(reply.status.success());
    let report: Value = serde_json::from_slice(&reply.stdout).unwrap();
    assert_eq!(report["checked"], rows.len());
    println!(
        "formatting source pairs={} generated={} generated_edges={}",
        rows.len() / 2,
        generated.checked,
        generated.uses
    );
}
