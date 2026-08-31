//! End-to-end Doctor TUI first-frame and input-to-frame benchmarks.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use vize::DoctorTuiBenchmark;
use vize_doctor::{
    AnalysisProvenance, DoctorCategory, DoctorFinding, DoctorReport, FindingAssessment,
    FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
};
use vize_fresco::{
    ColorPreference, FeaturePreference, TerminalCapabilities, TerminalCapabilityProbe,
    TerminalProfileOptions,
};

const FINDING_COUNT: usize = 10_000;
const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;

fn benchmark_first_frame(criterion: &mut Criterion) {
    let report = report();
    let capabilities = capabilities();
    let mut group = criterion.benchmark_group("doctor_tui_10k");
    group.throughput(Throughput::Elements(FINDING_COUNT as u64));
    group.bench_function("first_frame_120x40", |bencher| {
        bencher.iter(|| {
            let mut tui =
                DoctorTuiBenchmark::new(black_box(&report), WIDTH, HEIGHT, black_box(capabilities));
            black_box(tui.render())
        });
    });
    group.finish();
}

fn benchmark_input_to_frame(criterion: &mut Criterion) {
    let report = report();
    let capabilities = capabilities();
    let mut tui = DoctorTuiBenchmark::new(&report, WIDTH, HEIGHT, capabilities);
    let _ = tui.render();
    assert!(tui.materialized_findings() <= usize::from(HEIGHT));

    let mut group = criterion.benchmark_group("doctor_tui_input_to_frame_10k");
    group.throughput(Throughput::Elements(1));
    group.bench_function("selection", |bencher| {
        bencher.iter(|| black_box(tui.toggle_selection_and_render()));
    });
    group.bench_function("search", |bencher| {
        bencher.iter(|| black_box(tui.toggle_search_and_render()));
    });
    group.finish();
}

fn report() -> DoctorReport {
    DoctorReport::new(
        "benchmark",
        (0..FINDING_COUNT).map(|index| {
            DoctorFinding::new(
                "VIZE_DOCTOR_TUI_BENCH",
                DoctorCategory::Performance,
                FindingAssessment::new(
                    FindingSeverity::Warning,
                    FindingConfidence::High,
                    FindingImpact::Medium,
                    HealthPenalty::new(1, "Measured TUI presentation cost"),
                ),
                SourceLocation::new("src/Benchmark.vue", index as u32, index as u32 + 1),
                "Benchmark finding",
                "The workspace renders a deterministic evidence-rich finding.",
                AnalysisProvenance::new("benchmark-analysis", RuleCost::Low),
            )
        }),
    )
}

fn capabilities() -> TerminalCapabilities {
    TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(WIDTH, HEIGHT, true).with_locale("C.UTF-8"),
        TerminalProfileOptions {
            color: ColorPreference::Never,
            unicode: FeaturePreference::Always,
            interactive: FeaturePreference::Always,
            narrow_width: 60,
        },
    )
}

criterion_group!(benches, benchmark_first_frame, benchmark_input_to_frame);
criterion_main!(benches);
