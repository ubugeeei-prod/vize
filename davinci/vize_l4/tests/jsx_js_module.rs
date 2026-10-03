use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::lang::js::{JsxFile, JsxFileProducer};
use vize_l3::jsx::{NativeJsxAnalysis, build_jsx_decisions};
use vize_l4::jsx::{JsxEmitErrorKind as Error, emit_js_module};
use vize_l4::write::{NoLinks, Recorded};

#[path = "jsx_js_module/window.rs"]
mod window;

fn lower<'a>(
    arena: &'a Allocator,
    source: &'a str,
    profile: SourceType,
) -> Result<JsxFile<'a>, &'static str> {
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let observation = Parser::new(arena, source, profile).parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, observation, block, 0).map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    producer.finish().map_err(|_| "finish")
}

fn analyze<'a>(
    arena: &'a Allocator,
    source: &'a str,
    profile: SourceType,
) -> Result<NativeJsxAnalysis<'a>, &'static str> {
    build_jsx_decisions(lower(arena, source, profile)?).map_err(|_| "L3")
}

fn comments(source: &str) -> Result<Vec<&str>, &'static str> {
    let arena = Allocator::default();
    let observation = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    observation.admitted().ok_or("emitted parse")?;
    observation
        .comments()
        .iter()
        .map(|comment| {
            source
                .get(comment.span.start as usize..comment.span.end as usize)
                .ok_or("emitted comment span")
        })
        .collect()
}

#[test]
fn genuine_js_modules_keep_original_comments_scalar_roles_and_complete_maps()
-> Result<(), &'static str> {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/jsx-js-module-vue-3.5.35.json"))
            .map_err(|_| "fixtures")?;
    let mut captured = Vec::new();
    for fixture in pack["fixtures"].as_array().ok_or("fixtures")? {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().ok_or("source")?;
        let analysis = analyze(&arena, source, SourceType::jsx())?;
        let original_body = analysis
            .owner()
            .observation()
            .admitted()
            .ok_or("original observation")?
            .program()
            .body
            .as_ptr();
        // Normal ownership movement cannot invalidate the owner-bound lookup.
        let mut moved = vec![analysis];
        moved.reserve(256);
        let analysis = moved.pop().ok_or("moved owner")?;
        for decision in analysis.decisions() {
            let queried = analysis
                .decision_for(decision.node())
                .ok_or("owner query")?;
            assert_eq!(queried.kind(), decision.kind());
            assert!(core::ptr::eq(queried.node().owner(), analysis.owner()));
        }
        let recorded = emit_js_module::<Recorded>(&analysis).map_err(|_| "emission")?;
        let plain = emit_js_module::<NoLinks>(&analysis).map_err(|_| "plain emission")?;
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let expected_helpers = if analysis
            .decisions()
            .any(|decision| matches!(decision.kind(), vize_l3::jsx::JsxDecisionKind::Text(_)))
        {
            2
        } else {
            1
        };
        assert_eq!(recorded.helpers.in_use_order().len(), expected_helpers);
        let document = recorded.into_document();
        for link in document.links() {
            assert!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
            );
            assert!(
                document
                    .as_str()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some()
            );
        }
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("Native雪🌸.jsx", source))
                .map_err(|_| "source map")?;
        assert_eq!(map["version"], 3);
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(
            analysis
                .owner()
                .observation()
                .admitted()
                .ok_or("custody")?
                .program()
                .body
                .as_ptr(),
            original_body
        );
        assert_eq!(
            serde_json::json!(comments(document.as_str())?),
            fixture["comments"]
        );
        captured.push(serde_json::json!({
            "id": fixture["id"], "source": source, "code": document.as_str(), "map": map,
        }));
    }
    if let Ok(path) = std::env::var("VIZE_L4_JSX_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&captured).map_err(|_| "capture JSON")?,
        )
        .map_err(|_| "capture")?;
    }
    for (actual, expected) in captured
        .iter()
        .zip(pack["fixtures"].as_array().ok_or("fixtures")?)
    {
        assert_eq!(actual["code"], expected["code"], "{}", expected["id"]);
        assert_eq!(actual["map"], expected["map"], "{}", expected["id"]);
    }
    Ok(())
}

