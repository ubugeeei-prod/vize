//! Binding patterns keep their evaluated callee and Promise query vectors.

use serde_json::{Value, json};
use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping};
use vize_l0::String;

use super::{document::TypeAwareDocument, template_queries::collect_template_query_sets};

pub(super) fn whole_parser_errors(errors: &[vize_relief::CompilerError]) -> Value {
    let rows: Vec<_> = errors
        .iter()
        .map(|error| {
            json!({"code": format!("{:?}", error.code), "message": error.message, "loc": error.loc})
        })
        .collect();
    json!(rows)
}

pub(super) fn expected_parser_notices(id: &str, source: &str) -> Value {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/parser-compatibility-oracles.json"
    ))
    .expect("independent complete parser notices");
    let case = &packet["cases"][id];
    assert_eq!(case["source"], source);
    case["errors"].clone()
}

#[test]
fn complete_slot_query_and_promise_vectors_preserve_evaluated_values() {
    let packet: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/slot-query-direct-controls.json"
    ))
    .expect("independent full collector oracle");
    assert_eq!(
        include_str!("template_queries/calls.rs"),
        include_str!(
            "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/preparation/existing-slot-promise-calls.rs.fixture"
        ),
        "complete unchanged Promise producer"
    );
    let existing_promises: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/unsafe-template-binding/reports-8752-8755/slot-existing-promise-contract.json"
    ))
    .expect("explicit existing Promise contract correction");
    let cases = packet["cases"].as_array().expect("all cases");
    let corrected = existing_promises["cases"]
        .as_array()
        .expect("all corrected cases");
    assert_eq!(corrected.len(), cases.len());
    assert_eq!(cases.len(), 4);
    for (case, promise_contract) in cases.iter().zip(corrected) {
        let mut historical = promise_contract.clone();
        for (field, old_field) in [
            ("beforePromiseQueries", "formerAuthoredBeforePromiseQueries"),
            ("afterPromiseQueries", "formerAuthoredAfterPromiseQueries"),
        ] {
            historical[field] = historical[old_field].clone();
            historical
                .as_object_mut()
                .expect("complete case")
                .remove(old_field);
        }
        assert_eq!(&historical, case, "all original case fields remain exact");
        let allocator = vize_l0::Allocator::new();
        let (root, errors) = vize_armature::parse(
            &allocator,
            case["templateSource"].as_str().expect("whole template"),
        );
        assert_eq!(
            whole_parser_errors(&errors),
            expected_parser_notices(
                case["id"].as_str().expect("case id"),
                case["templateSource"].as_str().expect("whole template"),
            ),
            "{}: complete existing parser notices",
            case["id"]
        );
        let document = TypeAwareDocument {
            content: String::from(case["virtualDocument"].as_str().expect("whole document")),
            mapping: ProjectionMapping::from_spans(
                case["projectionMappings"]
                    .as_array()
                    .expect("all rows")
                    .iter()
                    .map(|row| {
                        assert_eq!(row["subSpans"], json!([]));
                        VizeMapping::new(
                            row["genRange"][0].as_u64().expect("generated start") as usize
                                ..row["genRange"][1].as_u64().expect("generated end") as usize,
                            row["srcRange"][0].as_u64().expect("authored start") as usize
                                ..row["srcRange"][1].as_u64().expect("authored end") as usize,
                        )
                    })
                    .collect(),
            ),
        };
        let (queries, promises) = collect_template_query_sets(
            &document,
            &root,
            case["templateOffset"].as_u64().expect("template offset") as u32,
            case["includeTemplateQueries"]
                .as_bool()
                .expect("query flag"),
            case["includeTemplatePromiseQueries"]
                .as_bool()
                .expect("promise flag"),
        );
        let actual: Vec<_> = queries
            .iter()
            .map(|query| {
                json!({
                    "kind": format!("{:?}", query.kind),
                    "context": format!("{:?}", query.context),
                    "generated_offset": query.generated_offset,
                    "source_start": query.source_start,
                    "source_end": query.source_end,
                    "owner_start": query.owner_start,
                    "owner_end": query.owner_end,
                })
            })
            .collect();
        let promise_actual: Vec<_> = promises
            .iter()
            .map(|query| {
                json!({
                    "context": format!("{:?}", query.context),
                    "generated_offset": query.generated_offset,
                    "source_start": query.source_start,
                    "source_end": query.source_end,
                })
            })
            .collect();
        assert_eq!(json!(actual), case["afterQueries"], "{}", case["id"]);
        assert_eq!(
            json!(promise_actual),
            promise_contract["afterPromiseQueries"],
            "{}: explicit existing Promise contract",
            case["id"]
        );
    }
}
