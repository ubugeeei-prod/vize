use serde_json::Value;

use crate::{
    AiContextBudget, AiContextError, AnalysisProvenance, DoctorCategory, DoctorFinding,
    DoctorReport, EvidenceKind, FindingAssessment, FindingConfidence, FindingEvidence, FindingFix,
    FindingImpact, FindingSeverity, FixSafety, HealthPenalty, RelatedLocation, RuleCost,
    SourceLocation, TextEdit, build_ai_context,
};

fn assessment(severity: FindingSeverity) -> FindingAssessment {
    FindingAssessment::new(
        severity,
        FindingConfidence::High,
        FindingImpact::High,
        HealthPenalty::new(20, "fixture"),
    )
}

fn finding(code: &str, path: &str, start: u32, end: u32) -> DoctorFinding {
    DoctorFinding::new(
        code,
        DoctorCategory::Correctness,
        assessment(FindingSeverity::Warning),
        SourceLocation::new(path, start, end),
        "Reactive value can become stale",
        "Read the value inside its reactive owner.",
        AnalysisProvenance::new("reactivity-graph", RuleCost::Low),
    )
    .with_failure_scenario("The rendered value no longer updates.")
    .with_evidence(
        FindingEvidence::new(EvidenceKind::Reactivity, "The dependency edge is missing")
            .with_location(SourceLocation::new(path, start, end))
            .with_detail("binding", "count"),
    )
    .with_related(RelatedLocation::new(
        SourceLocation::new(path, 0, 5),
        "Reactive owner",
    ))
    .with_fix(
        FindingFix::new(FixSafety::Safe, "Move the read")
            .with_edit(TextEdit::new(
                SourceLocation::new(path, start, end),
                "count.value",
            ))
            .with_verification("vize doctor --format json"),
    )
}

#[test]
fn packet_preserves_graph_source_edit_and_verification_semantics() {
    let source = "const count = ref(0);\nconst stale = count;\n";
    let start = source.find("count;\n").unwrap() as u32;
    let report = DoctorReport::new(
        "workspace",
        [finding("VIZE_AI_001", "src/App.vue", start, start + 5)],
    );

    let packet = build_ai_context(
        &report,
        [("src/App.vue", source)],
        AiContextBudget::default(),
    )
    .unwrap();

    assert_eq!(packet.findings().len(), 1);
    assert_eq!(packet.evidence_graph().nodes.len(), 3);
    assert_eq!(packet.evidence_graph().edges.len(), 2);
    assert_eq!(packet.source_snippets().len(), 2);
    assert_eq!(packet.edit_plans().len(), 1);
    assert_eq!(
        packet.edit_plans()[0].operations[0].replacement,
        "count.value"
    );
    assert_eq!(packet.edit_plans()[0].verification[0].expected_exit_code, 0);
    assert_eq!(packet.omissions(), &Default::default());

    let json = serde_json::to_string(&packet).unwrap();
    let decoded = serde_json::from_str(&json).unwrap();
    assert_eq!(packet, decoded);
}

#[test]
fn packet_is_deterministic_across_finding_and_source_input_order() {
    let a = finding("VIZE_AI_A", "src/a.ts", 0, 1);
    let b = finding("VIZE_AI_B", "src/b.ts", 0, 1);
    let left_report = DoctorReport::new("workspace", [a.clone(), b.clone()]);
    let right_report = DoctorReport::new("workspace", [b, a]);
    let budget = AiContextBudget::default();

    let left =
        build_ai_context(&left_report, [("src/a.ts", "a"), ("src/b.ts", "b")], budget).unwrap();
    let right = build_ai_context(
        &right_report,
        [("src/b.ts", "b"), ("src/a.ts", "a")],
        budget,
    )
    .unwrap();

    assert_eq!(
        serde_json::to_vec(&left).unwrap(),
        serde_json::to_vec(&right).unwrap()
    );
}

