use super::*;

/// Measure registration and the first read with a prepared analysis for each side.
/// Original parser inputs and the target registry belong to the caller on both paths.
#[test]
#[ignore = "registration plus first render request, including source revision capture"]
fn registration_and_first_read() {
    alloc::mark_installed();
    for (size, padding) in [(0, 0), (0, 65_536), (1, 0), (8, 0), (32, 0)] {
        let mut source = String::from("<script setup>const unused=1;");
        let mut registries = [ModuleRegistry::new(), ModuleRegistry::new()];
        for n in 0..size {
            source.push_str(&format!("import Card{n} from './Card{n}.vue';"));
            for registry in &mut registries {
                register(
                    registry,
                    Path::new(&format!("/project/Card{n}.vue")),
                    "<template/>",
                );
            }
        }
        source.push_str("</script><template><div>");
        source.push_str(&"x".repeat(padding));
        for n in 0..size {
            source.push_str(&format!("<Card{n}/><Card{n}/>"));
        }
        source.push_str("</div></template>");
        let path = Path::new("/project/App.vue");
        let arena = Allocator::new();
        let owner = observe(&arena, &source);
        let imports = owner.setup_component_imports().unwrap();
        let run = |native, registry: &mut ModuleRegistry, facts| {
            if native {
                let (_, input) = registry
                    .register_setup_caller(path, facts, &imports)
                    .unwrap();
                registry.setup_render_products(path, &input).unwrap()
            } else {
                registry.register(path, &source, facts);
                original(registry, path)
            }
        };
        let [old_registry, new_registry] = &mut registries;
        assert_eq!(
            run(false, old_registry, analysis(path, &source)),
            run(true, new_registry, analysis(path, &source))
        );
        let sample_alloc = |native, registry: &mut ModuleRegistry| {
            registry.remove(path);
            let facts = analysis(path, &source);
            alloc::measure(|| run(native, registry, facts)).unwrap()
        };
        let old = sample_alloc(false, old_registry);
        let new = sample_alloc(true, new_registry);
        assert!(new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start);
        let mut time = |native, iterations: u128| {
            let registry = if native {
                &mut *new_registry
            } else {
                &mut *old_registry
            };
            let mut elapsed = 0;
            for _ in 0..iterations {
                registry.remove(path);
                let facts = analysis(path, &source);
                let start = Instant::now();
                drop(black_box(run(native, registry, facts)));
                elapsed += start.elapsed().as_nanos();
            }
            elapsed / iterations
        };
        let pairs = sample(&mut time, 4, false);
        eprintln!(
            "setup_registration_time size={size} padding={padding} ns={pairs:?} calls={}/{} peak={}/{}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        if slower(&pairs) {
            let confirmation = sample(&mut time, 32, true);
            eprintln!(
                "setup_registration_confirmation size={size} padding={padding} ns={confirmation:?}"
            );
            assert!(!slower(&confirmation), "size={size} padding={padding}");
        }
    }
}
