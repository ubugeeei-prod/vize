use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::lang::js::JsxFileProducer;
use vize_l3::jsx::{JsxDecisionKind as Kind, build_jsx_decisions};
use vize_l4::jsx::emit_js_module;
use vize_l4::write::{NoLinks, Recorded};

#[test]
fn actual_raw_jsx_values_keep_complete_normalized_module_fixtures() -> Result<(), &'static str> {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/jsx-static-whitespace-vue-3.5.35.json"
    ))
    .map_err(|_| "fixtures")?;
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().ok_or("fixtures")? {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().ok_or("source")?;
        let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
        let original = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
        let body = original
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr();
        let mut producer =
            JsxFileProducer::new(&arena, original, block, 0).map_err(|_| "producer")?;
        producer.walk().map_err(|_| "sole walk")?;
        let owner = producer.finish().map_err(|_| "completed File")?;
        let analysis = build_jsx_decisions(owner).map_err(|_| "L3")?;
        let raw_texts: Vec<_> = analysis
            .decisions()
            .filter_map(|row| match row.kind() {
                Kind::Text(value) => Some(value),
                _ => None,
            })
            .collect();
        let raw_attributes: Vec<_> = analysis
            .decisions()
            .filter_map(|row| match row.kind() {
                Kind::AttributeString(value) => Some(value),
                _ => None,
            })
            .collect();
        let mut moved = vec![analysis];
        moved.reserve(128);
        let analysis = moved.pop().ok_or("moved owner")?;
        assert_eq!(
            analysis
                .owner()
                .observation()
                .admitted()
                .ok_or("retained")?
                .program()
                .body
                .as_ptr(),
            body
        );
        for decision in analysis.decisions() {
            assert!(core::ptr::eq(decision.node().owner(), analysis.owner()));
            assert_eq!(
                analysis
                    .decision_for(decision.node())
                    .ok_or("own decision")?
                    .kind(),
                decision.kind()
            );
        }
        let recorded = emit_js_module::<Recorded>(&analysis).map_err(|_| "recorded")?;
        let plain = emit_js_module::<NoLinks>(&analysis).map_err(|_| "plain")?;
        assert_eq!(recorded.text, plain.text);
        assert_eq!(recorded.helpers, plain.helpers);
        let helpers = recorded.helpers.in_use_order().len();
        let document = recorded.into_document();
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("Whitespace.jsx", source))
                .map_err(|_| "whole native map")?;
        captures.push(serde_json::json!({
            "id":fixture["id"], "code":document.as_str(), "map":map,
            "rawTexts":raw_texts, "rawAttributes":raw_attributes, "helpers":helpers,
        }));
    }
    // Strict pending fields deliberately fail until genuine exact-head hosted
    // captures are retained. This marker preserves every complete observation.
    println!(
        "native-static-whitespace-capture={}",
        serde_json::to_string(&captures).map_err(|_| "capture")?
    );
    for (actual, expected) in captures
        .iter()
        .zip(pack["fixtures"].as_array().ok_or("fixtures")?)
    {
        for field in ["rawTexts", "rawAttributes", "helpers", "code", "map"] {
            assert_eq!(actual[field], expected[field], "{} {field}", expected["id"]);
        }
    }
    Ok(())
}
