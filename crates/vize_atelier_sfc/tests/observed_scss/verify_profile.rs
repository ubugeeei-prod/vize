//! Compare two independent checks of complete SCSS observables. The
//! materialized path renders both CSS results before comparing bytes.
use davinci_harness::alloc::{self, CountingAllocator};
use davinci_harness::paired::assert_paired_time;
use std::hint::black_box;
use vize_atelier_sfc::scss::ScssProducts;
use vize_l0::SourceRoot;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn materialized(a: &ScssProducts<'_>, b: &ScssProducts<'_>) -> bool {
    let (Ok(left), Ok(right)) = (a.program(), b.program()) else {
        return false;
    };
    if a.css().ok() != b.css().ok()
        || !a.classes().is_some_and(|c| c.is_complete())
        || a.classes() != b.classes()
    {
        return false;
    }
    if left.variables().len() != right.variables().len()
        || left.variable_uses().len() != right.variable_uses().len()
    {
        return false;
    }
    for (x, y) in left.variables().iter().zip(right.variables()) {
        let (px, py) = (x.span, y.span);
        if (x.name, x.value) != (y.name, y.value)
            || a.block()
                .root_source()
                .get(px.start as usize..px.end as usize)
                != Some(x.name)
            || b.block()
                .root_source()
                .get(py.start as usize..py.end as usize)
                != Some(y.name)
        {
            return false;
        }
    }
    left.variable_uses()
        .iter()
        .zip(right.variable_uses())
        .all(|(x, y)| {
            let (px, py) = (x.span, y.span);
            x.declaration == y.declaration
                && a.block()
                    .root_source()
                    .get(px.start as usize..px.end as usize)
                    == b.block()
                        .root_source()
                        .get(py.start as usize..py.end as usize)
        })
}

fn run(label: &str, a: &str, b: &str) {
    let aa = ScssProducts::read(SourceRoot::new(a).unwrap().whole_block());
    let bb = ScssProducts::read(SourceRoot::new(b).unwrap().whole_block());
    assert!(aa.program().is_ok() && bb.program().is_ok());
    let check = |joined| {
        let (a, b) = (black_box(&aa), black_box(&bb));
        if joined {
            a.check_equivalent(b).is_ok()
        } else {
            materialized(a, b)
        }
    };
    let (a, am) = alloc::measure_returning(|| check(false));
    let (b, bm) = alloc::measure_returning(|| check(true));
    assert!(a && b);
    let (am, bm) = (am.unwrap(), bm.unwrap());
    let label = format!(
        "style certificate {label} calls={}/{} peak={}/{}",
        am.calls, bm.calls, am.peak_bytes_over_start, bm.peak_bytes_over_start
    );
    assert_eq!(bm.calls, 0);
    assert!(bm.peak_bytes_over_start <= am.peak_bytes_over_start);
    assert_paired_time(&label, 128, |candidate| {
        black_box(check(candidate));
    });
}

fn main() {
    alloc::mark_installed();
    run("single", ".x{color:red}", ".x { color: red; }");
    run(
        "bindings",
        "$tone:red;.x{color:$tone;&.on{color:$tone}}",
        "$tone: red;\n.x { color: $tone; &.on { color: $tone; } }",
    );
    let mut tight = String::from("$v: red;");
    let mut spaced = String::from("$v: red;\n");
    for i in 0..64 {
        tight.push_str(&format!(".r{i}{{color:$v}}"));
        spaced.push_str(&format!(".r{i} {{ color: $v; }}\n"));
    }
    run("64 rules", &tight, &spaced);
}
