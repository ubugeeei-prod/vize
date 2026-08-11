use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use vize_fresco::{
    TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
    terminal::{Color, Style},
};

fn benchmark_capability_resolution(c: &mut Criterion) {
    let probe = TerminalCapabilityProbe::new(120, 40, true)
        .with_term("xterm-256color")
        .with_colorterm("truecolor")
        .with_locale("ja_JP.UTF-8");
    c.bench_function("terminal_capabilities/resolve", |b| {
        b.iter(|| {
            black_box(TerminalCapabilities::resolve(
                black_box(&probe),
                black_box(TerminalProfileOptions::default()),
            ))
        });
    });
}

fn benchmark_style_fallback(c: &mut Criterion) {
    let profile = TerminalCapabilities::resolve(
        &TerminalCapabilityProbe::new(80, 24, true).with_term("xterm-256color"),
        TerminalProfileOptions::default(),
    );
    let style = Style::new()
        .fg(Color::Rgb(91, 143, 249))
        .bg(Color::Rgb(17, 24, 39))
        .bold()
        .underline();
    c.bench_function("terminal_capabilities/adapt_style", |b| {
        b.iter(|| black_box(profile).adapt_style(black_box(style)));
    });
}

criterion_group!(
    benches,
    benchmark_capability_resolution,
    benchmark_style_fallback
);
criterion_main!(benches);
