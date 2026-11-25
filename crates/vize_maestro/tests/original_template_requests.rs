//! A genuine UTF-16 edit, incomplete source and repair on the same open URI.
#![cfg(test)]
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::NoObserver,
};
#[cfg(not(feature = "experimental-component-passes"))]
use vize_l1::container::Vue;
use vize_l1::{
    container::vue::{DescriptorOptions, L1Ir},
    markup::{SurfaceParseOptions, render},
};
use vize_l1_to_l2::pass::{cfg::DecisionKind, template::OriginalTemplate};
use vize_maestro::{
    document::DocumentStore,
    runtime::block_on,
    source_project::{SnapshotRefusal, SourceQueryResult, SourceSnapshot, SourceSnapshotCache},
    utils::offset_to_position_str,
};

#[derive(Debug, PartialEq, Eq)]
struct Response {
    version: i32,
    text: vize_l0::String,
    condition: Option<(Range, vize_l0::String)>,
    diagnostics: usize,
    whole: bool,
}

fn compute(
    docs: &DocumentStore,
    cache: &SourceSnapshotCache,
    uri: &Url,
) -> SourceQueryResult<Response> {
    let (query, _) = cache.begin_query(docs, uri).unwrap();
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    #[cfg(feature = "experimental-component-passes")]
    return block_on(query.run_vue_passes(docs, options, template_response)).unwrap();

    #[cfg(not(feature = "experimental-component-passes"))]
    block_on(query.run(docs, |snapshot| async move {
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, snapshot.source(), options);
        template_response(&snapshot, &arena, &owner)
    }))
    .unwrap()
}

fn template_response<'a>(
    snapshot: &SourceSnapshot,
    arena: &'a Allocator,
    desc: &L1Ir<'a>,
) -> Response {
    let source = snapshot.source();
    let selected = desc.admitted_template().unwrap().unwrap();
    let template = OriginalTemplate::analyze(arena, selected, &mut NoObserver).unwrap();
    let mut text = vize_l0::String::default();
    render(template.l1ir(), &mut |s| text.push_str(s));
    let condition = template
        .analysis()
        .complexity
        .contributions
        .iter()
        .find(|row| row.kind == DecisionKind::If)
        .map(|row| {
            let span = row.span;
            (
                Range::new(
                    offset_to_position_str(source, span.start as usize),
                    offset_to_position_str(source, span.end as usize),
                ),
                span.slice(source).into(),
            )
        });
    Response {
        version: snapshot.version(),
        text,
        condition,
        diagnostics: template.l2ir().diagnostics.len(),
        whole: desc.admitted().is_ok(),
    }
}

#[test]
fn independent_template_outputs_follow_actual_utf16_edits_and_reject_obsolete_publication() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///original-template.vue").unwrap();
    let prefix = "<script src='outside.ts'></script>\r\n";
    let body = "😀<b v-if='open && next'>雪</b>";
    docs.open(
        uri.clone(),
        vize_l0::cstr!("{prefix}<template>{body}</template>").into(),
        1,
        "vue".into(),
    );
    let first = compute(&docs, &cache, &uri)
        .publish(&docs, |value| value)
        .unwrap();
    let original_range = Range::new(Position::new(1, 21), Position::new(1, 33));
    assert_eq!(
        first,
        Response {
            version: 1,
            text: body.into(),
            condition: Some((original_range, "open && next".into())),
            diagnostics: 0,
            whole: false
        }
    );
    let previous = compute(&docs, &cache, &uri);
    assert!(docs.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: Some(original_range),
            range_length: Some(12),
            text: "ready".into()
        }],
        2
    ));
    assert_eq!(
        previous.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    let edited = compute(&docs, &cache, &uri).publish(&docs, |x| x).unwrap();
    assert_eq!(
        edited,
        Response {
            version: 2,
            text: "😀<b v-if='ready'>雪</b>".into(),
            condition: Some((
                Range::new(Position::new(1, 21), Position::new(1, 26)),
                "ready".into()
            )),
            diagnostics: 0,
            whole: false
        }
    );

    let replace = |text: vize_l0::String| TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: text.into(),
    };
    let damaged = "<p :x=>雪</p>";
    assert!(docs.apply_changes(
        &uri,
        vec![replace(vize_l0::cstr!(
            "{prefix}<template>{damaged}</template>"
        ))],
        3
    ));
    let broken = compute(&docs, &cache, &uri).publish(&docs, |x| x).unwrap();
    assert_eq!(
        (
            broken.version,
            broken.text.as_str(),
            broken.condition,
            broken.whole
        ),
        (3, damaged, None, false)
    );
    assert!(broken.diagnostics > 0);
    let stale = compute(&docs, &cache, &uri);
    let fixed = "<p>雪</p>";
    assert!(docs.apply_changes(
        &uri,
        vec![replace(vize_l0::cstr!("<template>{fixed}</template>"))],
        4
    ));
    assert_eq!(
        stale.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    assert_eq!(
        compute(&docs, &cache, &uri).publish(&docs, |x| x).unwrap(),
        Response {
            version: 4,
            text: fixed.into(),
            condition: None,
            diagnostics: 0,
            whole: true
        }
    );
    let (query, cancel) = cache.begin_query(&docs, &uri).unwrap();
    cancel.abort();
    let result = block_on(query.run(&docs, |_| async {
        panic!("cancelled template must not run")
    }));
    assert!(matches!(result, Err(SnapshotRefusal::Cancelled)));
}

#[cfg(feature = "experimental-component-passes")]
#[test]
fn completed_passes_recheck_cancellation_and_reopening_with_identical_bytes_and_client_version() {
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///reopened.vue").unwrap();
    let source = "<template>😀{{ value }}</template>";
    let open = || docs.open(uri.clone(), source.into(), 1, "vue".into());
    open();
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };

    let (request, stop) = cache.begin_query(&docs, &uri).unwrap();
    let ready = block_on(request.run_vue_passes(&docs, options, template_response)).unwrap();
    stop.abort();
    assert_eq!(
        ready.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Cancelled)
    );

    let before_close = compute(&docs, &cache, &uri);
    docs.close(&uri);
    open();
    assert_eq!(
        before_close.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    let fresh = compute(&docs, &cache, &uri)
        .publish(&docs, |response| response)
        .unwrap();
    assert_eq!((fresh.version, fresh.text.as_str()), (1, "😀{{ value }}"));
}

#[cfg(feature = "experimental-scoped-component-session")]
mod original_template_requests {
    mod shared;
}
