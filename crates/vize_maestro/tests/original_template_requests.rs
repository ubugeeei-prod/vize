//! A genuine UTF-16 edit, incomplete source and repair on the same open URI.
use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::NoObserver,
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::{SurfaceParseOptions, render},
};
use vize_l1_to_l2::pass::{cfg::DecisionKind, template::OriginalTemplate};
use vize_maestro::{
    document::DocumentStore,
    runtime::block_on,
    source_project::{SnapshotRefusal, SourceQueryResult, SourceSnapshotCache},
    utils::offset_to_position_str,
};

#[derive(Debug, PartialEq, Eq)]
struct Response {
    version: i32,
    text: String,
    condition: Option<(Range, String)>,
    diagnostics: usize,
    whole: bool,
}

fn compute(
    docs: &DocumentStore,
    cache: &SourceSnapshotCache,
    uri: &Url,
) -> SourceQueryResult<Response> {
    let (query, _) = cache.begin_query(docs, uri).unwrap();
    block_on(query.run(docs, |snapshot| async move {
        let arena = Allocator::default();
        let options = DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        };
        let source = snapshot.source();
        let desc = Vue.observe_descriptor(&arena, source, options);
        let selected = desc.admitted_template().unwrap().unwrap();
        let template = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
        let mut text = String::new();
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
    }))
    .unwrap()
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
        format!("{prefix}<template>{body}</template>"),
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

    let replace = |text: String| TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text,
    };
    let damaged = "<p :x=>雪</p>";
    assert!(docs.apply_changes(
        &uri,
        vec![replace(format!("{prefix}<template>{damaged}</template>"))],
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
        vec![replace(format!("<template>{fixed}</template>"))],
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
