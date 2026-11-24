#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "tests assert by panicking"
)]
use std::path::{Path, PathBuf};
use std::sync::{Barrier, Mutex, MutexGuard};
use std::time::Duration;

use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, begin_type_resolution_batch, compile_sfc,
    parse_sfc,
};
use vize_l0::ToCompactString;

/// A batch is process-wide: a test that opens one, or that expects a compile
/// outside every batch, must not overlap another.
static BATCHES: Mutex<()> = Mutex::new(());

fn exclusive_batches() -> MutexGuard<'static, ()> {
    BATCHES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn temp_project_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let name = vize_l0::cstr!(
        "vize-sfc-imported-type-cache-{}-{nonce}",
        std::process::id()
    );
    std::env::temp_dir().join(name.as_str())
}

fn compile_imported_props(source: &str, component: &Path) -> SfcCompileResult {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC should parse");
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(component.to_string_lossy().as_ref().to_compact_string());
    compile_sfc(&descriptor, options).expect("SFC should compile")
}

fn rewrite_with_newer_mtime(path: &Path, content: &str) {
    let previous_mtime = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .expect("type file should have an mtime");
    std::fs::write(path, content).unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(previous_mtime + Duration::from_secs(1)))
        .unwrap();
}

#[test]
fn single_compile_revalidates_imported_types_after_batch() {
    let _exclusive = exclusive_batches();
    let project = temp_project_dir();
    std::fs::create_dir_all(&project).unwrap();
    let component = project.join("App.vue");
    let types = project.join("types.ts");
    let source = r#"<script setup lang="ts">
import type { Props } from './types'
defineProps<Props>()
</script>"#;

    std::fs::write(&types, "export interface Props { first: string }\n").unwrap();
    let batch = {
        let _batch = begin_type_resolution_batch();
        compile_imported_props(source, &component)
    };
    assert!(batch.code.contains("first: {"), "{}", batch.code);

    // Same-length replacement: only metadata revalidation can detect it.
    rewrite_with_newer_mtime(&types, "export interface Props { other: string }\n");
    let hmr = compile_imported_props(source, &component);
    assert!(hmr.code.contains("other: {"), "{}", hmr.code);
    assert!(!hmr.code.contains("first: {"), "{}", hmr.code);

    let _ = std::fs::remove_dir_all(project);
}

/// A star barrel of twelve models behind `src/types.ts`, reached through a
/// relative import, a `paths` alias and the barrel itself, plus two
/// directories whose `./types` are different files.
fn write_shared_types_project(project: &Path) -> Vec<(PathBuf, String)> {
    let src = project.join("src");
    for directory in ["components", "models", "left", "right"] {
        std::fs::create_dir_all(src.join(directory)).unwrap();
    }
    std::fs::write(
        project.join("tsconfig.json"),
        r#"{ "compilerOptions": { "paths": { "~/*": ["./src/*"] } }, "include": ["src"] }"#,
    )
    .unwrap();
    let mut barrel = String::new();
    for index in 0..12 {
        let model =
            vize_l0::cstr!("export interface Model{index} {{\n  field{index}: string;\n}}\n");
        std::fs::write(
            src.join(vize_l0::cstr!("models/Model{index}.ts").as_str()),
            model.as_str(),
        )
        .unwrap();
        barrel.push_str(vize_l0::cstr!("export * from \"./Model{index}\";\n").as_str());
    }
    std::fs::write(src.join("models/index.ts"), barrel).unwrap();
    std::fs::write(
        src.join("types.ts"),
        "import type { Model0 } from \"./models\";\n\nexport interface ItemProps extends Model0 {\n  id: string;\n  label: string;\n}\n",
    )
    .unwrap();
    std::fs::write(
        src.join("left/types.ts"),
        "export interface Props { left: string }\n",
    )
    .unwrap();
    std::fs::write(
        src.join("right/types.ts"),
        "export interface Props { right: number }\n",
    )
    .unwrap();

    let component = |path: &str, script: &str| {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">\n{script}\n</script>\n\n<template><div /></template>\n"
        );
        let path = src.join(path);
        std::fs::write(&path, source.as_str()).unwrap();
        (path, String::from(source.as_str()))
    };
    let mut components = Vec::new();
    for index in 0..6 {
        components.push(component(
            vize_l0::cstr!("components/Relative{index}.vue").as_str(),
            "import type { ItemProps } from \"../types\";\nconst props = defineProps<ItemProps>();",
        ));
        components.push(component(
            vize_l0::cstr!("components/Alias{index}.vue").as_str(),
            "import type { ItemProps } from \"~/types\";\nconst props = defineProps<ItemProps>();",
        ));
        components.push(component(
            vize_l0::cstr!("components/Barrel{index}.vue").as_str(),
            vize_l0::cstr!(
                "import type {{ Model{index} }} from \"../models\";\nconst props = defineProps<Model{index}>();"
            )
            .as_str(),
        ));
    }
    for directory in ["left", "right"] {
        components.push(component(
            vize_l0::cstr!("{directory}/Sibling.vue").as_str(),
            "import type { Props } from \"./types\";\nconst props = defineProps<Props>();",
        ));
    }
    components
}

