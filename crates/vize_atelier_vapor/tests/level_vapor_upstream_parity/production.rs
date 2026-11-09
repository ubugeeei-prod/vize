//! The native compiler and upstream must retain the same trace with dev
//! branches removed and the published runtime minified.

use super::{Allocator, Fixture, VaporCompilerOptions, WalkCounts, compile_vapor, json, trace};

#[test]
fn production_branch_updates_and_disposal_match_official_vapor() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/davinci-ts33-vapor-branch.json"
    ))
    .unwrap();
    let allocator = Allocator::new();
    let before = WalkCounts::snapshot();
    let compiled = compile_vapor(
        &allocator,
        &fixture.source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    );
    assert!(
        compiled.error_messages.is_empty(),
        "{:?}",
        compiled.error_messages
    );
    assert_eq!(WalkCounts::snapshot().since(before).total_walks(), 0);
    let vize = trace(
        "davinci-mounted-trace.mjs",
        json!({
            "backend": "vapor", "code": compiled.code, "context": fixture.context,
            "steps": fixture.steps, "identities": true, "production": true,
        }),
    );
    let upstream = trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({
            "source": fixture.source, "context": fixture.context,
            "steps": fixture.steps, "production": true,
        }),
    );
    assert_eq!(vize, fixture.expected, "production native Vapor trace");
    assert_eq!(
        upstream, fixture.expected,
        "production official Vapor trace"
    );
}
