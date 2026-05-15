//! Benchmarks for strict cross-file analysis hot paths.
//!
//! Run with: cargo bench -p vize_croquis --bench cross_file

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::path::Path;
use vize_carton::{cstr, CompactString};
use vize_croquis::cross_file::{CrossFileAnalyzer, CrossFileOptions};
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

#[derive(Clone)]
struct FixtureFile {
    path: String,
    source: String,
    used_components: Vec<CompactString>,
}

fn script_analysis(source: &str, used_components: &[CompactString]) -> Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(source);
    for component in used_components {
        analyzer
            .croquis_mut()
            .used_components
            .insert(component.clone());
    }
    analyzer.finish()
}

fn build_analyzer(files: &[FixtureFile], options: CrossFileOptions) -> CrossFileAnalyzer {
    let mut analyzer = CrossFileAnalyzer::new(options);
    for file in files {
        analyzer.add_file_with_analysis(
            Path::new(file.path.as_str()),
            file.source.as_str(),
            script_analysis(file.source.as_str(), &file.used_components),
        );
    }
    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();
    analyzer
}

fn fixture_project(parent_count: usize, leaves_per_parent: usize) -> Vec<FixtureFile> {
    let mut files = Vec::new();
    let app_children = (0..parent_count)
        .map(|index| CompactString::new(format!("Parent{index}")))
        .collect::<Vec<_>>();
    files.push(FixtureFile {
        path: "App.vue".to_string(),
        source: "// app shell".to_string(),
        used_components: app_children,
    });

    for parent in 0..parent_count {
        let mut used_components = Vec::with_capacity(leaves_per_parent);
        for leaf in 0..leaves_per_parent {
            used_components.push(CompactString::new(format!("Leaf{parent}_{leaf}")));
        }

        files.push(FixtureFile {
            path: format!("Parent{parent}.vue"),
            source: format!(
                "import {{ provide, reactive }} from 'vue'\n\
                 const state = reactive({{ count: {parent}, user: {{ id: {parent} }} }})\n\
                 provide('state', state)"
            ),
            used_components,
        });

        for leaf in 0..leaves_per_parent {
            files.push(FixtureFile {
                path: format!("Leaf{parent}_{leaf}.vue"),
                source: format!(
                    "import {{ inject, ref, watch }} from 'vue'\n\
                     const state = inject('state')!\n\
                     const query = ref('')\n\
                     watch(query, async () => {{\n\
                       await load({leaf})\n\
                       state.count = {leaf}\n\
                     }})"
                ),
                used_components: Vec::new(),
            });
        }
    }

    files
}

fn bench_provide_inject_tree(c: &mut Criterion) {
    let files = fixture_project(40, 5);
    let options = CrossFileOptions::default().with_provide_inject(true);

    c.bench_function("cross_file/provide_inject_tree_241_files", |b| {
        b.iter_batched(
            || build_analyzer(&files, options.clone()),
            |mut analyzer| black_box(analyzer.analyze()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_strict_reactivity_and_race(c: &mut Criterion) {
    let files = fixture_project(40, 5);
    let options = CrossFileOptions::default()
        .with_provide_inject(true)
        .with_reactivity_tracking(true)
        .with_race_conditions(true)
        .with_unique_ids(true);

    c.bench_function("cross_file/strict_reactivity_race_241_files", |b| {
        b.iter_batched(
            || build_analyzer(&files, options.clone()),
            |mut analyzer| black_box(analyzer.analyze()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_local_race_without_provider_tree(c: &mut Criterion) {
    let files = (0..240)
        .map(|index| FixtureFile {
            path: format!("LocalRace{index}.vue"),
            source: cstr!(
                "import {{ ref, watch }} from 'vue'\n\
                 const source = ref('')\n\
                 const result = ref(null)\n\
                 watch(source, async () => {{\n\
                   result.value = await load({index})\n\
                 }})"
            )
            .to_string(),
            used_components: Vec::new(),
        })
        .collect::<Vec<_>>();
    let options = CrossFileOptions::default().with_race_conditions(true);

    c.bench_function("cross_file/local_race_no_provider_tree_240_files", |b| {
        b.iter_batched(
            || build_analyzer(&files, options.clone()),
            |mut analyzer| black_box(analyzer.analyze()),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_provide_inject_tree,
    bench_strict_reactivity_and_race,
    bench_local_race_without_provider_tree
);
criterion_main!(benches);
