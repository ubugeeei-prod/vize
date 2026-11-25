//! A single open document through formatting, a broken initializer and repair.
use super::*;
use crate::{document::DocumentStore, utils::offset_to_position_str};
use serde_json::Value;
use tower_lsp::lsp_types::{
    GotoDefinitionResponse, OneOf, Position, Range, TextDocumentContentChangeEvent, Url,
};
use vize_atelier_sfc::{NativeSfcCompileOptions, compile_native_sfc};
use vize_glyph::{FormatOptions, GlyphFormatter};
use vize_l0::Allocator;
use vize_l1::container::vue::{L1Ir, ScriptRole};
use vize_l2::query::script::assignments::Verdict;
use vize_patina::{
    checked_prefer_const::PreferConstRefusal,
    rules::css::{CssLinter, StyleRuleInput},
};

const ORIGINAL: &str =
    include_str!("../../../../tests/fixtures/shared-toolchain/editor-session.vue.txt");
const REFERENCE: &str = include_str!(
    "../../../vize_atelier_sfc/tests/fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json"
);
const ROLE: ScriptRole = ScriptRole::Setup;

fn at(text: &str, marker: &str) -> Position {
    offset_to_position_str(text, text.find(marker).unwrap())
}
fn apply(docs: &DocumentStore, uri: &Url, version: i32, range: Option<Range>, text: &str) {
    assert!(docs.apply_changes(
        uri,
        vec![TextDocumentContentChangeEvent {
            range,
            range_length: None,
            text: text.into(),
        }],
        version
    ));
}
fn check_source(owner: &L1Ir<'_>, snapshot: &SourceSnapshot) {
    let mut end = 0;
    for segment in owner.container().segments() {
        let span = segment.span();
        assert_eq!(span.start, end);
        assert_eq!(span.slice(owner.source()), span.slice(snapshot.source()));
        end = span.end;
    }
    assert_eq!(end as usize, snapshot.source().len());
}
fn local_types(snapshot: &SourceSnapshot, docs: &DocumentStore, owner: &L1Ir<'_>) {
    let start = at(snapshot.source(), "other:");
    let range = Range::new(start, Position::new(start.line, start.character + 5));
    let products = snapshot
        .prepare_literal_products(docs, owner, ROLE)
        .unwrap()
        .publish(docs, Some(start))
        .unwrap();
    let [error] = products.type_errors.as_slice() else {
        panic!("one original diagnostic")
    };
    assert_eq!(error.range, range);
    assert_eq!(
        error.message,
        "Type 'boolean' is not assignable to type 'number'."
    );
    assert_eq!(products.binding_hover.unwrap().range, Some(range));
    assert!(matches!(
        products.prefer_const,
        Err(PreferConstRefusal::Envelope)
    ));
}

