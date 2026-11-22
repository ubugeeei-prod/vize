use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

fn profile(
    state: &ServerState,
    uri: &Url,
    source: &str,
    label: &str,
    repetitions: u128,
) -> (bool, bool) {
    let arena = Allocator::new();
    let owner = observe(&arena, source);
    let retained = owner.setup_component_imports().unwrap();
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let lines = LineIndex::new(source);
    let product = |imports: &SetupComponentImports<'_, '_>, joined| {
        let diagnostics = if joined {
            DiagnosticService::imported_component_required_props(state, uri, imports).unwrap()
        } else {
            DiagnosticService::collect_component_required_prop_diagnostics(
                state,
                uri,
                source,
                &descriptor,
                &lines,
            )
        };
        // Editor diagnostics and the project's resolved navigation destinations
        // are both consumed after the temporary file owner may have been dropped.
        let targets: Vec<_> = imports
            .sites()
            .iter()
            .map(|site| {
                (
                    site.name_span(),
                    vize_l0::String::new(site.module()),
                    vize_l0::String::new(site.export()),
                )
            })
            .collect();
        (targets, diagnostics)
    };
    let mode = |cold| {
        let run = |joined| {
            if cold {
                let arena = Allocator::new();
                let observed = observe(&arena, black_box(source));
                let imports = observed.setup_component_imports().unwrap();
                product(&imports, joined)
            } else {
                product(black_box(&retained), joined)
            }
        };
        assert_eq!(run(false), run(true), "{label}");
        let old = alloc::measure(|| run(false)).unwrap();
        let new = alloc::measure(|| run(true)).unwrap();
        let time = |joined, n| {
            let at = Instant::now();
            for _ in 0..n {
                drop(black_box(run(joined)));
            }
            at.elapsed().as_nanos() / n
        };
        let sample = |n, reverse| {
            let mut pairs = Vec::with_capacity(31);
            for round in 0..35 {
                let first = (round % 2 == 0) != reverse;
                let a = time(first, n);
                let b = time(!first, n);
                if round >= 4 {
                    pairs.push(if first { (b, a) } else { (a, b) });
                }
            }
            pairs
        };
        let mut pairs = sample(repetitions, false);
        let slower = |p: &[(u128, u128)]| p.iter().filter(|(a, b)| b > a).count() >= 23;
        let mut pass =
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start;
        if slower(&pairs) {
            let confirm = sample(repetitions * 8, true);
            eprintln!("prop_confirm input={label} cold={cold} ns={confirm:?}");
            pass &= !slower(&confirm);
        }
        if repetitions > 1 || !pass {
            eprintln!("prop_pairs input={label} cold={cold} ns={pairs:?}");
        }
        pairs.sort_unstable_by_key(|p| p.0);
        let before = pairs[15].0;
        pairs.sort_unstable_by_key(|p| p.1);
        eprintln!(
            "prop_request input={label} cold={cold} ns={}/{} calls={}/{} peak={}/{} pass={pass}",
            before,
            pairs[15].1,
            old.calls,
            new.calls,
            old.peak_bytes_over_start,
            new.peak_bytes_over_start
        );
        pass
    };
    (mode(false), mode(true))
}

#[test]
#[ignore = "actual resident and newly assembled component diagnostics and import destinations"]
fn complete_component_requests() {
    alloc::mark_installed();
    let temp = tempfile::tempdir().unwrap();
    child(temp.path());
    let uri = Url::from_file_path(temp.path().join("Parent.vue")).unwrap();
    for count in [0, 1, 8, 32] {
        let mut source =
            String::from("<script setup>import Box from './Box.vue'</script><template>");
        for _ in 0..count {
            source.push_str("<Box first/><Box first user-name='猫'/>");
        }
        source.push_str("</template>");
        if count == 0 {
            source = String::from("<script setup>const x=1</script><template><div/></template>");
        }
        let state = ServerState::new();
        open(&state, &uri, &source);
        assert_eq!(
            profile(&state, &uri, &source, &count.to_string(), 2),
            (true, true)
        );
    }
}

#[test]
#[ignore = "per-original complete diagnostic and navigation responses"]
fn repository_component_requests() {
    alloc::mark_installed();
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mut failures = Vec::new();
    let mut count = 0;
    for name in manifest.split_terminator('\0') {
        let path = root.join(name);
        let bytes = fs::read(&path).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            continue;
        };
        let arena = Allocator::new();
        let observed = observe(&arena, source);
        if observed.setup_component_imports().is_err() {
            continue;
        }
        let path = if path.extension().is_some_and(|e| e == "txt") {
            path.with_extension("")
        } else {
            path
        };
        let uri = Url::from_file_path(path).unwrap();
        let state = ServerState::new();
        open(&state, &uri, source);
        let result = profile(&state, &uri, source, name, 1);
        if result != (true, true) {
            failures.push((name, result));
        }
        count += 1;
    }
    assert!(count > 0);
    assert!(failures.is_empty(), "{failures:?}");
}
