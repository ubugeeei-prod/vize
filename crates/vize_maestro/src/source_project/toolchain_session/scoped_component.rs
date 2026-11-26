use super::*;
use crate::runtime::block_on;
use crate::source_project::style_format::composition::diagnostics;
use std::sync::Arc;
use vize_atelier_sfc::NativeScssSession;
use vize_l1::css::lang::StyleLanguage;

pub(super) fn scope_the_live_style(
    docs: &DocumentStore,
    snapshots: &SourceSnapshotCache,
    before: &SourceSnapshot,
    options: NativeSfcCompileOptions<'_>,
) -> Arc<SourceSnapshot> {
    let uri = before.uri();
    let (old, _) = snapshots.begin_query(docs, uri).unwrap();
    let obsolete =
        block_on(old.run_scoped_component(docs, options, |_, session| session.emit([]))).unwrap();
    let insertion = before.source().find("<style lang=").unwrap() + "<style".len();
    let at_style = offset_to_position_str(before.source(), insertion);
    apply(
        docs,
        uri,
        6,
        Some(Range::new(at_style, at_style)),
        " scoped",
    );
    assert!(matches!(
        obsolete.publish(docs, |_| panic!("superseded output")),
        Err(SnapshotRefusal::Superseded)
    ));

    let current = snapshots.capture(docs, uri).unwrap();
    let (request, _) = snapshots.begin_query(docs, uri).unwrap();
    let options = NativeSfcCompileOptions {
        scope_id: Some("data-v-oracle"),
        ..options
    };
    let response = block_on(
        request.run_scoped_component(docs, options, |snapshot, session| {
            assert!(core::ptr::eq(snapshot, current.as_ref()));
            products(docs, snapshot, session, options)
        }),
    )
    .unwrap();
    let payload = response.publish(docs, core::convert::identity).unwrap();
    if let Some(path) = std::env::var_os("VIZE_SCOPED_SESSION_OUTPUT") {
        std::fs::write(path, serde_json::to_vec(&payload).unwrap()).unwrap();
    }

    let (query, cancel) = snapshots.begin_query(docs, uri).unwrap();
    cancel.abort();
    assert!(matches!(
        block_on(query.run_scoped_component(docs, options, |_, _| panic!("cancelled request"))),
        Err(ScopedComponentQueryError::Host(SnapshotRefusal::Cancelled))
    ));
    current
}

#[test]
fn toolchain_session_rejects_a_foreign_language_and_checks_the_revision_after_scoped_passes() {
    let docs = DocumentStore::new();
    let snapshots = SourceSnapshotCache::default();
    let uri = Url::parse("file:///pass.vue").unwrap();
    let options = NativeSfcCompileOptions::default();
    docs.open(uri.clone(), "let n = 1".into(), 1, "typescript".into());
    let (foreign, _) = snapshots.begin_query(&docs, &uri).unwrap();
    assert!(matches!(
        block_on(foreign.run_scoped_component(&docs, options, |_, _| panic!("foreign request"))),
        Err(ScopedComponentQueryError::Language)
    ));
    let source = "<template><p/></template><style scoped lang=scss>.a{color:red}</style>";
    docs.open(uri.clone(), source.into(), 2, "vue".into());
    let (query, _) = snapshots.begin_query(&docs, &uri).unwrap();
    let changed = block_on(query.run_scoped_component(&docs, options, |_, session| {
        let output = session.emit([]).unwrap();
        apply(&docs, &uri, 3, None, "<template><b/></template>");
        output
    }));
    assert!(matches!(
        changed,
        Err(ScopedComponentQueryError::Host(SnapshotRefusal::Superseded))
    ));
    let (query, cancel) = snapshots.begin_query(&docs, &uri).unwrap();
    let prepared = block_on(query.run_scoped_component(&docs, options, |_, session| {
        session.observation().l1ir().source().to_owned()
    }))
    .unwrap();
    cancel.abort();
    assert_eq!(
        prepared.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Cancelled)
    );
}

