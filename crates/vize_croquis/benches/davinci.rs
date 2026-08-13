//! Davinci microbenches: descriptor-level Croquis analysis over the P0-2
//! fixture ladder.
//!
//! Run with: cargo bench -p vize_croquis --bench davinci
//!
//! The measured entry point is `vize_atelier_sfc::croquis::analyze_sfc_descriptor`
//! - the exact orchestration the compiler, linter, and type checker call - in
//! its `SfcCroquisOptions::full()` and `::for_compile()` presets. SFC block
//! splitting and template parsing happen once per fixture outside the timed
//! window, so the numbers isolate analysis cost (script analysis + template
//! scope walk + finish), not parsing. The default dialect flags apply
//! (no Options-API resolution, no legacy Vue 2 helpers); medium.vue is an
//! Options-API component and its script takes the plain-script path here,
//! which is the same treatment the standard Vue 3 pipeline gives it
//! without dialect hints.

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::LADDER;
use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{Bump, cstr};

fn davinci(criterion: &mut Criterion) {
    for fixture in &LADDER {
        let descriptor = parse_sfc(fixture.source, SfcParseOptions::default())
            .expect("every ladder fixture is a well-formed SFC");
        let allocator = Bump::new();
        let template_ast = descriptor.template.as_ref().map(|block| {
            let (root, _errors) = Parser::new(&allocator, block.content.as_ref()).parse();
            root
        });

        let presets = [
            ("full", SfcCroquisOptions::full()),
            ("for_compile", SfcCroquisOptions::for_compile()),
        ];
        for (preset_name, options) in presets {
            let bench_id = cstr!("croquis_analyze_{}_{}", preset_name, fixture.name);
            davinci_harness::bench_with_metrics(
                criterion,
                &bench_id,
                fixture.relative_path,
                || analyze_sfc_descriptor(&descriptor, template_ast.as_ref(), options),
            );
        }
    }
}

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