#[test]
fn toolchain_session_from_one_open_buffer_through_edit_error_and_repair() {
    let initial = ORIGINAL.trim_end().replace('\n', "\r\n");
    let uri = Url::parse("file:///Setup雪🌸.vue").unwrap();
    let docs = DocumentStore::new();
    docs.open(uri.clone(), initial, 1, "vue".into());
    let snapshots = SourceSnapshotCache::default();
    let source = snapshots.capture(&docs, &uri).unwrap();
    let arena = Allocator::new();
    let options = NativeSfcCompileOptions {
        filename: "Setup雪🌸.vue",
        source_map: true,
        ..Default::default()
    };
    let compiler = compile_native_sfc(&arena, source.source(), options);
    let owner = compiler.observation().descriptor();
    check_source(owner, &source);
    local_types(&source, &docs, owner);
    let rules = CssLinter::with_all_rules();
    let (style, lint) = source
        .prepare_scss_with(&docs, owner, 0, |s| {
            rules.lint_file_style(StyleRuleInput::Observed(s), true)
        })
        .unwrap();
    assert!(!lint.diagnostics.is_empty());
    let css = style.products().css().unwrap();
    assert_eq!(
        css.as_str(),
        ".card {\n  color: red;\n  display: none;\n  margin-left: 4px;\n}"
    );
    assert!(
        style
            .products()
            .classes()
            .unwrap()
            .names()
            .iter()
            .map(|name| name.as_str())
            .eq(["card"])
    );
    let cursor = at(source.source(), "$ink;");
    let Some(GotoDefinitionResponse::Scalar(def)) = source
        .scss_definition(&docs, style.products(), cursor)
        .unwrap()
    else {
        panic!("original variable")
    };
    assert_eq!(def.uri, uri);
    assert_eq!(def.range.start, at(source.source(), "$ink:"));
    let fmt = FormatOptions::default();
    let glyph = GlyphFormatter::new(&fmt, &arena);
    let verified = style.verified_format(&docs, &glyph).unwrap();
    let edit = verified.publish(&docs).unwrap().unwrap();
    assert_eq!(edit.text_document.version, Some(1));
    let [OneOf::Left(replacement)] = edit.edits.as_slice() else {
        panic!("one edit")
    };
    apply(
        &docs,
        &uri,
        2,
        Some(replacement.range),
        &replacement.new_text,
    );
    assert!(matches!(
        verified.publish(&docs),
        Err(StyleFormatError::Host(SnapshotRefusal::Superseded))
    ));
    assert!(matches!(
        style.verified_format(&docs, &glyph),
        Err(StyleFormatError::Host(SnapshotRefusal::Superseded))
    ));

    let formatted = snapshots.capture(&docs, &uri).unwrap();
    let current = compile_native_sfc(&arena, formatted.source(), options);
    let surface = current.observation().descriptor();
    check_source(surface, &formatted);
    local_types(&formatted, &docs, surface);
    let (after, ()) = formatted
        .prepare_scss_with(&docs, surface, 0, |_| ())
        .unwrap();
    assert_eq!(after.products().css().unwrap(), css);
    assert!(
        after
            .verified_format(&docs, &glyph)
            .unwrap()
            .publish(&docs)
            .unwrap()
            .is_none()
    );
    let saved = formatted
        .retain_literal_prefix(&docs, surface, ROLE)
        .unwrap();
    let tail = at(formatted.source(), "3;");
    apply(
        &docs,
        &uri,
        3,
        Some(Range::new(
            tail,
            Position::new(tail.line, tail.character + 1),
        )),
        "",
    );
    let damaged = snapshots.capture(&docs, &uri).unwrap();
    let damaged_file = compile_native_sfc(&arena, damaged.source(), options);
    let damaged_owner = damaged_file.observation().descriptor();
    check_source(damaged_owner, &damaged);
    assert!(damaged_file.result().is_err());
    #[cfg(feature = "experimental-scoped-component-session")]
    let damaged_style =
        scoped_component::current_styles(&docs, &snapshots, &damaged, options, &css);
    assert!(
        damaged
            .prepare_literal_products(&docs, damaged_owner, ROLE)
            .is_err()
    );
    let warm = saved.recover(&docs, &damaged, damaged_owner).unwrap();
    let cold = damaged
        .open_literal_prefix(&docs, damaged_owner, ROLE)
        .unwrap();
    let cursor = at(damaged.source(), "other:");
    let answer = warm.publish(&docs, cursor).unwrap();
    assert_eq!(
        answer,
        cold.recover(&docs, &damaged, damaged_owner)
            .unwrap()
            .publish(&docs, cursor)
            .unwrap()
    );
    assert_eq!(
        (answer.version, answer.bindings, answer.assignment),
        (3, 2, Some(Verdict::Mismatch))
    );
    assert!(answer.hover.is_some());
    assert!(
        warm.publish(&docs, at(damaged.source(), "final:"))
            .unwrap()
            .assignment
            .is_none()
    );

    let pack: Value = serde_json::from_str(REFERENCE).unwrap();
    let reference = pack["fixtures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "ts-annotation-mixed-mutable")
        .unwrap();
    let repaired = damaged
        .source()
        .split_once("<script")
        .unwrap()
        .1
        .replace("other:number=false", "other:number=2")
        .replace("final:number=;", "final:number=3;");
    let repaired = format!("<script{repaired}");
    assert_eq!(repaired, reference["source"].as_str().unwrap());
    apply(&docs, &uri, 4, None, &repaired);
    assert!(matches!(
        warm.publish(&docs, cursor),
        Err(recovered_script::RecoveredScriptError::Host(
            SnapshotRefusal::Superseded
        ))
    ));
    #[cfg(feature = "experimental-scoped-component-session")]
    assert_eq!(
        damaged_style.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    let final_snapshot = snapshots.capture(&docs, &uri).unwrap();
    let actual = compile_native_sfc(&arena, final_snapshot.source(), options);
    let final_owner = actual.observation().descriptor();
    check_source(final_owner, &final_snapshot);
    let final_types = final_snapshot
        .prepare_literal_products(&docs, final_owner, ROLE)
        .unwrap()
        .publish(&docs, None)
        .unwrap();
    assert!(final_types.type_errors.is_empty());
    let module = actual.result().unwrap();
    assert_eq!(module.code(), reference["code"].as_str().unwrap());
    assert_eq!(
        serde_json::from_str::<Value>(module.source_map().unwrap()).unwrap(),
        reference["nativeMap"]
    );
    #[cfg(feature = "experimental-native-scss-session")]
    let styled = styled_component::restore_formatted_style(
        &docs,
        &snapshots,
        &final_snapshot,
        &formatted,
        reference,
        options,
    );
    #[cfg(feature = "experimental-scoped-component-session")]
    let scoped = scoped_component::scope_the_live_style(&docs, &snapshots, &styled, options);
    if let Some(history) = std::env::var_os("VIZE_TOOLCHAIN_HISTORY") {
        let revisions = [
            source.source(),
            formatted.source(),
            damaged.source(),
            final_snapshot.source(),
            #[cfg(feature = "experimental-native-scss-session")]
            styled.source(),
            #[cfg(feature = "experimental-scoped-component-session")]
            scoped.source(),
        ];
        std::fs::write(history, revisions.join("\0")).unwrap();
    }
}

#[cfg(feature = "experimental-scoped-component-session")]
mod scoped_component;
mod style_corpus;
#[cfg(feature = "experimental-native-scss-session")]
mod styled_component;
