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

## Finding filters

`DoctorFilterSpec` is a serializable, provider-neutral query shared by CLI,
TUI, editor, CI, and AI clients. Enum values within a dimension are ORed,
populated dimensions are ANDed, and string dimensions support validated shell
globs. Applying a compiled filter produces a newly scored `DoctorReport` while
leaving the source report unchanged.

```rust
use vize_doctor::{
    DoctorCategory, DoctorFilterSpec, DoctorReport, FindingSeverity,
};

# let report = DoctorReport::new("example", []);
let filter = DoctorFilterSpec {
    categories: vec![DoctorCategory::Correctness],
    severities: vec![FindingSeverity::Error],
    paths: vec!["packages/**/src/*.vue".into()],
    changed_files: vec!["packages/account/**".into()],
    ..DoctorFilterSpec::default()
}
.compile()?;
let focused = filter.apply(&report);

assert!(focused
    .findings()
    .iter()
    .all(|finding| filter.matches(finding)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Primary `paths` are intentionally distinct from `changed_files`. Changed-file
matching traverses primary, related, evidence, fix-edit, and invalidation-input
paths so dependent findings stay visible when a shared contract changes.

## Capability cache identities

`CapabilityCacheIdentity` gives each independently reusable analysis product a
domain-separated cache key. The key covers the stable capability identifier,
an implementation fingerprint, a configuration fingerprint, and the complete
set of logical input fingerprints. Input order cannot affect the key; ambiguous
or duplicate input identities fail closed. Comparing two identities reports
added, removed, and content-changed inputs separately in linear time.

```rust
use vize_doctor::{CapabilityCacheIdentity, ContentFingerprint};

let implementation = ContentFingerprint::digest("template-analyzer-v2");
let configuration = ContentFingerprint::digest("strict=true");
let previous = CapabilityCacheIdentity::from_fingerprints(
    "template-semantics",
    implementation,
    configuration,
    [("src/App.vue", ContentFingerprint::digest("before"))],
)?;
let current = CapabilityCacheIdentity::from_fingerprints(
    "template-semantics",
    implementation,
    configuration,
    [("src/App.vue", ContentFingerprint::digest("after"))],
)?;

let invalidation = current.invalidation_from(&previous);
assert_eq!(invalidation.changed_inputs(), ["src/App.vue"]);
assert_ne!(current.cache_key(), previous.cache_key());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The input set is a producer contract, not an automatic filesystem scan. A
capability must include its complete discovery boundary so a newly added or
removed source also changes the identity. Absolute paths, timestamps, and host
metadata must not be smuggled into logical identifiers or configuration hashes.

`CapabilitySnapshot` binds an identity to its normalized findings. Construction
fails unless every finding names the same capability and every provenance input
has the exact fingerprint declared by the identity. A finding that carries a
fingerprint for an input it does not declare is rejected as an orphan, so a
producer cannot widen a finding's invalidation boundary without declaring it.
Its wire form repeats the derived cache key and includes a domain-separated
streaming fingerprint of the complete normalized output, so stale keys and
accidentally corrupted cache payloads fail before findings reach scoring,
editors, reporters, or AI clients.