#[test]
fn budgets_account_for_every_whole_item_they_drop() {
    let primary = finding("VIZE_AI_A", "src/a.ts", 0, 2)
        .with_evidence(FindingEvidence::new(
            EvidenceKind::Source,
            "second evidence",
        ))
        .with_related(RelatedLocation::new(
            SourceLocation::new("src/a.ts", 3, 5),
            "second relation",
        ))
        .with_fix(
            FindingFix::new(FixSafety::ReviewRequired, "Two edits")
                .with_edit(TextEdit::new(SourceLocation::new("src/a.ts", 0, 1), "ok"))
                .with_edit(TextEdit::new(
                    SourceLocation::new("src/a.ts", 1, 2),
                    "too-long",
                ))
                .with_verification("ok")
                .with_verification("too-long"),
        );
    let report = DoctorReport::new(
        "workspace",
        [primary, finding("VIZE_AI_B", "src/b.ts", 0, 1)],
    );
    let budget = AiContextBudget {
        max_findings: 1,
        max_evidence_per_finding: 1,
        max_related_per_finding: 1,
        max_source_snippets: 1,
        max_source_bytes: 8,
        max_source_bytes_per_snippet: 8,
        max_edits_per_finding: 2,
        max_edit_bytes: 3,
        max_verification_steps_per_finding: 2,
        max_verification_bytes: 3,
    };

    let packet = build_ai_context(
        &report,
        [("src/a.ts", "0123456789"), ("src/b.ts", "b")],
        budget,
    )
    .unwrap();

    assert_eq!(packet.findings().len(), 1);
    assert_eq!(packet.source_snippets()[0].text.len(), 8);
    assert_eq!(packet.edit_plans()[0].operations.len(), 1);
    assert_eq!(packet.edit_plans()[0].verification.len(), 1);
    assert_eq!(packet.omissions().dropped_findings, 1);
    assert_eq!(packet.omissions().dropped_evidence_nodes, 1);
    assert_eq!(packet.omissions().dropped_related_nodes, 1);
    assert_eq!(packet.omissions().dropped_edit_operations, 1);
    assert_eq!(packet.omissions().dropped_verification_steps, 1);
}

#[test]
fn source_windows_preserve_utf8_boundaries_and_focus_metadata() {
    let source = "先頭\nconst 挨拶 = 'こんにちは世界';\n末尾\n";
    let start = source.find("こんにちは").unwrap() as u32;
    let end = start + "こんにちは".len() as u32;
    let report = DoctorReport::new(
        "workspace",
        [finding("VIZE_AI_UTF8", "src/挨拶.ts", start, end)],
    );
    let budget = AiContextBudget {
        max_source_bytes_per_snippet: 21,
        max_source_bytes: 21,
        max_source_snippets: 1,
        ..AiContextBudget::default()
    };

    let packet = build_ai_context(&report, [("src/挨拶.ts", source)], budget).unwrap();
    let snippet = &packet.source_snippets()[0];

    assert!(std::str::from_utf8(snippet.text.as_bytes()).is_ok());
    assert!(snippet.text.contains("こんにちは"));
    assert_eq!(
        snippet.relative_focus_end() - snippet.relative_focus_start(),
        "こんにちは".len() as u32
    );
    assert!(!snippet.focus_truncated);
    assert!(snippet.truncated_before);
    assert!(snippet.truncated_after);
}

#[test]
fn source_budget_smaller_than_one_scalar_omits_the_snippet() {
    let report = DoctorReport::new("workspace", [finding("VIZE_AI_UTF8", "src/App.vue", 0, 3)]);
    let budget = AiContextBudget {
        max_source_bytes: 1,
        max_source_bytes_per_snippet: 1,
        ..AiContextBudget::default()
    };

    let packet = build_ai_context(&report, [("src/App.vue", "あ")], budget).unwrap();

    assert!(packet.source_snippets().is_empty());
    assert_eq!(packet.omissions().dropped_source_snippets, 2);
}

#[test]
fn oversized_and_stale_focus_spans_are_explicitly_marked() {
    let oversized = DoctorReport::new("workspace", [finding("VIZE_AI_WIDE", "src/a.ts", 0, 10)]);
    let budget = AiContextBudget {
        max_source_snippets: 1,
        max_source_bytes: 4,
        max_source_bytes_per_snippet: 4,
        ..AiContextBudget::default()
    };
    let packet = build_ai_context(&oversized, [("src/a.ts", "0123456789")], budget).unwrap();
    let snippet = &packet.source_snippets()[0];
    assert_eq!(snippet.text, "0123");
    assert!(snippet.focus_truncated);

    let stale = DoctorReport::new("workspace", [finding("VIZE_AI_STALE", "src/a.ts", 2, 99)]);
    let packet = build_ai_context(
        &stale,
        [("src/a.ts", "short")],
        AiContextBudget {
            max_source_snippets: 1,
            ..AiContextBudget::default()
        },
    )
    .unwrap();
    assert!(packet.source_snippets()[0].focus_truncated);
}