fn products(
    docs: &DocumentStore,
    snapshot: &SourceSnapshot,
    session: &NativeScssSession<'_, '_>,
    options: NativeSfcCompileOptions<'_>,
) -> Value {
    let uri = snapshot.uri();
    let owner = session.observation().l1ir();
    check_source(owner, snapshot);
    let types = snapshot
        .prepare_literal_products(docs, owner, ROLE)
        .unwrap()
        .publish(docs, Some(at(snapshot.source(), "other:")))
        .unwrap();
    assert_eq!(types.version, 6);
    assert!(types.type_errors.is_empty() && types.binding_hover.is_some());
    let rules = CssLinter::with_all_rules();
    let (style, lint) = snapshot
        .prepare_scss_with(docs, owner, 0, |original| {
            rules.lint_file_style(StyleRuleInput::Observed(original), true)
        })
        .unwrap();
    let block = style.products().block();
    let separate_lint = rules.lint_block(
        block.source(),
        block.start() as usize,
        StyleLanguage::Scss,
        true,
    );
    assert!(!lint.diagnostics.is_empty());
    assert_eq!(diagnostics(&lint), diagnostics(&separate_lint));
    let cursor = at(snapshot.source(), "$ink;");
    let Some(GotoDefinitionResponse::Scalar(def)) = snapshot
        .scss_definition(docs, style.products(), cursor)
        .unwrap()
    else {
        panic!("original scoped variable")
    };
    assert_eq!(
        (def.uri, def.range.start),
        (uri.clone(), at(snapshot.source(), "$ink:"))
    );
    assert!(
        style
            .products()
            .classes()
            .unwrap()
            .names()
            .iter()
            .map(|n| n.as_str())
            .eq(["card"])
    );
    let arena = Allocator::new();
    let format = FormatOptions::default();
    assert!(
        style
            .verified_format(docs, &GlyphFormatter::new(&format, &arena))
            .unwrap()
            .publish(docs)
            .unwrap()
            .is_none()
    );

    let output = session.emit([style.products()]).unwrap();
    let cold = NativeScssSession::new_scoped(&arena, snapshot.source(), options)
        .emit([])
        .unwrap();
    assert_eq!(output.document(), cold.document());
    assert_eq!(output.css_document(), cold.css_document());
    assert_eq!(output.source_map(), cold.source_map());
    assert_eq!(output.css_source_map(), cold.css_source_map());
    let css_doc = output.css_document().unwrap();
    assert_eq!(
        output.source_map().unwrap(),
        output
            .document()
            .source_map(options.filename, snapshot.source())
            .as_str()
    );
    assert_eq!(
        output.css_source_map().unwrap(),
        css_doc
            .source_map(options.filename, snapshot.source())
            .as_str()
    );
    assert_eq!(output.scope_id(), options.scope_id);
    assert_eq!(
        output.css(),
        Some(".card[data-v-oracle] {\n  color: red;\n  display: none;\n  margin-left: 4px;\n}")
    );
    serde_json::json!([{
        "name":options.filename, "filename":options.filename, "source":snapshot.source(),
        "trim":options.style_trim, "scopedScss":true, "code":output.code(), "css":output.css(),
        "map":serde_json::from_str::<Value>(output.source_map().unwrap()).unwrap(),
        "cssMap":serde_json::from_str::<Value>(output.css_source_map().unwrap()).unwrap()
    }])
}

#[cfg(feature = "micro-profiling")]
mod profile;

