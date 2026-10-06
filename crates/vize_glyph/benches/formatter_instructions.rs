//! Instruction probes of the four original public formatter routines.
//!
//! Original Criterion source and its inputs remain unchanged. These copied
//! input bodies are qualified against that complete immutable source owner.
//! StageWindow measures the full public call and unwrap, before result drop;
//! options and the reused allocator are outside the measured window.
//! Copied input bytes have a fixed cache-line alignment so unrelated binary
//! layout changes do not move borrowed template slices between search paths.
#![expect(
    clippy::unwrap_used,
    reason = "the bench unwraps original known-good fixtures"
)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_glyph::{
    Allocator, FormatOptions, format_script, format_sfc, format_sfc_with_allocator, format_template,
};

#[repr(C, align(64))]
struct AlignedFixture<const N: usize>([u8; N]);

impl<const N: usize> AlignedFixture<N> {
    fn as_str(&'static self) -> &'static str {
        assert!(self.0.as_ptr().addr().is_multiple_of(64));
        std::str::from_utf8(&self.0).expect("original formatter fixture must be UTF-8")
    }
}

static SIMPLE_SFC: AlignedFixture<454> =
    AlignedFixture(*include_bytes!("fixtures/formatter/simple-sfc.txt"));
static LARGE_SCRIPT: AlignedFixture<1083> =
    AlignedFixture(*include_bytes!("fixtures/formatter/large-script.txt"));
static COMPLEX_TEMPLATE: AlignedFixture<1279> =
    AlignedFixture(*include_bytes!("fixtures/formatter/complex-template.txt"));

fn original_sfc(c: &mut Criterion) {
    let source = SIMPLE_SFC.as_str();
    let options = FormatOptions::default();
    bench_stage_with_metrics(
        c,
        "formatter_sfc_simple",
        "crates/vize_glyph/benches/fixtures/formatter/simple-sfc.txt",
        |window| window.measure(|| format_sfc(black_box(source), black_box(&options)).unwrap()),
    );
}

fn original_sfc_reuse(c: &mut Criterion) {
    let source = SIMPLE_SFC.as_str();
    let options = FormatOptions::default();
    let allocator = Allocator::with_capacity(8192);
    bench_stage_with_metrics(
        c,
        "formatter_sfc_reuse",
        "crates/vize_glyph/benches/fixtures/formatter/simple-sfc.txt",
        |window| {
            window.measure(|| {
                format_sfc_with_allocator(
                    black_box(source),
                    black_box(&options),
                    black_box(&allocator),
                )
                .unwrap()
            })
        },
    );
}

fn original_script(c: &mut Criterion) {
    let source = LARGE_SCRIPT.as_str();
    let options = FormatOptions::default();
    bench_stage_with_metrics(
        c,
        "formatter_script_large",
        "crates/vize_glyph/benches/fixtures/formatter/large-script.txt",
        |window| window.measure(|| format_script(black_box(source), black_box(&options)).unwrap()),
    );
}

fn original_template(c: &mut Criterion) {
    let source = COMPLEX_TEMPLATE.as_str();
    let options = FormatOptions::default();
    bench_stage_with_metrics(
        c,
        "formatter_template_complex",
        "crates/vize_glyph/benches/fixtures/formatter/complex-template.txt",
        |window| {
            window.measure(|| format_template(black_box(source), black_box(&options)).unwrap())
        },
    );
}

criterion_group!(
    probes,
    original_sfc,
    original_sfc_reuse,
    original_script,
    original_template,
);
davinci_harness::main!(probes);
