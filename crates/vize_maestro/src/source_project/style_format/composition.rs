//! Run the configured rule registry and the compiler, formatter and definition
//! provider on a single original style. Both paths return complete products.
use super::*;
use crate::{source_project::SourceSnapshotCache, utils::offset_to_position_str};
use tower_lsp::lsp_types::{GotoDefinitionResponse, Position, Url};
use vize_glyph::FormatOptions;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{
    container::vue::DescriptorOptions,
    css::{AuthoredClasses, lang::StyleLanguage},
};
use vize_patina::rules::css::{CssLintResult, CssLinter, StyleRuleInput};

struct Outputs {
    css: String,
    classes: Option<AuthoredClasses>,
    lint: CssLintResult,
    definition: Option<GotoDefinitionResponse>,
    edit: Option<TextDocumentEdit>,
}

fn outputs(
    products: &ScssProducts<'_>,
    snapshot: &SourceSnapshot,
    docs: &DocumentStore,
    cursor: Position,
    lint: CssLintResult,
    edit: VerifiedStyleFormat<'_>,
) -> Outputs {
    Outputs {
        css: products.css().unwrap(),
        classes: products.classes().cloned(),
        lint,
        definition: snapshot.scss_definition(docs, products, cursor).unwrap(),
        edit: edit.publish(docs).unwrap(),
    }
}

fn paired(body: &str, inspect: impl FnOnce(&dyn Fn(bool) -> Outputs)) {
    let source = format!("<template>雪🦀</template>\r\n<style lang='scss'>{body}</style>");
    let uri = Url::parse("file:///Theme.vue").unwrap();
    let documents = DocumentStore::new();
    documents.open(uri.clone(), source.clone(), 7, "vue".into());
    let cache = SourceSnapshotCache::default();
    let snapshot = cache.capture(&documents, &uri).unwrap();
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        snapshot.source(),
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let format = FormatOptions::default();
    let formatter = GlyphFormatter::new(&format, &arena);
    let rules = CssLinter::with_all_rules();
    let cursor = offset_to_position_str(&source, source.rfind("$ink").unwrap() + 1);
    let run = |shared: bool| {
        if shared {
            let (prepared, lint) = snapshot
                .prepare_scss_with(&documents, &descriptor, 0, |s| {
                    rules.lint_file_style(StyleRuleInput::Observed(s), true)
                })
                .unwrap();
            let edit = prepared.verified_format(&documents, &formatter).unwrap();
            outputs(
                prepared.products(),
                &snapshot,
                &documents,
                cursor,
                lint,
                edit,
            )
        } else {
            let style = descriptor.admitted().unwrap().styles().next().unwrap();
            let block = style.block();
            let lint = rules.lint_block(
                block.source(),
                block.start() as usize,
                StyleLanguage::Scss,
                true,
            );
            let compiled = ScssProducts::from_style(style).unwrap();
            let edit = snapshot
                .verified_style_format(&documents, &descriptor, 0, &formatter)
                .unwrap();
            outputs(&compiled, &snapshot, &documents, cursor, lint, edit)
        }
    };
    inspect(&run);
}

pub(in crate::source_project) fn diagnostics(lint: &CssLintResult) -> serde_json::Value {
    let rows = lint
        .diagnostics
        .iter()
        .map(|d| {
            let labels = d
                .labels
                .iter()
                .map(|l| (&l.message, l.start, l.end))
                .collect::<Vec<_>>();
            serde_json::json!([
                d.rule_name,
                d.severity,
                d.message,
                d.start,
                d.end,
                d.help,
                labels,
                d.fix
            ])
        })
        .collect::<Vec<_>>();
    serde_json::json!([lint.error_count, lint.warning_count, rows])
}

fn compare(run: &dyn Fn(bool) -> Outputs) {
    let reference = run(false);
    let shared = run(true);
    assert!(shared.definition.is_some() && shared.edit.is_some());
    assert!(
        !shared.css.is_empty()
            && shared
                .classes
                .as_ref()
                .is_some_and(|c| !c.names().is_empty())
    );
    assert!(!shared.lint.diagnostics.is_empty());
    assert_eq!(shared.css, reference.css);
    assert_eq!(shared.classes, reference.classes);
    assert_eq!(diagnostics(&shared.lint), diagnostics(&reference.lint));
    assert_eq!(shared.definition, reference.definition);
    assert_eq!(shared.edit, reference.edit);
}

