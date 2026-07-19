# vize_doctor

`vize_doctor` defines the versioned contracts for Vize whole-application health
analysis. It gives analyzers, the CLI, editors, CI, Musea, and automated tooling
one deterministic representation for findings, evidence, fixes, provenance,
scores, and health gates.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## Guarantees

- authored source spans remain attached to primary and related evidence;
- severity, confidence, impact, fix safety, and analysis cost are explicit;
- findings and nested evidence are normalized into deterministic order;
- reports use versioned, language-neutral serialization;
- default health gates never block on low-confidence findings;
- report output contains no timestamps or process-specific values.

## Application analysis

The `application-analysis` feature is disabled by default so consumers that
only exchange finding and report contracts do not link the analysis engine.
When enabled, the adapter converts an existing Vize whole-project analysis into
source-aware findings and reports without reparsing files. It fails closed if a
diagnostic references a stale file or a path outside the declared workspace.

## Example

```rust
use vize_doctor::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, DoctorReport,
    FindingAssessment, FindingConfidence, FindingImpact, FindingSeverity,
    HealthPenalty, RuleCost, SourceLocation,
};

let finding = DoctorFinding::new(
    "VIZE_DOCTOR_REACTIVITY_001",
    DoctorCategory::Correctness,
    FindingAssessment::new(
        FindingSeverity::Error,
        FindingConfidence::Certain,
        FindingImpact::High,
        HealthPenalty::new(30, "Proven stale state read"),
    ),
    SourceLocation::new("src/Counter.vue", 120, 132),
    "State is read outside its reactive owner",
    "Move the read into the derived computation that owns the dependency.",
    AnalysisProvenance::new("reactivity-graph", RuleCost::Low),
);

let report = DoctorReport::new("example", [finding]);
assert!(report.summary().has_blocking_errors);
```

## License

MIT