#[test]
fn equal_source_and_numeric_positions_never_authorize_a_foreign_decision_query()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "export function render() { return <div/>; }";
    let first = analyze(&arena, source, SourceType::jsx())?;
    let second = analyze(&arena, source, SourceType::jsx())?;
    let own = first.owner().node(0).ok_or("first")?;
    let foreign = second.owner().node(0).ok_or("second")?;
    assert_eq!(own.index(), foreign.index());
    assert_eq!(own.span(), foreign.span());
    assert!(first.decision_for(own).is_some());
    assert!(first.decision_for(foreign).is_none());
    Ok(())
}

#[test]
fn unsupported_module_semantics_are_whole_typed_refusals() -> Result<(), &'static str> {
    let arena = Allocator::default();
    assert!(
        lower(
            &arena,
            "export function render(message: string) { return <div>{message}</div>; }",
            SourceType::tsx().with_module(true),
        )
        .is_err(),
        "unsupported TS syntax cannot mint a lower completed owner"
    );
    for (source, profile, expected) in [
        (
            "export function render(message) { return <div>{message}</div>; }",
            SourceType::tsx().with_module(true),
            Error::Typescript,
        ),
        (
            "'use client'; export function render() { return <div/>; }",
            SourceType::jsx(),
            Error::DirectiveOrHashbang,
        ),
        (
            "#!/usr/bin/env node\nexport function render() { return <div/>; }",
            SourceType::jsx(),
            Error::DirectiveOrHashbang,
        ),
        (
            "export function render(UI) { return <UI.Widget/>; }",
            SourceType::jsx(),
            Error::MemberTag,
        ),
        (
            "export function render() { return <my-widget/>; }",
            SourceType::jsx(),
            Error::UnknownIntrinsic,
        ),
        (
            "/** @jsx h */\nimport { h } from 'jsx-factory'; export function render() { return <div/>; }",
            SourceType::jsx(),
            Error::FactoryPragma,
        ),
        (
            "export function render() { return <search/>; }",
            SourceType::jsx(),
            Error::UnknownIntrinsic,
        ),
        (
            "export function render() { return <div /* keep */ id='x'/>; }",
            SourceType::jsx(),
            Error::TagComment,
        ),
        (
            "export function render() { return <div></div /*keep*/>; }",
            SourceType::jsx(),
            Error::TagComment,
        ),
        (
            "export function render() { return <div></ /*keep*/ div>; }",
            SourceType::jsx(),
            Error::TagComment,
        ),
        (
            "export function render() { return <div></div //keep\n>; }",
            SourceType::jsx(),
            Error::TagComment,
        ),
        (
            "function Widget() {} export function render() { return <Widget>text</Widget>; }",
            SourceType::jsx(),
            Error::ComponentChildren,
        ),
        (
            "export function render() { return <div>A&amp;B</div>; }",
            SourceType::jsx(),
            Error::TextNormalization,
        ),
        (
            "export function render() { return <div title=\"A&amp;B\"/>; }",
            SourceType::jsx(),
            Error::AttributeNormalization,
        ),
        (
            "export function render() { return <div id=\"one\" id=\"two\"/>; }",
            SourceType::jsx(),
            Error::DuplicateAttribute,
        ),
    ] {
        let arena = Allocator::default();
        let analysis = analyze(&arena, source, profile).unwrap_or_else(|error| {
            panic!("{error}: {source}");
        });
        let error = emit_js_module::<Recorded>(&analysis).unwrap_err();
        assert_eq!(error.kind, expected, "{source}");
        assert!(
            source
                .get(error.span.start as usize..error.span.end as usize)
                .is_some()
        );
        assert_eq!(analysis.owner().file().artifact().source(), source);
        assert!(emit_js_module::<NoLinks>(&analysis).is_err());
    }
    Ok(())
}
