use super::super::*;
use vize_atelier_sfc::{
    NativeSfcCompileError, NativeStyleOptions, NativeStyleOutput, compile_native_styles,
};
use vize_l0::String;
use vize_l1::container::vue::L1Ir;
use vize_l1_to_l2::pass::cfg::Contribution;
use vize_maestro::source_project::SourceSnapshot;

#[derive(Debug, PartialEq, Eq)]
struct Template {
    text: String,
    decisions: Vec<Contribution>,
    syntax_errors: usize,
}

fn template<'a>(arena: &'a Allocator, owner: &L1Ir<'a>) -> Template {
    let selected = owner.admitted_template().unwrap().unwrap();
    let session = OriginalTemplate::analyze(arena, selected, &mut NoObserver).unwrap();
    let mut text = String::default();
    render(session.l1ir(), &mut |part| text.push_str(part));
    Template {
        text,
        decisions: session.analysis().complexity.contributions.clone(),
        syntax_errors: session.syntax_errors().len(),
    }
}

fn style(owner: &L1Ir<'_>, map: bool) -> Result<NativeStyleOutput, NativeSfcCompileError> {
    compile_native_styles(
        owner,
        NativeStyleOptions {
            filename: "joint.vue",
            scope_id: None,
            trim: false,
            source_map: map,
        },
        [],
    )
}

fn query<R>(
    docs: &DocumentStore,
    cache: &SourceSnapshotCache,
    uri: &Url,
    compute: impl for<'a> FnOnce(&'a SourceSnapshot, &'a Allocator, &L1Ir<'a>) -> R,
) -> SourceQueryResult<R> {
    let (query, _) = cache.begin_query(docs, uri).unwrap();
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    block_on(query.run_vue_passes(docs, options, compute)).unwrap()
}

#[test]
fn a_template_and_stylesheet_share_the_open_revision_across_edits_and_reopening() {
    #[derive(Debug, PartialEq, Eq)]
    struct Products {
        template: String,
        decisions: Vec<Contribution>,
        syntax_errors: usize,
        css: Option<String>,
    }
    let docs = DocumentStore::new();
    let cache = SourceSnapshotCache::default();
    let uri = Url::parse("file:///joint.vue").unwrap();
    let input = |body: &str, style: &str| {
        vize_l0::cstr!(
            "<script src='external.ts'></script>\r\n<template>{body}</template>\r\n{style}"
        )
        .into()
    };
    let body = "😀<p v-if='open && next'>雪</p>";
    let red = "<style>.box{color:red}</style>";
    let green = "<style>.box{color:green}</style>";
    docs.open(uri.clone(), input(body, red), 1, "vue".into());
    let current = || {
        query(&docs, &cache, &uri, |snapshot, arena, descriptor| {
            assert!(std::ptr::eq(snapshot.source(), descriptor.source()));
            assert!(descriptor.admitted().is_err());
            let result = template(arena, descriptor);
            let compiled = style(descriptor, true);
            assert_eq!(compiled.is_ok(), descriptor.admitted_styles().is_ok());
            let css = compiled.ok().and_then(|out| {
                let map: serde_json::Value =
                    serde_json::from_str(out.css_source_map().unwrap()).unwrap();
                assert_eq!(
                    map["sourcesContent"],
                    serde_json::json!([snapshot.source()])
                );
                out.css().map(String::from)
            });
            Products {
                template: result.text,
                decisions: result.decisions,
                syntax_errors: result.syntax_errors,
                css,
            }
        })
    };
    let first = current().publish(&docs, |x| x).unwrap();
    assert_eq!(first.template, body);
    assert_eq!(first.css.as_deref(), Some(".box{color:red}"));
    assert_eq!(first.syntax_errors, 0);
    let source = input(body, red);
    let decision = first
        .decisions
        .iter()
        .find(|r| r.kind == DecisionKind::If)
        .unwrap();
    assert_eq!(decision.span.slice(&source), "open && next");
    let stale = current();
    let start = source.find("red}").unwrap();
    let range = Range::new(
        offset_to_position_str(&source, start),
        offset_to_position_str(&source, start + 3),
    );
    assert!(docs.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: Some(range),
            range_length: None,
            text: "green".into(),
        }],
        2
    ));
    assert_eq!(
        stale.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    let second = current().publish(&docs, |x| x).unwrap();
    assert_eq!(
        second,
        Products {
            css: Some(".box{color:green}".into()),
            ..first
        }
    );

    let before_reopening = current();
    docs.close(&uri);
    let damaged = "<p :x=>雪</p>";
    docs.open(uri.clone(), input(damaged, green), 2, "vue".into());
    assert_eq!(
        before_reopening.publish(&docs, |_| ()),
        Err(SnapshotRefusal::Superseded)
    );
    let recovered = current().publish(&docs, |x| x).unwrap();
    assert_eq!(recovered.template, damaged);
    assert!(recovered.syntax_errors > 0);
    assert_eq!(recovered.css, second.css);
    assert!(docs.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: input(body, "<style src='outside.css'></style>"),
        }],
        3
    ));
    let external_style = current().publish(&docs, |x| x).unwrap();
    assert_eq!(
        external_style,
        Products {
            css: None,
            ..second
        }
    );
}

mod profile;
