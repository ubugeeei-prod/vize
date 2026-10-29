#![expect(
    clippy::disallowed_types,
    reason = "serde fixture fields use std strings"
)]

use serde::Deserialize;
use vize_carton::Allocator;
use vize_croquis::sfc::__internal::{
    extract_and_transform_v_bind_with_context, extract_and_transform_v_bind_with_edits,
};

#[derive(Deserialize)]
struct Fixture {
    schema: String,
    captures: Vec<Capture>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Capture {
    source: String,
    scope: Option<String>,
    production_filename: Option<String>,
    source_calls: Vec<String>,
    vars: Vec<String>,
    replacements: Vec<String>,
    expected_transformed: String,
}

#[test]
fn actual_replacement_writes_preserve_all_untouched_bytes_and_existing_variables() {
    // All original seven inputs and six contexts retain complete published
    // calls/results. Whole literal replacement/output vectors supplement the
    // old-path comparison, so a shared implementation drift cannot pass.
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../tests/fixtures/css/compile-source-maps/binding-recorder-public.json"
    ))
    .unwrap();
    assert_eq!(fixture.schema, "vize.css.binding-recorder.public-oracle");
    assert_eq!(fixture.captures.len(), 42);
    for case in fixture.captures {
        let source = case.source.as_str();
        let plain_arena = Allocator::default();
        let mapped_arena = Allocator::default();
        let plain = extract_and_transform_v_bind_with_context(
            &plain_arena,
            source,
            case.scope.as_deref(),
            case.production_filename.as_deref(),
        );
        let mut edits = Vec::new();
        let mapped = extract_and_transform_v_bind_with_edits(
            &mapped_arena,
            source,
            case.scope.as_deref(),
            case.production_filename.as_deref(),
            |edit| edits.push(edit),
        );
        assert_eq!(mapped, plain);
        assert_eq!(mapped.0, case.expected_transformed);
        assert_eq!(
            mapped.1.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
            case.vars.iter().map(String::as_str).collect::<Vec<_>>()
        );
        assert_eq!(edits.len(), case.source_calls.len());
        assert_eq!(edits.len(), case.replacements.len());
        let (mut original, mut generated) = (0, 0);
        for (edit, (call, replacement)) in edits
            .iter()
            .zip(case.source_calls.iter().zip(&case.replacements))
        {
            assert_eq!(
                source.get(original..edit.source.start),
                mapped.0.get(generated..edit.generated.start)
            );
            assert_eq!(source.get(edit.source.clone()), Some(call.as_str()));
            assert_eq!(
                mapped.0.get(edit.generated.clone()),
                Some(replacement.as_str())
            );
            original = edit.source.end;
            generated = edit.generated.end;
        }
        assert_eq!(source.get(original..), mapped.0.get(generated..));
        if case.vars.is_empty() {
            assert_eq!(edits.capacity(), 0);
        }
    }
}
