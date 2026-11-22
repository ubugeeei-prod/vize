use super::*;
mod registration;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired complete render table and dependency graph requests"]
fn paired_render_products() {
    alloc::mark_installed();
    for size in [0, 1, 8, 32, 128] {
        let mut source = String::from("<script setup>const unused=1;");
        for n in 0..size {
            source.push_str(&format!("import Card{n} from './Card{n}.vue';"));
        }
        source.push_str("</script><template><div>");
        for n in 0..size {
            source.push_str(&format!("<Card{n}/><Card{n}/>"));
        }
        source.push_str("</div></template>");
        let mut registry = ModuleRegistry::new();
        let path = Path::new("/project/App.vue");

        for n in 0..size {
            register(
                &mut registry,
                Path::new(&format!("/project/Card{n}.vue")),
                "<template/>",
            );
        }
        let arena = Allocator::new();
        let owner = observe(&arena, &source);
        let imports = owner.setup_component_imports().unwrap();
        let (_, input) = registry
            .register_setup_caller(path, analysis(path, &source), &imports)
            .unwrap();
        assert!(measure(&registry, path, &input, &size.to_string()));
    }
}

#[test]
#[ignore = "paired complete requests at original Vue source sizes"]
fn original_render_performance() {
    alloc::mark_installed();
    let mut regressions = Vec::new();
    corpus::requests(|path, registry, imports| {
        if !measure(registry, path, imports, &path.to_string_lossy()) {
            regressions.push(path.to_owned());
        }
    });
    assert!(regressions.is_empty(), "{regressions:?}");
}

fn measure(
    registry: &ModuleRegistry,
    path: &Path,
    imports: &SetupRenderInput<'_, '_, '_>,
    label: &str,
) -> bool {
    let run = |native| {
        if native {
            registry
                .setup_render_products(black_box(path), black_box(imports))
                .unwrap()
        } else {
            original(black_box(registry), black_box(path))
        }
    };
    assert_eq!(run(false), run(true));
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let time = |native, iterations| {
        let start = Instant::now();
        for _ in 0..iterations {
            drop(black_box(run(native)));
        }
        start.elapsed().as_nanos() / iterations
    };
    let pairs = sample(time, 8, false);
    eprintln!(
        "setup_render_time input={label} ns={pairs:?} calls={}/{} peak={}/{}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    if slower(&pairs) {
        let confirmation = sample(time, 64, true);
        eprintln!("setup_render_confirmation input={label} ns={confirmation:?}");
        if slower(&confirmation) {
            return false;
        }
    }
    new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start
}

fn sample(
    mut time: impl FnMut(bool, u128) -> u128,
    iterations: u128,
    reverse: bool,
) -> Vec<(u128, u128)> {
    let mut pairs = Vec::with_capacity(31);
    for round in 0..35 {
        let first = (round % 2 == 0) != reverse;
        let a = time(first, iterations);
        let b = time(!first, iterations);
        if round >= 4 {
            pairs.push(if first { (b, a) } else { (a, b) });
        }
    }
    pairs
}

// The one-sided paired sign test has probability < 0.006 under equal cost.
// Confirm a consistent slowdown independently with more iterations in reverse order.
fn slower(pairs: &[(u128, u128)]) -> bool {
    pairs.iter().filter(|(old, new)| new > old).count() >= 23
}
