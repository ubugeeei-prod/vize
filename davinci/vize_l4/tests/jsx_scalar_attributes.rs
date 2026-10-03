use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{Namespace, ReferenceTarget};
use vize_l2::lang::js::JsxFileProducer;
use vize_l3::jsx::{JsxDecisionKind as Kind, NativeJsxAnalysis, build_jsx_decisions};
use vize_l4::jsx::{JsxEmitErrorKind as Error, emit_js_module};
use vize_l4::write::{NoLinks, Recorded};

fn analyze<'a>(
    arena: &'a Allocator,
    source: &'a str,
    profile: SourceType,
) -> Result<NativeJsxAnalysis<'a>, &'static str> {
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let original = Parser::new(arena, source, profile).parse_observed();
    let mut producer =
        JsxFileProducer::new(arena, original, block, 0).map_err(|_| "original producer")?;
    producer.walk().map_err(|_| "sole walk")?;
    let owner = producer.finish().map_err(|_| "completed File")?;
    build_jsx_decisions(owner).map_err(|_| "L3")
}

#[test]
fn complete_scalar_attribute_modules_keep_original_owner_reads_maps_and_plain_output()
-> Result<(), &'static str> {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/jsx-scalar-attributes-vue-3.5.35.json"
    ))
    .map_err(|_| "fixtures")?;
    let mut captured = Vec::new();
    for fixture in pack["fixtures"].as_array().ok_or("fixtures")? {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().ok_or("source")?;
        let analysis = analyze(&arena, source, SourceType::jsx())?;
        let body = analysis
            .owner()
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr();
        let mut moved = vec![analysis];
        moved.reserve(256);
        let analysis = moved.pop().ok_or("moved")?;
        let mut attribute_names = Vec::new();
        let mut read_names = Vec::new();
        for row in analysis.decisions() {
            assert!(core::ptr::eq(row.node().owner(), analysis.owner()));
            assert_eq!(
                analysis.decision_for(row.node()).ok_or("own query")?.kind(),
                row.kind()
            );
            match row.kind() {
                Kind::ExpressionAttribute { name } => attribute_names.push(name),
                Kind::Read { name, binding } => {
                    read_names.push(name);
                    let [reference] = row.node().references().ok_or("reference")? else {
                        return Err("one original read");
                    };
                    assert_eq!(reference.target, ReferenceTarget::Resolved(binding));
                    assert_eq!(
                        analysis
                            .owner()
                            .file()
                            .lookup(reference.scope, name, Namespace::Value)
                            .ok_or("actual File binding")?
                            .id(),
                        binding
                    );
                }
                _ => {}
            }
        }
        let recorded = emit_js_module::<Recorded>(&analysis).map_err(|_| "recorded")?;
        let plain = emit_js_module::<NoLinks>(&analysis).map_err(|_| "plain")?;
        assert_eq!(recorded.text, plain.text);
        assert_eq!(recorded.helpers, plain.helpers);
        let helpers = recorded.helpers.in_use_order().len();
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
            serde_json::from_str(&document.source_map("Scalar.jsx", source))
                .map_err(|_| "complete map")?;
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
        captured.push(serde_json::json!({
            "id":fixture["id"], "code":document.as_str(), "map":map,
            "attributeNames":attribute_names, "readNames":read_names, "helpers":helpers,
        }));
    }
    // Recover every genuine complete hosted observation before strict fields.
    println!(
        "native-scalar-attributes-capture={}",
        serde_json::to_string(&captured).map_err(|_| "capture")?
    );
    for (actual, fixture) in captured
        .iter()
        .zip(pack["fixtures"].as_array().ok_or("fixtures")?)
    {
        for field in ["attributeNames", "readNames", "helpers", "code", "map"] {
            assert_eq!(actual[field], fixture[field], "{} {field}", fixture["id"]);
        }
    }
    Ok(())
}

#[test]
fn mixed_duplicate_and_attribute_comment_forms_keep_exact_target_refusals()
-> Result<(), &'static str> {
    for (source, expected) in [
        (
            "export function render(x){return <div id='fixed' id={x}/>;}",
            Error::DuplicateAttribute,
        ),
        (
            "export function render(x){return <div id={x} id='fixed'/>;}",
            Error::DuplicateAttribute,
        ),
        (
            "export function render(x){return <div class='fixed' class={x}/>;}",
            Error::DuplicateAttribute,
        ),
        (
            "export function render(x){return <div class={x} class='fixed'/>;}",
            Error::DuplicateAttribute,
        ),
        (
            "export function render(x){return <div title={/* attribute */x}/>;}",
            Error::TagComment,
        ),
        (
            "export function render(x){return <div id={x} id={x}/>;}",
            Error::DuplicateAttribute,
        ),
        (
            "export function render(x){return <div title={x/* tail */}/>;}",
            Error::TagComment,
        ),
        (
            "export function render(x){return <div title={x// tail\n}/>;}",
            Error::TagComment,
        ),
    ] {
        let arena = Allocator::default();
        let analysis = analyze(&arena, source, SourceType::jsx())?;
        for recording in [false, true] {
            let kind = if recording {
                emit_js_module::<Recorded>(&analysis)
                    .err()
                    .ok_or("recorded refusal")?
                    .kind
            } else {
                emit_js_module::<NoLinks>(&analysis)
                    .err()
                    .ok_or("plain refusal")?
                    .kind
            };
            assert_eq!(kind, expected);
        }
        assert!(analysis.owner().file().is_complete());
    }
    Ok(())
}

#[test]
fn genuine_tsx_attribute_owner_does_not_gain_runtime_typescript_erasure() -> Result<(), &'static str>
{
    let arena = Allocator::default();
    let typed = "export function render(label: string){return <div title={label}/>;}";
    assert!(analyze(&arena, typed, SourceType::tsx().with_module(true)).is_err());
    let source = "export function render(label){return <div title={label}/>;}";
    let analysis = analyze(&arena, source, SourceType::tsx().with_module(true))?;
    assert!(
        analysis
            .decisions()
            .any(|row| row.kind() == Kind::ExpressionAttribute { name: "title" })
    );
    assert_eq!(
        emit_js_module::<Recorded>(&analysis)
            .err()
            .ok_or("TS refusal")?
            .kind,
        Error::Typescript
    );
    assert_eq!(
        emit_js_module::<NoLinks>(&analysis)
            .err()
            .ok_or("plain TS refusal")?
            .kind,
        Error::Typescript
    );
    assert!(analysis.owner().file().is_complete());
    Ok(())
}
