//! Production CF traversal and provider-key storage measurements. The same
//! fixture and allocator are copied to the base checkout by the A/B workflow.

use super::{FixtureFile, build_analyzer};
use criterion::{BatchSize, Criterion};
use std::{hint::black_box, path::Path};
use vize_carton::{CompactString, cstr};
use vize_croquis_cf::{CrossFileOptions, providers::ProjectSources};

fn tree_project(long: bool) -> Vec<FixtureFile> {
    let keys = (0..16)
        .map(|index| {
            if long {
                format!("shared_application_state_with_authored_unicode_状態_{index}")
            } else {
                format!("s{index}")
            }
        })
        .collect::<Vec<_>>();
    let mut files = vec![FixtureFile {
        path: "App.vue".into(),
        source: format!(
            "import {{ provide }} from 'vue'\n{}",
            keys.iter()
                .map(|key| format!("provide('{key}', {{ value: 1 }})\n"))
                .collect::<String>()
        ),
        used_components: (0..40)
            .map(|index| CompactString::new(format!("Parent{index}")))
            .collect(),
    }];
    for parent in 0..40 {
        files.push(FixtureFile {
            path: format!("Parent{parent}.vue"),
            source: "// pass-through component".into(),
            used_components: (0..5)
                .map(|leaf| CompactString::new(format!("Leaf{parent}_{leaf}")))
                .collect(),
        });
        for leaf in 0..5 {
            files.push(FixtureFile {
                path: format!("Leaf{parent}_{leaf}.vue"),
                source: format!(
                    "import {{ inject }} from 'vue'\n{}",
                    keys.iter()
                        .enumerate()
                        .map(|(index, key)| format!("const value{index} = inject('{key}')\n"))
                        .collect::<String>()
                ),
                used_components: Vec::new(),
            });
        }
    }
    files
}

fn module_paths(prefix: &str) -> Vec<CompactString> {
    (0..1_000)
        .map(|index| cstr!("{prefix}module{index}.ts"))
        .collect()
}

fn add_modules(paths: &[CompactString]) -> ProjectSources {
    let mut project = ProjectSources::new();
    for path in paths {
        black_box(project.add(path, "export const value = 1"));
    }
    project
}

fn write_report(name: &str, value: &serde_json::Value) {
    let Ok(directory) = std::env::var("STRING_STORAGE_REPORT_DIR") else {
        return;
    };
    let path = Path::new(&directory).join(cstr!("{name}.json").as_str());
    let result =
        std::fs::create_dir_all(directory).and_then(|_| std::fs::write(path, value.to_string()));
    if let Err(error) = result {
        eprintln!("cannot write string-storage report: {error}");
        std::process::exit(1);
    }
}

pub(super) fn bench(c: &mut Criterion) {
    level_harness::alloc::mark_installed();
    for (name, long) in [("short_keys", false), ("long_unicode_keys", true)] {
        let files = tree_project(long);
        let options = CrossFileOptions::default().with_provide_inject(true);
        let mut analyzer = build_analyzer(&files, options.clone());
        let (result, allocation) = level_harness::alloc::measure_returning(|| analyzer.analyze());
        if result.provide_inject_matches.len() != 3_200 {
            eprintln!("tree fixture did not resolve all 3,200 authored inject calls");
            std::process::exit(1);
        }
        write_report(
            &format!("tree_{name}"),
            &serde_json::json!({
                "allocationCalls": allocation.map(|metrics| metrics.calls),
                "peakBytesOverStart": allocation.map(|metrics| metrics.peak_bytes_over_start),
                "semantic": {
                    "tree": result.provide_inject_tree,
                    "matches": result.provide_inject_matches,
                },
            }),
        );
        c.bench_function(
            &format!("string_storage/tree_{name}_241_files_16_keys"),
            |b| {
                b.iter_batched(
                    || build_analyzer(&files, options.clone()),
                    |mut analyzer| black_box(analyzer.analyze()),
                    BatchSize::SmallInput,
                );
            },
        );
    }
    for (name, prefix) in [
        ("short", ""),
        ("middle_inline_boundary", "src/"),
        ("long_unicode", "src/components/application/状態/"),
    ] {
        let paths = module_paths(prefix);
        let (project, allocation) = level_harness::alloc::measure_returning(|| add_modules(&paths));
        write_report(
            &format!("modules_{name}"),
            &serde_json::json!({
                "allocationCalls": allocation.map(|metrics| metrics.calls),
                "peakBytesOverStart": allocation.map(|metrics| metrics.peak_bytes_over_start),
                "modules": project.len(),
                "pathBytes": paths.iter().map(|path| path.len()).collect::<Vec<_>>(),
            }),
        );
        c.bench_function(
            &format!("string_storage/module_add_{name}_1000_files"),
            |b| {
                b.iter(|| black_box(add_modules(&paths)));
            },
        );
        let mut resolved = ProjectSources::new();
        let root = resolved.add(&format!("{prefix}entry.ts"), "");
        for path in &paths {
            black_box(resolved.add(path, ""));
        }
        c.bench_function(
            &format!("string_storage/module_resolve_{name}_1000_files"),
            |b| {
                b.iter(|| {
                    if let Some(root) = root {
                        for index in 0..1_000 {
                            black_box(resolved.resolve(root, cstr!("./module{index}").as_str()));
                        }
                    }
                });
            },
        );
    }
}