/// What a host reads from one compile.
type Compiled = (String, Vec<String>, usize, usize);

fn compiled(source: &str, component: &Path) -> Compiled {
    let result = compile_imported_props(source, component);
    let dependencies = result
        .dependencies
        .iter()
        .map(|dependency| String::from(dependency.as_str()))
        .collect();
    (
        String::from(result.code.as_str()),
        dependencies,
        result.errors.len(),
        result.warnings.len(),
    )
}

/// Every component of the batch, compiled by `threads` workers that start
/// together inside one batch.
fn compile_batch(components: &[(PathBuf, String)], threads: usize) -> Vec<Compiled> {
    let _batch = begin_type_resolution_batch();
    let start = Barrier::new(threads);
    let mut results: Vec<(usize, Compiled)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    components
                        .iter()
                        .enumerate()
                        .skip(worker)
                        .step_by(threads)
                        .map(|(index, (path, source))| (index, compiled(source, path)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect()
    });
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

#[test]
fn batch_workers_share_imported_types_without_changing_any_component() {
    let _exclusive = exclusive_batches();
    let project = tempfile::tempdir().unwrap();
    let components = write_shared_types_project(project.path());

    // Outside a batch every component walks its imports on its own.
    let alone: Vec<Compiled> = components
        .iter()
        .map(|(path, source)| compiled(source, path))
        .collect();
    // The barrel, its twelve models and `types.ts`, and the discovery inputs.
    let discovery = vize_atelier_sfc::script::SfcTypeProject::discovery_inputs(&components[0].0)
        .unwrap()
        .len();
    assert_eq!(alone[0].1.len(), 14 + discovery);
    assert_eq!(alone[0].2, 0);
    // The alias reaches the same modules; the barrel alone skips `types.ts`.
    assert_eq!(alone[1].1, alone[0].1);
    assert_eq!(alone[2].1.len(), 13 + discovery);
    // `./types` is another file in each sibling directory.
    assert_ne!(alone[18].0, alone[19].0);
    assert_ne!(alone[18].1, alone[19].1);

    for threads in [1, 4, 8] {
        for _ in 0..3 {
            assert_eq!(compile_batch(&components, threads), alone);
        }
    }
}

#[test]
fn each_batch_reads_a_shared_type_as_it_is_when_the_batch_begins() {
    let _exclusive = exclusive_batches();
    let project = tempfile::tempdir().unwrap();
    let components = write_shared_types_project(project.path());
    let model = project.path().join("src/models/Model0.ts");

    let before = compile_batch(&components, 4);
    // Same length: only the bytes tell the two revisions apart.
    std::fs::write(&model, "export interface Model0 {\n  other0: string;\n}\n").unwrap();
    let after = compile_batch(&components, 4);
    let alone: Vec<Compiled> = components
        .iter()
        .map(|(path, source)| compiled(source, path))
        .collect();

    assert_eq!(after, alone);
    assert_ne!(before[0].0, after[0].0);
    assert_eq!(before[0].1, after[0].1);
}