```rust
use std::collections::BTreeMap;
use vize_doctor::{
    AnalysisProvenance, CapabilityCacheIdentity, CapabilitySnapshot, ContentFingerprint,
    DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence, FindingImpact,
    FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
};

let source = ContentFingerprint::digest("<template><li v-for=\"item in items\" /></template>");
let identity = CapabilityCacheIdentity::from_fingerprints(
    "template-semantics",
    ContentFingerprint::digest("template-analyzer-v2"),
    ContentFingerprint::digest("strict=true"),
    [("src/App.vue", source)],
)?;

let finding = DoctorFinding::new(
    "VIZE_DOCTOR_TEMPLATE_001",
    DoctorCategory::Correctness,
    FindingAssessment::new(
        FindingSeverity::Warning,
        FindingConfidence::Certain,
        FindingImpact::Medium,
        HealthPenalty::new(10, "List render without a stable key"),
    ),
    SourceLocation::new("src/App.vue", 42, 58),
    "List render has no key",
    "Add a stable `:key` to the `v-for`.",
    AnalysisProvenance::new("template-semantics", RuleCost::Low)
        .with_invalidation_fingerprints(BTreeMap::from([("src/App.vue".into(), source)])),
);

let snapshot = CapabilitySnapshot::try_new(identity, [finding])?;
assert_eq!(snapshot.cache_key(), snapshot.identity().cache_key());
assert_ne!(snapshot.output_fingerprint(), source);

let report = snapshot.into_report("example");
assert_eq!(report.findings().len(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`execute_cached_capability` connects those contracts to execution reuse. A cache
hit is accepted only when the returned snapshot is exactly bound to the requested
identity, and the analysis runner is not invoked on a trusted hit. On a miss,
analysis output must validate as a snapshot before storage, and a miss is
reported only after the cache acknowledges the same identity and output. The
provided `MemoryCapabilitySnapshotCache` makes identical re-stores idempotent
and rejects divergent output for one cache key.

```rust
use std::{collections::BTreeMap, convert::Infallible};
use vize_doctor::{
    AnalysisProvenance, CapabilityCacheIdentity, ContentFingerprint, DoctorCategory,
    DoctorFinding, FindingAssessment, FindingConfidence, FindingImpact, FindingSeverity,
    HealthPenalty, MemoryCapabilitySnapshotCache, RuleCost, SourceLocation,
    execute_cached_capability,
};

let source = ContentFingerprint::digest("source");
let identity = CapabilityCacheIdentity::from_fingerprints(
    "template-semantics",
    ContentFingerprint::digest("template-analyzer-v2"),
    ContentFingerprint::digest("strict=true"),
    [("src/App.vue", source)],
)?;
let mut cache = MemoryCapabilitySnapshotCache::new();

let outcome = execute_cached_capability(&mut cache, identity.clone(), |_| {
    Ok::<_, Infallible>([DoctorFinding::new(
        "VIZE_DOCTOR_TEMPLATE_001",
        DoctorCategory::Correctness,
        FindingAssessment::new(
            FindingSeverity::Warning,
            FindingConfidence::Certain,
            FindingImpact::Medium,
            HealthPenalty::new(10, "List render without a stable key"),
        ),
        SourceLocation::new("src/App.vue", 42, 58),
        "List render has no key",
        "Add a stable `:key` to the `v-for`.",
        AnalysisProvenance::new("template-semantics", RuleCost::Low)
            .with_invalidation_fingerprints(BTreeMap::from([("src/App.vue".into(), source)])),
    )])
})?;

assert!(outcome.is_cache_miss());
assert_eq!(cache.len(), 1);
assert_eq!(outcome.telemetry().finding_count(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`CapabilityExecutionOutcome::telemetry()` and
`CapabilityInvalidation::telemetry()` expose deterministic cache status,
snapshot identity, output identity, finding count, and invalidation-boundary
counts for reporters, CI, and editor integrations.

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

### SARIF code-host annotations

`SarifReporter` emits OASIS SARIF 2.1.0 plus Errata 01 without depending on a
specific code host. Doctor spans are UTF-8 byte offsets, so the caller injects
the exact source text that was analyzed. This makes Unicode line and column
conversion deterministic and keeps the reporter free of filesystem access.
Missing or stale sources fail before output by default; callers must opt in to
artifact-only locations when precise annotations are intentionally unavailable.

```rust
use vize_doctor::{DoctorReport, DoctorReporter, SarifReporter, SarifSource, render_report};

let report = DoctorReport::new("example", []);
let reporter = SarifReporter::new().with_sources([
    SarifSource::new("src/App.vue", "<template><main /></template>"),
])?;
let mut sarif = Vec::new();
let receipt = render_report(&reporter, &report, &mut sarif)?;

assert_eq!(receipt.reporter_id(), "vize.sarif");
assert_eq!(reporter.descriptor().media_type(), "application/sarif+json");
# Ok::<(), Box<dyn std::error::Error>>(())
```

The CLI wires its already-discovered sources into the same reporter:

```sh
vize doctor src --format sarif > vize-doctor.sarif
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