pub(super) fn current_styles(
    docs: &DocumentStore,
    snapshots: &SourceSnapshotCache,
    expected: &SourceSnapshot,
    options: NativeSfcCompileOptions<'_>,
    expected_css: &str,
) -> SourceQueryResult<vize_atelier_sfc::NativeStyleOutput> {
    let (query, _) = snapshots.begin_query(docs, expected.uri()).unwrap();
    block_on(
        query.run_vue_descriptor(docs, options.descriptor, |snapshot, owner| {
            assert!(core::ptr::eq(snapshot, expected));
            check_source(owner, snapshot);
            let rules = CssLinter::with_all_rules();
            let (style, lint) = snapshot
                .prepare_scss_with(docs, owner, 0, |original| {
                    rules.lint_file_style(StyleRuleInput::Observed(original), true)
                })
                .unwrap();
            assert!(!lint.diagnostics.is_empty());
            let cursor = at(snapshot.source(), "$ink;");
            let Some(GotoDefinitionResponse::Scalar(def)) = snapshot
                .scss_definition(docs, style.products(), cursor)
                .unwrap()
            else {
                panic!("current original definition")
            };
            assert_eq!(def.range.start, at(snapshot.source(), "$ink:"));
            let arena = Allocator::default();
            let format = FormatOptions::default();
            let verified = style
                .verified_format(docs, &GlyphFormatter::new(&format, &arena))
                .unwrap();
            assert!(verified.publish(docs).unwrap().is_none());
            let out =
                vize_atelier_sfc::compile_native_styles(owner, options.into(), [style.products()])
                    .unwrap();
            assert_eq!(out.css_document().unwrap().as_str(), expected_css);
            let map: Value = serde_json::from_str(out.css_source_map().unwrap()).unwrap();
            assert_eq!(
                map["sourcesContent"],
                serde_json::json!([snapshot.source()])
            );
            out
        }),
    )
    .unwrap()
}

#[test]
fn toolchain_session_style_only_rechecks_publication_even_with_an_external_script() {
    let docs = DocumentStore::new();
    let snapshots = SourceSnapshotCache::default();
    let uri = Url::parse("file:///partial.vue").unwrap();
    let options = NativeSfcCompileOptions {
        filename: "partial.vue",
        scope_id: Some("data-v-oracle"),
        source_map: true,
        ..Default::default()
    };
    let source = "<script src='unavailable.ts'></script>\r\n<style lang=scss scoped>$ink:red;.card{color:$ink;display:none;margin-left:4px;}</style>";
    docs.open(uri.clone(), source.into(), 1, "vue".into());
    let (query, _) = snapshots.begin_query(&docs, &uri).unwrap();
    let result = block_on(query.run_vue_descriptor(
        &docs,
        options.descriptor,
        |snapshot, owner| {
            assert!(owner.admitted().is_err());
            let (style, ()) = snapshot.prepare_scss_with(&docs, owner, 0, |_| ()).unwrap();
            let output =
                vize_atelier_sfc::compile_native_styles(owner, options.into(), [style.products()])
                    .unwrap();
            assert_eq!(
                output.css().unwrap(),
                ".card[data-v-oracle] {\n  color: red;\n  display: none;\n  margin-left: 4px;\n}"
            );
            let arena = Allocator::default();
            let options = FormatOptions::default();
            let glyph = GlyphFormatter::new(&options, &arena);
            let original = owner.admitted_styles().unwrap().styles().next().unwrap();
            let formatted = glyph.format_observed_style(original).unwrap();
            let edit = style
                .verified_format(&docs, &glyph)
                .unwrap()
                .publish(&docs)
                .unwrap()
                .unwrap();
            assert_eq!(
                (edit.text_document.uri, edit.text_document.version),
                (uri.clone(), Some(1))
            );
            let [OneOf::Left(edit)] = edit.edits.as_slice() else {
                panic!("one original stylesheet edit")
            };
            assert_eq!(
                edit.range,
                Range::new(
                    at(source, "<style"),
                    offset_to_position_str(source, source.len())
                )
            );
            assert_eq!(edit.new_text, formatted.code.as_str());
            snapshot.version()
        },
    ))
    .unwrap();
    apply(&docs, &uri, 2, None, "<template>新</template>");
    assert_eq!(
        result.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    let (query, abort) = snapshots.begin_query(&docs, &uri).unwrap();
    abort.abort();
    assert!(matches!(
        block_on(
            query.run_vue_descriptor(&docs, options.descriptor, |_, _| panic!(
                "cancelled stylesheet request"
            ))
        ),
        Err(ScopedComponentQueryError::Host(SnapshotRefusal::Cancelled))
    ));
}
