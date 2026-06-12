//! Native Rust benchmarks for JSX/TSX compilation performance.
//!
//! Mirrors `vize_atelier_sfc`'s `sfc_compile` bench: most cases measure the
//! full parse + lower + compile pipeline (the whole call lives inside
//! `b.iter`, with no parse hoisting), tagged with `Throughput::Bytes` so
//! criterion reports MB/s.
//!
//! Together with `vize_patina`'s `markup_ir_bench` (`jsx_lint` group) these
//! groups split the JSX cost surface into the four dimensions #1501 requires so
//! a regression points at the right stage:
//!
//! - `jsx_lower` — parse + lower to the shared relief IR (parser/lowering).
//! - `jsx_croquis_analyze` — Croquis semantic analysis (binding/scope/
//!   reactivity) over an already-parsed program, isolated from lowering and
//!   codegen so this group moves only when the analysis itself does.
//! - `jsx_compile_dom` / `jsx_compile_vapor` / `jsx_compile_mode_aware` —
//!   VDOM / Vapor backend codegen.
//! - (Patina rule traversal lives in `vize_patina`'s `jsx_lint` group.)
//!
//! Run with: cargo bench -p vize_atelier_jsx --bench jsx_compile

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use vize_atelier_jsx::{
    DomCompileOptions, JsxCompileConfig, JsxLang, VaporCompileOptions, analyze_jsx_program,
    compile_jsx, compile_to_dom, compile_to_vapor, lower_source, parse_module,
};
use vize_carton::Bump;

/// A minimal single-element component.
const SMALL_JSX: &str = r#"const App = () => <div class="hero">{title}</div>;"#;

/// A medium component exercising attributes, interpolation, a `.map` list, a
/// conditional, and an event handler.
const MEDIUM_JSX: &str = r#"const Dashboard = () => (
  <section class="dashboard" id={dashboardId}>
    <header class="topbar">
      <h1>{title}</h1>
      <button class="refresh" onClick={refresh}>{loading ? "Refreshing" : "Refresh"}</button>
    </header>
    <ul class="metrics">
      {metrics.map((metric) => (
        <li key={metric.id} class="metric">
          <span class="label">{metric.label}</span>
          <strong class="value">{metric.value}</strong>
        </li>
      ))}
    </ul>
    {hasFooter && <footer class="footer">{footerText}</footer>}
  </section>
);"#;

/// A TSX variant of the medium component: typed signatures plus the same
/// attribute / interpolation / list / conditional / handler surface.
const MEDIUM_TSX: &str = r#"interface Metric {
  id: number;
  label: string;
  value: string;
}

const Dashboard = (): JSX.Element => (
  <section class="dashboard" id={dashboardId}>
    <header class="topbar">
      <h1>{title}</h1>
      <button class="refresh" onClick={refresh}>{loading ? "Refreshing" : "Refresh"}</button>
    </header>
    <ul class="metrics">
      {metrics.map((metric: Metric) => (
        <li key={metric.id} class="metric">
          <span class="label">{metric.label}</span>
          <strong class="value">{metric.value}</strong>
        </li>
      ))}
    </ul>
    {hasFooter && <footer class="footer">{footerText}</footer>}
  </section>
);"#;

/// The cases shared across every benchmark group: `(name, source, lang)`.
const CASES: &[(&str, &str, JsxLang)] = &[
    ("small_jsx", SMALL_JSX, JsxLang::Jsx),
    ("medium_jsx", MEDIUM_JSX, JsxLang::Jsx),
    ("medium_tsx", MEDIUM_TSX, JsxLang::Tsx),
];

fn bench_lower(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsx_lower");
    for &(name, source, lang) in CASES {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let bump = Bump::new();
                let out = lower_source(&bump, black_box(source), black_box(lang));
                black_box(out);
            });
        });
    }
    group.finish();
}

/// Croquis semantic analysis only, isolated from parsing and lowering.
///
/// Unlike the other groups, the parse is hoisted out of `b.iter`: we parse each
/// fixture once with the right JSX/TSX dialect, then time only
/// [`analyze_jsx_program`] over the already-parsed program. That is the Croquis
/// binding/scope/reactivity pass Patina's zero-cost `lint_jsx` and the lowering
/// backends both consume, so this group attributes regressions to the analysis
/// stage rather than the parser or the relief lowering.
fn bench_croquis_analyze(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsx_croquis_analyze");
    for &(name, source, lang) in CASES {
        // Parse once outside the timed region; the OXC program borrows this
        // allocator, so both must outlive `b.iter`.
        let allocator = oxc_allocator::Allocator::default();
        let parsed = parse_module(&allocator, source, lang);
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let croquis = analyze_jsx_program(black_box(&parsed.program), black_box(source));
                black_box(croquis);
            });
        });
    }
    group.finish();
}

fn bench_compile_dom(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsx_compile_dom");
    for &(name, source, lang) in CASES {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let bump = Bump::new();
                let out = compile_to_dom(
                    &bump,
                    black_box(source),
                    black_box(lang),
                    DomCompileOptions::default(),
                );
                black_box(out);
            });
        });
    }
    group.finish();
}

fn bench_compile_vapor(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsx_compile_vapor");
    for &(name, source, lang) in CASES {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let bump = Bump::new();
                let out = compile_to_vapor(
                    &bump,
                    black_box(source),
                    black_box(lang),
                    VaporCompileOptions::default(),
                );
                black_box(out);
            });
        });
    }
    group.finish();
}

fn bench_compile_mode_aware(c: &mut Criterion) {
    let config = JsxCompileConfig::default();
    let mut group = c.benchmark_group("jsx_compile_mode_aware");
    for &(name, source, lang) in CASES {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                let bump = Bump::new();
                let out = compile_jsx(&bump, black_box(source), black_box(lang), &config);
                black_box(out);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_lower,
    bench_croquis_analyze,
    bench_compile_dom,
    bench_compile_vapor,
    bench_compile_mode_aware,
);
criterion_main!(benches);
