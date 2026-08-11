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

## Reporter integrations

External CI, editor, code-hosting, and AI integrations implement the object-safe
`DoctorReporter` trait and advertise a versioned `ReporterDescriptor`. Reporters
are installed into an explicitly owned `ReporterSet`; there is no process-global
registry for one integration to replace or reorder another. `render_report`
streams to any `std::io::Write` destination and returns the reporter identity,
format versions, finding count, and exact accepted byte count.

Descriptors declare their media type, delivery transport, intended audiences,
and the Doctor semantics preserved by the output. Registration rejects invalid
contracts and duplicate stable identifiers. Identical reports and explicit
reporter configuration must produce identical bytes.

```rust
use std::io::Write;
use vize_doctor::{
    DoctorReport, DoctorReporter, ReporterAudience, ReporterCapability,
    ReporterDescriptor, ReporterError, ReporterOutput, ReporterTransport,
    render_report,
};

struct AgentContextReporter {
    descriptor: ReporterDescriptor,
}

impl AgentContextReporter {
    fn new() -> Self {
        Self {
            descriptor: ReporterDescriptor::new(
                "example.agent-context",
                "Example agent context",
                "application/vnd.example.agent-context+json",
                ReporterTransport::Document,
            )
            .with_file_extension("json")
            .with_audiences([ReporterAudience::Ai])
            .with_capabilities([
                ReporterCapability::Findings,
                ReporterCapability::Evidence,
                ReporterCapability::Fixes,
                ReporterCapability::Provenance,
            ]),
        }
    }
}

impl DoctorReporter for AgentContextReporter {
    fn descriptor(&self) -> &ReporterDescriptor {
        &self.descriptor
    }

    fn write_report(
        &self,
        report: &DoctorReport,
        output: &mut ReporterOutput<'_>,
    ) -> Result<(), ReporterError> {
        writeln!(output, "{}", report.summary().overall_score)?;
        Ok(())
    }
}

# let report = DoctorReport::new("example", []);
let reporter = AgentContextReporter::new();
let mut bytes = Vec::new();
let receipt = render_report(&reporter, &report, &mut bytes)?;
assert_eq!(receipt.reporter_id(), "example.agent-context");
# Ok::<(), Box<dyn std::error::Error>>(())
```

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
