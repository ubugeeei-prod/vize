use super::*;
use crate::source_project::SourceSnapshotCache;
use tower_lsp::lsp_types::Url;
use vize_l0::{
    config::{VueDialect, VueVersion},
    line_index::LineIndex,
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_patina::{LintPreset, Linter, checked_prefer_const::RULE};

pub(super) fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}

#[test]
fn exact_stock_warnings_tsc_binding_spans_hover_and_same_text_reopen() {
    let documents = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///Actual.vue").unwrap();
    let text = "<script lang=ts>\r\nconst crab: string = '🦀'; let 雪: number = true, second: string = 'ok';\r\nvar x: boolean = 4; export let pubName: string = false;\r\n</script>";
    documents.open(uri.clone(), text.into(), 5, "vue".into());
    let first = cache.capture(&documents, &uri).unwrap();
    let arena = Allocator::new();
    let owner = Vue.observe_descriptor(&arena, first.source(), options());
    let read = || first.prepare_literal_products(&documents, &owner, ScriptRole::Ordinary);
    let index = LineIndex::new(text);
    let pos = |byte| {
        let p = index.position(byte);
        Position::new(p.line, p.character)
    };
    let hover_pos = pos(text.find('雪').unwrap());
    let out = read()
        .unwrap()
        .publish(&documents, Some(hover_pos))
        .unwrap();
    assert_eq!((out.version, out.outside), (5, 0));
    let by_name = |name: &str| {
        let at = text.find(name).unwrap();
        Range::new(pos(at), pos(at + name.len()))
    };
    let errors: Vec<_> = out
        .type_errors
        .iter()
        .map(|d| (d.range, d.code.as_ref(), d.message.as_str()))
        .collect();
    let code = NumberOrString::Number(2322);
    assert_eq!(
        errors,
        [
            (
                by_name("雪"),
                Some(&code),
                "Type 'boolean' is not assignable to type 'number'."
            ),
            (
                by_name("x"),
                Some(&code),
                "Type 'number' is not assignable to type 'boolean'."
            ),
            (
                by_name("pubName"),
                Some(&code),
                "Type 'boolean' is not assignable to type 'string'."
            )
        ]
    );
    let hover = out.binding_hover.unwrap();
    assert_eq!(hover.range, Some(by_name("雪")));
    assert_eq!(
        hover.contents,
        HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
            language: "typescript".into(),
            value: "雪: number".into()
        }))
    );
    let stock = Linter::with_preset(LintPreset::Ecosystem)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .lint_sfc(text, "Actual.vue");
    let expected: Vec<_> = stock
        .diagnostics
        .iter()
        .filter(|d| d.rule_name == RULE)
        .map(|d| {
            (
                Range::new(pos(d.start as usize), pos(d.end as usize)),
                d.message.as_str(),
            )
        })
        .collect();
    let lint = out.prefer_const.unwrap();
    assert_eq!(expected.len(), 3);
    assert_eq!(
        lint.iter()
            .map(|d| (d.range, d.message.as_str()))
            .collect::<Vec<_>>(),
        expected
    );
    let pending = read().unwrap();
    documents.close(&uri);
    documents.open(uri.clone(), text.into(), 5, "vue".into());
    assert!(matches!(
        pending.publish(&documents, None),
        Err(LiteralProductError::Host(SnapshotRefusal::Superseded))
    ));
}

#[test]
fn local_judgments_survive_a_lint_refusal_and_broken_code_withholds_results() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///Scopes.vue").unwrap();
    for (text, reason) in [
        (
            "<script setup lang=ts>let x: string = 1; x = 'later';</script>",
            PreferConstRefusal::OpenScript,
        ),
        (
            "<script setup lang=ts>let x: string = 1;</script><template><button @click=\"x++\"/></template>",
            PreferConstRefusal::Envelope,
        ),
        (
            "<script setup lang=ts>var x: string = 1; var x: string='a';</script>",
            PreferConstRefusal::BindingIdentity,
        ),
        (
            "<script setup lang=ts>/* vize-disable */ let x: string = 1;</script>",
            PreferConstRefusal::Comments,
        ),
    ] {
        docs.open(uri.clone(), text.into(), 9, "vue".into());
        let snap = cache.capture(&docs, &uri).unwrap();
        let arena = Allocator::new();
        let owner = Vue.observe_descriptor(&arena, snap.source(), options());
        let out = snap
            .prepare_literal_products(&docs, &owner, ScriptRole::Setup)
            .unwrap()
            .publish(&docs, None)
            .unwrap();
        assert_eq!(out.prefer_const.unwrap_err(), reason);
        assert_eq!(out.type_errors.len(), 1);
        let copy = text.to_owned();
        let other = Vue.observe_descriptor(&arena, &copy, options());
        assert!(matches!(
            snap.prepare_literal_products(&docs, &other, ScriptRole::Setup),
            Err(LiteralProductError::ForeignSource)
        ));
    }
    for text in [
        "<script setup lang=ts>let a: string = 1; let broken: = ;</script>",
        "<script setup lang=ts>// @ts-expect-error\nlet a: string = 1;</script>",
    ] {
        docs.open(uri.clone(), text.into(), 10, "vue".into());
        let snap = cache.capture(&docs, &uri).unwrap();
        let arena = Allocator::new();
        let owner = Vue.observe_descriptor(&arena, snap.source(), options());
        assert!(matches!(
            snap.prepare_literal_products(&docs, &owner, ScriptRole::Setup),
            Err(LiteralProductError::Script(_))
        ));
    }
}