const ONE: &str = "$ink:red;.card{color:$ink;display:none;margin-left:4px;}";
#[test]
fn shared_original_style_preserves_every_css_lint_edit_and_definition_field() {
    paired(ONE, compare);
}

#[test]
fn a_style_lowering_refusal_preserves_rules_and_a_superseded_snapshot_skips_the_pass() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///Partial.vue").unwrap();
    let text = "<template><p/></template><style lang='scss'>.card { color:$missing; } .notice { color:red !important; margin-left:4px; }</style>";
    docs.open(uri.clone(), text.into(), 2, "vue".into());
    let source = cache.capture(&docs, &uri).unwrap();
    let arena = Allocator::new();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source.source(),
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let rules = CssLinter::with_all_rules();
    let format = FormatOptions::default();
    let glyph = GlyphFormatter::new(&format, &arena);
    let (prepared, lint) = source
        .prepare_scss_with(&docs, &descriptor, 0, |s| {
            rules.lint_file_style(StyleRuleInput::Observed(s), true)
        })
        .unwrap();
    let block = descriptor
        .admitted()
        .unwrap()
        .styles()
        .next()
        .unwrap()
        .block();
    let stock = rules.lint_block(
        block.source(),
        block.start() as usize,
        StyleLanguage::Scss,
        true,
    );
    assert!(
        stock
            .diagnostics
            .iter()
            .any(|row| row.rule_name == "css/no-important")
    );
    assert_eq!(diagnostics(&lint), diagnostics(&stock));
    assert!(prepared.products().css().is_err());
    assert!(matches!(
        prepared.verified_format(&docs, &glyph),
        Err(StyleFormatError::Style(_))
    ));
    docs.open(uri, text.into(), 2, "vue".into());
    assert!(matches!(
        prepared.verified_format(&docs, &glyph),
        Err(StyleFormatError::Host(SnapshotRefusal::Superseded))
    ));
    assert!(matches!(
        source.prepare_scss_with(&docs, &descriptor, 0, |_| panic!("superseded")),
        Err(StyleFormatError::Host(SnapshotRefusal::Superseded))
    ));
}

#[cfg(feature = "micro-profiling")]
#[test]
#[ignore = "paired optimized style tool timing with allocation measurement"]
fn measure_shared_style_queries() {
    use davinci_harness::alloc;
    use std::{hint::black_box, time::Instant};
    alloc::mark_installed();
    for count in [1, 8, 48] {
        let mut body = String::from("$ink:red;");
        for i in 0..count {
            use std::fmt::Write;
            write!(
                &mut body,
                ".card{i}{{color:$ink;display:none;margin-left:4px;}}"
            )
            .unwrap();
        }
        paired(&body, |run| {
            compare(run);
            let fresh = alloc::measure(|| run(false)).unwrap();
            let shared = alloc::measure(|| run(true)).unwrap();
            let elapsed = |mode| {
                let start = Instant::now();
                for _ in 0..8 {
                    black_box(run(black_box(mode)));
                }
                start.elapsed().as_nanos() / 8
            };
            let mut separate = Vec::new();
            let mut fused = Vec::new();
            for round in 0..35 {
                let (a, b) = if round % 2 == 0 {
                    (elapsed(false), elapsed(true))
                } else {
                    let b = elapsed(true);
                    (elapsed(false), b)
                };
                if round >= 4 {
                    separate.push(a);
                    fused.push(b);
                }
            }
            separate.sort_unstable();
            fused.sort_unstable();
            println!(
                "composed style rules={count} separate_ns={} shared_ns={} allocations={}/{} peak={}/{}",
                separate[15],
                fused[15],
                fresh.calls,
                shared.calls,
                fresh.peak_bytes_over_start,
                shared.peak_bytes_over_start
            );
            assert!(
                shared.calls < fresh.calls
                    && shared.peak_bytes_over_start <= fresh.peak_bytes_over_start
            );
            assert!(fused[15] <= separate[15]);
        });
    }
}
