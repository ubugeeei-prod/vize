//! Exercise the combined style request on actual whole Vue documents.
use super::*;
use crate::source_project::style_format::composition::diagnostics;
use std::path::Path;
use vize_atelier_sfc::scss::ScssProducts;
use vize_l1::{container::vue::Vue, css::lang::StyleLanguage};

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    non_utf8: usize,
    styles: usize,
    external: usize,
    diagnostics: usize,
    compiler_refusals: usize,
    definitions: usize,
    checked_formats: usize,
    refused_formats: usize,
}

#[test]
#[ignore = "reads the tracked or external Vue document corpus"]
fn shared_style_queries_on_repository_documents() {
    let manifest = std::fs::read(std::env::var_os("VIZE_COMPOSED_STYLE_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let uri = Url::parse("file:///Original.vue").unwrap();
    let rules = CssLinter::with_all_rules();
    let options = FormatOptions::default();
    let mut counts = Counts::default();
    for path in manifest.split(|n| *n == 0).filter(|name| !name.is_empty()) {
        let name = std::str::from_utf8(path).unwrap();
        let data = std::fs::read(root.join(name)).unwrap();
        counts.files += 1;
        let Ok(source) = String::from_utf8(data) else {
            counts.non_utf8 += 1;
            continue;
        };
        if !source.to_ascii_lowercase().contains("scss") {
            continue;
        }
        let docs = DocumentStore::new();
        docs.open(uri.clone(), source, 1, "vue".into());
        let snap = SourceSnapshotCache::default().capture(&docs, &uri).unwrap();
        let arena = Allocator::new();
        let descriptor = Vue.observe_descriptor(
            &arena,
            snap.source(),
            NativeSfcCompileOptions::default().descriptor,
        );
        let Ok(view) = descriptor.admitted() else {
            continue;
        };
        let formatter = GlyphFormatter::new(&options, &arena);
        for (ordinal, style) in view.styles().enumerate() {
            let Ok(reference) = ScssProducts::from_style(style) else {
                continue;
            };
            counts.styles += 1;
            let shared = snap.prepare_scss_with(&docs, &descriptor, ordinal, |s| {
                rules.lint_file_style(StyleRuleInput::Observed(s), true)
            });
            if style.attrs().iter().any(|a| a.name == "src") {
                assert!(
                    matches!(shared, Err(StyleFormatError::ExternalStyle)),
                    "{name}"
                );
                counts.external += 1;
                continue;
            }
            let (prepared, lint) = shared.unwrap_or_else(|e| panic!("{name}: {e:?}"));
            let block = style.block();
            let stock = rules.lint_block(
                block.source(),
                block.start() as usize,
                StyleLanguage::Scss,
                true,
            );
            assert_eq!(diagnostics(&lint), diagnostics(&stock), "{name}");
            counts.diagnostics += lint.diagnostics.len();
            let native = prepared.products();
            assert_eq!(native.css(), reference.css(), "{name}");
            assert_eq!(native.classes(), reference.classes(), "{name}");
            if let Ok(program) = reference.program() {
                for used in program.variable_uses() {
                    let expected = program.definition_at(used.span.start).unwrap().span;
                    let pos = |offset: u32| offset_to_position_str(snap.source(), offset as usize);
                    let location = snap
                        .scss_definition(&docs, native, pos(used.span.start))
                        .unwrap();
                    let Some(GotoDefinitionResponse::Scalar(location)) = location else {
                        panic!("{name}: missing definition")
                    };
                    assert_eq!(location.uri, uri, "{name}");
                    assert_eq!(
                        location.range,
                        Range::new(pos(expected.start), pos(expected.end)),
                        "{name}"
                    );
                    counts.definitions += 1;
                }
            } else {
                counts.compiler_refusals += 1;
            }
            let baseline = snap.verified_style_format(&docs, &descriptor, ordinal, &formatter);
            match (prepared.verified_format(&docs, &formatter), baseline) {
                (Ok(shared), Ok(separate)) => {
                    assert_eq!(
                        shared.publish(&docs).unwrap(),
                        separate.publish(&docs).unwrap(),
                        "{name}"
                    );
                    counts.checked_formats += 1;
                }
                (Err(a), Err(b)) => {
                    assert_eq!(format!("{a:?}"), format!("{b:?}"), "{name}");
                    counts.refused_formats += 1;
                }
                _ => panic!("{name}: composed and separate format decisions differ"),
            }
        }
    }
    println!("shared toolchain style corpus {counts:?}");
    assert!(counts.checked_formats > 0 && counts.refused_formats > 0 && counts.diagnostics > 0);
    assert_eq!(
        counts.styles,
        counts.checked_formats + counts.refused_formats + counts.external
    );
}