#[test]
fn missing_sources_are_sorted_deduplicated_and_explicit() {
    let report = DoctorReport::new(
        "workspace",
        [
            finding("VIZE_AI_B", "src/b.ts", 0, 1),
            finding("VIZE_AI_A", "src/a.ts", 0, 1),
        ],
    );

    let packet = build_ai_context(&report, [], AiContextBudget::default()).unwrap();

    assert_eq!(
        packet.omissions().missing_source_paths,
        ["src/a.ts", "src/b.ts"]
    );
    assert!(packet.source_snippets().is_empty());
}

#[test]
fn duplicate_source_paths_fail_closed() {
    let report = DoctorReport::new("workspace", [finding("VIZE_AI", "src/a.ts", 0, 1)]);

    let error = build_ai_context(
        &report,
        [("src/a.ts", "a"), ("src/a.ts", "different")],
        AiContextBudget::default(),
    )
    .unwrap_err();

    assert_eq!(
        error,
        AiContextError::DuplicateSourcePath("src/a.ts".into())
    );
}

#[test]
fn wire_validation_rejects_version_reference_range_and_unknown_field_tampering() {
    let report = DoctorReport::new("workspace", [finding("VIZE_AI", "src/a.ts", 0, 1)]);
    let packet =
        build_ai_context(&report, [("src/a.ts", "abc")], AiContextBudget::default()).unwrap();
    let original = serde_json::to_value(packet).unwrap();

    let mut version = original.clone();
    version["formatVersion"] = Value::from(999);
    assert!(serde_json::from_value::<crate::AiContextPacket>(version).is_err());

    let mut edge = original.clone();
    edge["evidenceGraph"]["edges"][0]["to"] = Value::from("missing");
    assert!(serde_json::from_value::<crate::AiContextPacket>(edge).is_err());

    let mut relation = original.clone();
    relation["evidenceGraph"]["edges"][0]["relation"] = Value::from("related");
    assert!(serde_json::from_value::<crate::AiContextPacket>(relation).is_err());

    let mut range = original.clone();
    range["sourceSnippets"][0]["contentEnd"] = Value::from(999);
    assert!(serde_json::from_value::<crate::AiContextPacket>(range).is_err());

    let mut orphaned_source = original.clone();
    orphaned_source["findings"][0]["sourceSnippetIds"] = Value::Array(Vec::new());
    assert!(serde_json::from_value::<crate::AiContextPacket>(orphaned_source).is_err());

    let mut wrong_plan = original.clone();
    wrong_plan["editPlans"][0]["findingId"] = Value::from("missing");
    assert!(serde_json::from_value::<crate::AiContextPacket>(wrong_plan).is_err());

    let mut reordered = original.clone();
    reordered["evidenceGraph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    assert!(serde_json::from_value::<crate::AiContextPacket>(reordered).is_err());

    let mut unknown = original;
    unknown["vendor"] = Value::from("specific-provider");
    assert!(serde_json::from_value::<crate::AiContextPacket>(unknown).is_err());
}

#[test]
fn zero_budget_produces_a_valid_metadata_only_packet() {
    let report = DoctorReport::new("workspace", [finding("VIZE_AI", "src/a.ts", 0, 1)]);
    let budget = AiContextBudget {
        max_findings: 0,
        max_evidence_per_finding: 0,
        max_related_per_finding: 0,
        max_source_snippets: 0,
        max_source_bytes: 0,
        max_source_bytes_per_snippet: 0,
        max_edits_per_finding: 0,
        max_edit_bytes: 0,
        max_verification_steps_per_finding: 0,
        max_verification_bytes: 0,
    };
    let packet = build_ai_context(&report, [("src/a.ts", "a")], budget).unwrap();
    assert!(packet.findings().is_empty());
    assert!(packet.evidence_graph().nodes.is_empty());
    assert_eq!(packet.omissions().dropped_findings, 1);
    assert_eq!(
        serde_json::from_value::<crate::AiContextPacket>(serde_json::to_value(&packet).unwrap())
            .unwrap(),
        packet
    );
}

#[test]
fn wire_budgets_are_architecture_independent_u64_values() {
    let report = DoctorReport::new("workspace", []);
    let budget = AiContextBudget {
        max_source_bytes: u64::MAX,
        ..AiContextBudget::default()
    };
    let packet = build_ai_context(&report, [], budget).unwrap();
    let wire = serde_json::to_value(&packet).unwrap();
    assert_eq!(wire["budget"]["maxSourceBytes"], Value::from(u64::MAX));
    assert!(serde_json::from_value::<crate::AiContextPacket>(wire).is_ok());
}
