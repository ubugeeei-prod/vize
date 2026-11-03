use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::lang::js::JsxFileProducer;
use vize_l3::jsx::{JsxDecisionKind as Decision, build_jsx_decisions};
use vize_l4::jsx::emit_js_module;
use vize_l4::write::{NoLinks, Recorded};

#[test]
fn genuine_original_expressions_keep_complete_source_roles_and_module_execution_fixture()
-> Result<(), &'static str> {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/jsx-original-expression-vue-3.5.35.json"
    ))
    .map_err(|_| "fixtures")?;
    let mut captured = Vec::new();
    for fixture in pack["fixtures"].as_array().ok_or("fixtures")? {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().ok_or("source")?;
        let block = SourceRoot::new(source)
            .map_err(|_| "source root")?
            .whole_block();
        let original = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
        let body = original
            .admitted()
            .ok_or("original parse")?
            .program()
            .body
            .as_ptr();
        let mut producer =
            JsxFileProducer::new(&arena, original, block, 0).map_err(|_| "producer")?;
        producer.walk().map_err(|_| "sole walk")?;
        let owner = producer.finish().map_err(|_| "completed owner")?;
        let analysis = build_jsx_decisions(owner).map_err(|_| "L3")?;
        let actual: Vec<_> = analysis
            .decisions()
            .filter(|row| row.kind() == Decision::OriginalExpression)
            .map(|row| row.node().source().ok_or("decision source"))
            .collect::<Result<_, _>>()?;
        let mut moved = vec![analysis];
        moved.reserve(128);
        let analysis = moved.pop().ok_or("moved owner")?;
        assert_eq!(
            analysis
                .owner()
                .observation()
                .admitted()
                .ok_or("retained parse")?
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
                    .ok_or("own row")?
                    .kind(),
                decision.kind()
            );
        }
        let recorded = emit_js_module::<Recorded>(&analysis).map_err(|_| "recorded module")?;
        let plain = emit_js_module::<NoLinks>(&analysis).map_err(|_| "plain module")?;
        assert_eq!(recorded.text, plain.text);
        assert_eq!(recorded.helpers, plain.helpers);
        let doc = recorded.into_document();
        let map: serde_json::Value = serde_json::from_str(&doc.source_map("Original.jsx", source))
            .map_err(|_| "native map")?;
        captured.push(serde_json::json!({
            "id":fixture["id"], "code":doc.as_str(), "map":map, "originalExpressions":actual,
        }));
    }
    // Complete genuine captures are retained in failed-test stdout/JUnit on
    // Actions while fixture generation is pending; assertions remain exact.
    println!(
        "native-original-expression-capture={}",
        serde_json::to_string(&captured).map_err(|_| "capture JSON")?
    );
    if let Ok(path) = std::env::var("VIZE_L4_JSX_ORIGINAL_CAPTURE") {
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
        assert_eq!(
            actual["originalExpressions"], expected["originalExpressions"],
            "{}",
            expected["id"]
        );
        assert_eq!(actual["code"], expected["code"], "{}", expected["id"]);
        assert_eq!(actual["map"], expected["map"], "{}", expected["id"]);
    }
    Ok(())
}
