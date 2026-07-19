# vize_doctor

`vize_doctor` defines the versioned contracts for Vize whole-application health
analysis. It gives analyzers, the CLI, editors, CI, Musea, and automated tooling
one deterministic representation for findings, evidence, fixes, provenance,
and health assessments.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## Guarantees

- authored source spans remain attached to primary and related evidence;
- severity, confidence, impact, fix safety, and analysis cost are explicit;
- contracts use language-neutral serialization;
- findings contain no timestamps or process-specific values.

## Example

```rust
use vize_doctor::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, FindingAssessment,
    FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty,
    RuleCost, SourceLocation,
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

assert_eq!(finding.code, "VIZE_DOCTOR_REACTIVITY_001");
assert_eq!(finding.primary.path, "src/Counter.vue");
```

## License

MIT
