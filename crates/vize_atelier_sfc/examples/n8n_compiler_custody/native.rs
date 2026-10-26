//! Test-only complete code beside retained diagnostics, without product admission.
use serde_json::{Value, json};
use vize_l0::{
    Allocator, Span,
    diag::{Diagnostic, Stage},
    pass::BudgetObserver,
};
use vize_l1_to_l2::{DomEmitOptions, LegacyCaps, emit_dom_with_options, lower_with_caps};

fn diagnostic(value: &Diagnostic) -> Value {
    json!({
        "code": if value.message == vize_l1_to_l2::lower::SAME_KEY_MESSAGE { "VIfSameKey" } else { "NativeDiagnostic" },
        "message": value.message, "span": {"start": value.span.start, "end": value.span.end},
        "stage": format!("{:?}", value.stage), "severity": format!("{:?}", value.severity()),
        "parts": value.parts.iter().map(|part| json!({"kind": format!("{:?}", part.kind),
            "span": {"start": part.span.start, "end": part.span.end}, "message": part.message})).collect::<Vec<_>>(),
        "witness": format!("{:?}", value.witness()), "completeDebug": format!("{value:#?}")
    })
}

pub(super) fn expected_contract(coordinates: &Value) -> Result<Value, Box<dyn std::error::Error>> {
    let mut legacy = Vec::new();
    let mut native = Vec::new();
    for value in coordinates
        .as_array()
        .ok_or("complete ordered diagnostic contract")?
    {
        let start = value["span"]["start"]
            .as_u64()
            .ok_or("diagnostic byte start")? as u32;
        let end = value["span"]["end"].as_u64().ok_or("diagnostic byte end")? as u32;
        let message = value["message"].as_str().ok_or("diagnostic message")?;
        legacy.push(json!({"code": value["code"], "message": message,
            "location": vize_atelier_core::SourceLocation::new(start,end)}));
        // Complete typed diagnostic metadata uses the existing declared witness;
        // the ordered count and coordinates come from the frozen independent Vue oracle.
        native.push(diagnostic(&Diagnostic::legacy_error(
            &vize_l1_to_l2::exemptions::LOWERING,
            Stage::Semantic,
            Span::new(start, end),
            message,
        )));
    }
    Ok(
        json!({"coordinates":coordinates,"legacyErrors":legacy,"nativeDiagnostics":native,
        "defaultProduction":"refused","prefixedProduction":"returned"}),
    )
}

pub(super) fn capture(source: &str, options: &DomEmitOptions<'_>) -> Value {
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, source);
    let mut lowered = lower_with_caps(&allocator, &tree, &errors, LegacyCaps::VUE3);
    let mut profile = vize_l1_to_l2::pass::TransformProfile::DEFAULT;
    if !options.hoist_static {
        profile = profile.without_static_analysis();
    }
    let facts = vize_l1_to_l2::pass::run_dom_transform_with_profile(
        &mut lowered,
        &mut BudgetObserver::new(),
        profile,
    );
    let diagnostics = lowered
        .diagnostics
        .iter()
        .map(diagnostic)
        .collect::<Vec<_>>();
    let production = emit_dom_with_options(&lowered, &facts, options);
    let production = match production {
        Ok(value) => json!({"status": "returned", "assembled": value.assembled()}),
        Err(error) => json!({"status": "refused", "error": format!("{error:#?}")}),
    };
    // The exact original still rejects in the ordinary no-prefix product path.
    // This feature-required example retains its full output for diagnostic parity;
    // it never grants successful-module or runtime credit to this packet.
    let diagnosed_emission = if !lowered.diagnostics.is_empty()
        && lowered
            .diagnostics
            .iter()
            .all(|value| value.message == vize_l1_to_l2::lower::SAME_KEY_MESSAGE)
    {
        let retained = std::mem::take(&mut lowered.diagnostics);
        let result = emit_dom_with_options(&lowered, &facts, options);
        lowered.diagnostics = retained;
        match result {
            Ok(value) => json!({"status": "diagnosed-output", "assembled": value.assembled()}),
            Err(error) => json!({"status": "refused", "error": format!("{error:#?}")}),
        }
    } else {
        Value::Null
    };
    let effective = if production.get("status").and_then(Value::as_str) == Some("returned") {
        Vec::new()
    } else {
        diagnostics.clone()
    };
    json!({"assembled": production.get("assembled"), "error": production.get("error"),
        "production": production, "loweredDiagnostics": diagnostics,
        "effectiveDiagnostics": effective,
        "diagnosedEmission": diagnosed_emission})
}
