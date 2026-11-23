use super::tests::options;
use super::*;
use davinci_harness::alloc;
use std::{fmt::Write, hint::black_box, time::Instant};
use vize_l0::SourceBlock;
use vize_l1::container::Vue;

fn read<R>(
    block: SourceBlock<'_>,
    project: impl FnOnce(&vize_l2::query::script::assignments::CheckedAssignments<'_, '_, '_>) -> R,
) -> R {
    let arena = Allocator::new();
    let syntax = AuthoredProgram::parse(&arena, block, SourceType::ts());
    let query = LiteralAssignments::new(&syntax).unwrap();
    let rows = query.prove();
    project(&query.check(&rows).unwrap())
}

fn compare(name: &str, source: &str) {
    let arena = Allocator::new();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let block = owner.admitted().unwrap().setup().unwrap().block();
    let run = |shared| {
        let mut types = Vec::new();
        let mut lint = Vec::new();
        let mut type_product =
            |checked: &vize_l2::query::script::assignments::CheckedAssignments<'_, '_, '_>| {
                visit_binding_types(checked, |binding, d| {
                    types.push((binding, d.map(|d| d.message)))
                })
            };
        let mut lint_product =
            |checked: &vize_l2::query::script::assignments::CheckedAssignments<'_, '_, '_>| {
                default_prefer_const(&owner, checked, |d| lint.push((d.start, d.end, d.message)))
                    .unwrap()
            };
        if shared {
            read(block, |checked| {
                type_product(checked);
                lint_product(checked);
            });
        } else {
            read(block, &mut type_product);
            read(block, &mut lint_product);
        }
        (types, lint)
    };
    assert_eq!(run(false), run(true));
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let time = |shared| {
        let start = Instant::now();
        for _ in 0..16 {
            drop(black_box(run(black_box(shared))));
        }
        start.elapsed().as_nanos() / 16
    };
    let mut separate = Vec::new();
    let mut shared = Vec::new();
    for round in 0..35 {
        let (a, b) = if round % 2 == 0 {
            (time(false), time(true))
        } else {
            let b = time(true);
            (time(false), b)
        };
        if round >= 4 {
            separate.push(a);
            shared.push(b);
        }
    }
    separate.sort_unstable();
    shared.sort_unstable();
    let before = separate[15];
    let after = shared[15];
    eprintln!(
        "checked literal products {name} ns={before}/{after} allocations={}/{} peak={}/{}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    assert!(
        after <= before
            && new.calls <= old.calls
            && new.peak_bytes_over_start <= old.peak_bytes_over_start,
        "{name}"
    );
}

#[test]
#[ignore = "paired same-output production readers over original SFC script blocks"]
fn measured_separate_and_shared_verified_products() {
    alloc::mark_installed();
    compare(
        "single",
        "<script setup lang=ts>let x: string = 3;</script>",
    );
    compare(
        "mixed",
        "<script setup lang=ts>let 雪: string = 4, second: boolean = false; var x: number = 3; const yes: boolean = 9;</script>",
    );
    let mut source = String::from("<script setup lang=ts>");
    for i in 0..96 {
        writeln!(
            source,
            "let original{i}: number = {};",
            if i % 2 == 0 { "true" } else { "0x2a" }
        )
        .unwrap();
    }
    source.push_str("</script>");
    compare("96 bindings", &source);
}
