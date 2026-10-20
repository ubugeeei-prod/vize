//! Template-only parsing compared with the retained parser on identical bytes.
#![expect(clippy::expect_used, reason = "benchmarks abort on fixture errors")]

use criterion::{Criterion, criterion_group};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

const FIXTURE: &str = "crates/vize_atelier_sfc/tests/fixtures/template-only-bench.vue";
const SOURCE: &str = include_str!("../tests/fixtures/template-only-bench.vue");

fn template_only(c: &mut Criterion) {
    davinci_harness::bench_with_metrics(c, "sfc_parse_template_only", FIXTURE, || {
        parse_sfc(SOURCE, SfcParseOptions::default()).expect("native template descriptor")
    });
    davinci_harness::bench_with_metrics(c, "sfc_parse_template_only_retained", FIXTURE, || {
        vize_croquis::sfc::parse_sfc(SOURCE, SfcParseOptions::default())
            .expect("retained template descriptor")
    });
}

criterion_group!(benches, template_only);
davinci_harness::main!(benches);
