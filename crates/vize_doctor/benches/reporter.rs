use std::{hint::black_box, io};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use vize_doctor::{
    AiContextBudget, AnalysisProvenance, DoctorCategory, DoctorFilterSpec, DoctorFinding,
    DoctorReport, DoctorReporter, FindingAssessment, FindingConfidence, FindingImpact,
    FindingSeverity, HealthPenalty, JsonReporter, ReporterAudience, ReporterCapability,
    ReporterDescriptor, ReporterError, ReporterOutput, ReporterTransport, RuleCost, SarifReporter,
    SarifSource, SourceLocation, build_ai_context, render_report,
};

struct EmptyReporter {
    descriptor: ReporterDescriptor,
}

impl EmptyReporter {
    fn new() -> Self {
        Self {
            descriptor: ReporterDescriptor::new(
                "bench.empty",
                "Empty benchmark reporter",
                "application/vnd.vize.bench",
                ReporterTransport::Document,
            )
            .with_audiences([ReporterAudience::Automation])
            .with_capabilities([ReporterCapability::HealthSummary]),
        }
    }
}

impl DoctorReporter for EmptyReporter {
    fn descriptor(&self) -> &ReporterDescriptor {
        &self.descriptor
    }

    fn write_report(
        &self,
        _report: &DoctorReport,
        _output: &mut ReporterOutput<'_>,
    ) -> Result<(), ReporterError> {
        Ok(())
    }
}

fn benchmark_reporter_contract(c: &mut Criterion) {
    let report = DoctorReport::new("benchmark", []);
    let reporter = EmptyReporter::new();

    c.bench_function("doctor_reporter/contract_overhead", |b| {
        b.iter(|| {
            render_report(black_box(&reporter), black_box(&report), &mut io::sink()).unwrap()
        });
    });
}

fn benchmark_json_reporter(c: &mut Criterion) {
    let mut group = c.benchmark_group("doctor_reporter/json_compact");
    let reporter = JsonReporter::new().with_pretty(false);

    for finding_count in [0, 100, 10_000] {
        let report = report_with_findings(finding_count);
        let mut encoded = Vec::new();
        render_report(&reporter, &report, &mut encoded).unwrap();
        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(finding_count),
            &report,
            |b, report| {
                b.iter(|| {
                    render_report(black_box(&reporter), black_box(report), &mut io::sink()).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn benchmark_ai_context(c: &mut Criterion) {
    let source = "x".repeat(20_000);
    let mut group = c.benchmark_group("doctor_ai_context/build");

    for finding_count in [1, 100, 1_000] {
        let report = report_with_findings(finding_count);
        let finding_count_u64 = finding_count as u64;
        let budget = AiContextBudget {
            max_findings: finding_count_u64,
            max_source_snippets: finding_count_u64,
            max_source_bytes: finding_count_u64.saturating_mul(256),
            max_source_bytes_per_snippet: 256,
            ..AiContextBudget::default()
        };
        group.throughput(Throughput::Elements(finding_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(finding_count),
            &report,
            |b, report| {
                b.iter(|| {
                    build_ai_context(
                        black_box(report),
                        [("src/Benchmark.vue", black_box(source.as_str()))],
                        black_box(budget),
                    )
                    .unwrap()
                });
            },
        );
    }
    group.finish();
}

fn benchmark_sarif_reporter(c: &mut Criterion) {
    let mut group = c.benchmark_group("doctor_reporter/sarif_compact");
    let source = "x".repeat(10_001);
    let reporter = SarifReporter::new()
        .with_pretty(false)
        .with_sources([SarifSource::new("src/Benchmark.vue", &source)])
        .unwrap();

    for finding_count in [0, 100, 10_000] {
        let report = report_with_findings(finding_count);
        let mut encoded = Vec::new();
        render_report(&reporter, &report, &mut encoded).unwrap();
        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(finding_count),
            &report,
            |b, report| {
                b.iter(|| {
                    render_report(black_box(&reporter), black_box(report), &mut io::sink()).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn benchmark_finding_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("doctor_filter/matches");
    let filter = DoctorFilterSpec {
        categories: vec![DoctorCategory::Performance],
        severities: vec![FindingSeverity::Warning],
        rules: vec!["VIZE_DOCTOR_BENCH".into()],
        paths: vec!["src/**".into()],
        changed_files: vec!["src/Benchmark.vue".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap();

    for finding_count in [100, 10_000] {
        let report = report_with_findings(finding_count);
        group.throughput(Throughput::Elements(finding_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(finding_count),
            &report,
            |b, report| {
                b.iter(|| {
                    black_box(report)
                        .findings()
                        .iter()
                        .filter(|finding| filter.matches(finding))
                        .count()
                });
            },
        );
    }
    group.finish();
}

fn report_with_findings(finding_count: usize) -> DoctorReport {
    DoctorReport::new(
        "benchmark",
        (0..finding_count).map(|index| {
            DoctorFinding::new(
                "VIZE_DOCTOR_BENCH",
                DoctorCategory::Performance,
                FindingAssessment::new(
                    FindingSeverity::Warning,
                    FindingConfidence::High,
                    FindingImpact::Medium,
                    HealthPenalty::new(1, "Measured reporter cost"),
                ),
                SourceLocation::new("src/Benchmark.vue", index as u32, index as u32 + 1),
                "Benchmark finding",
                "The reporter preserves a complete evidence-rich finding.",
                AnalysisProvenance::new("benchmark-analysis", RuleCost::Low),
            )
        }),
    )
}

criterion_group!(
    reporter_benches,
    benchmark_reporter_contract,
    benchmark_json_reporter,
    benchmark_ai_context,
    benchmark_sarif_reporter,
    benchmark_finding_filter
);
criterion_main!(reporter_benches);
