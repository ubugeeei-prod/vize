//! Merging only the declarations a component can look up, against merging
//! every declaration its walks reached.
//!
//! The context-level cases run on generated graphs whose declarations name
//! each other, so a component reaches names through the bodies it merged. Every
//! name the component's source and those bodies spell must resolve to the
//! same declaration either way, and the lazy maps hold nothing else.
//!
//! The compile-level cases compile whole components of barrel projects twice:
//! alone, which merges everything, and in a batch, which merges what the
//! component references. Code, diagnostics and dependencies must not differ.

use std::path::Path;

use vize_carton::{FxHashMap, FxHashSet, String};

use super::super::super::batch_epoch::{begin_type_resolution_batch, tests::lock_epoch_state};
use super::super::modules::ModuleGraph;
use super::super::walk::TypeDependency;
use super::super::{FileTypeSummary, ScriptCompileContext, extract_script_summary};
use super::differential::Rng;
use super::{reference, synthetic_epoch};
use crate::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

/// Enough modules behind each barrel that a component merges by name.
const MODELS: usize = 48;

const NAMES: [&str; 6] = ["A", "B", "C", "D", "E", "F"];

fn pick(rng: &mut Rng) -> &'static str {
    NAMES[rng.below(NAMES.len())]
}

fn target(rng: &mut Rng, modules: usize) -> std::string::String {
    let target = rng.below(modules);
    if rng.below(12) == 0 {
        format!("./missing{target}")
    } else {
        format!("./m{target}")
    }
}

/// A module whose declarations refer to other names, with every edge form
/// the summary distinguishes, `export * as` included.
fn write_module(rng: &mut Rng, dir: &Path, index: usize, modules: usize) {
    let mut source = std::string::String::new();
    for declaration in 0..rng.below(4) {
        let (name, other, field) = (pick(rng), pick(rng), pick(rng));
        source.push_str(&match rng.below(5) {
            0 => format!(
                "export interface {name} extends {other} {{ m{index}x{declaration}: {field} }}\n"
            ),
            1 => format!("export type {name} = {other} & {{ m{index}t{declaration}: string }}\n"),
            2 => format!("interface {name} {{ m{index}p{declaration}: {field}[] }}\n"),
            3 => format!("export type {name} = {{ m{index}u{declaration}: '{field}' | number }}\n"),
            _ => format!("export interface {name} {{ m{index}i{declaration}: string }}\n"),
        });
    }
    for _ in 0..rng.below(4) {
        let target = target(rng, modules);
        let (first, second) = (pick(rng), pick(rng));
        source.push_str(&match rng.below(8) {
            0 => format!("import type {{ {first} }} from '{target}'\n"),
            1 => format!("import type * as N{first} from '{target}'\n"),
            2 => format!("export type {{ {first} as {second} }} from '{target}'\n"),
            3 => format!("export {{ {first} }} from '{target}'\n"),
            4 => format!("export * as {first}s from '{target}'\n"),
            _ => format!("export * from '{target}'\n"),
        });
    }
    std::fs::write(dir.join(format!("m{index}.ts")), source).unwrap();
}

/// A component script: its imports, a local declaration that may shadow an
/// imported one, and a macro whose type names a few names.
fn component_script(rng: &mut Rng, modules: usize) -> std::string::String {
    let mut script = std::string::String::new();
    for _ in 0..1 + rng.below(3) {
        let target = target(rng, modules);
        let (first, second) = (pick(rng), pick(rng));
        script.push_str(&match rng.below(3) {
            0 => format!("import type {{ {first}, {second} }} from '{target}'\n"),
            1 => format!("import type * as T from '{target}'\n"),
            _ => format!("import type {{ {first} }} from '{target}'\n"),
        });
    }
    if rng.below(3) == 0 {
        let name = pick(rng);
        script.push_str(&format!("type {name} = {{ local: string }}\n"));
    }
    let (first, second) = (pick(rng), pick(rng));
    script.push_str(&format!(
        "const props = defineProps<{first} & {{ own: {second} }}>()\n"
    ));
    script
}

fn roots(script: &str) -> Vec<TypeDependency> {
    let mut summary = FileTypeSummary::default();
    extract_script_summary(script, &mut summary, false);
    summary.dependencies
}

/// The names a lookup can reach from `texts` through the bodies of `ctx`.
fn reachable(ctx: &ScriptCompileContext, texts: &[&str]) -> FxHashSet<String> {
    let words = |text: &str| -> Vec<String> {
        text.split(|char: char| !oxc_syntax::identifier::is_identifier_part(char))
            .filter(|word| !word.is_empty())
            .map(String::from)
            .collect()
    };
    let mut seen = FxHashSet::default();
    let mut wanted: Vec<String> = texts.iter().flat_map(|text| words(text)).collect();
    while let Some(name) = wanted.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let body = ctx.interfaces.get(&name).or(ctx.type_aliases.get(&name));
        if let Some(body) = body {
            wanted.extend(words(body));
        }
    }
    seen
}

fn restricted(
    map: &FxHashMap<String, String>,
    names: &FxHashSet<String>,
) -> FxHashMap<String, String> {
    map.iter()
        .filter(|(name, _)| names.contains(*name))
        .map(|(name, body)| (name.clone(), body.clone()))
        .collect()
}

#[test]
fn referenced_merge_matches_the_full_merge_on_generated_graphs() {
    for seed in 1..=200_u64 {
        let mut rng = Rng(seed.wrapping_mul(0xD1B5_4A32_D192_ED03));
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path().canonicalize().unwrap();
        let modules = 2 + rng.below(10);
        for index in 0..modules {
            write_module(&mut rng, &dir, index, modules);
        }
        let shared = ModuleGraph::new(synthetic_epoch());
        for component in 0..6 {
            let script = component_script(&mut rng, modules);
            let roots = roots(&script);
            let base_file = dir.join(format!("Component{component}.vue"));
            let label = format!("seed {seed}, component {component}\n{script}");

            let mut full = ScriptCompileContext::new("");
            full.collect_types_from(&script);
            let (full_imports, full_files) =
                full.walk_imported_modules(&roots, &base_file, None, Some(&shared));
            // The full merge itself is the per-component reference's.
            let mut alone = ScriptCompileContext::new("");
            alone.collect_types_from(&script);
            reference::collect(&mut alone, &roots, &base_file, None, synthetic_epoch());
            assert_eq!(full.interfaces, alone.interfaces, "{label}");
            assert_eq!(full.type_aliases, alone.type_aliases, "{label}");

            let mut lazy = ScriptCompileContext::new("");
            lazy.collect_types_from(&script);
            let walked =
                lazy.walk_imported_declarations(&roots, &[], &base_file, None, Some(&shared));
            let (lazy_imports, lazy_files) = (walked.imports, walked.files);
            lazy.merge_by_reference(&[], &walked.declarations, &[&script]);

            assert_eq!(lazy_imports, full_imports, "{label}");
            assert_eq!(lazy_files, full_files, "{label}");
            let names = reachable(&full, &[&script]);
            assert_eq!(
                lazy.interfaces,
                restricted(&full.interfaces, &names),
                "{label}"
            );
            assert_eq!(
                lazy.type_aliases,
                restricted(&full.type_aliases, &names),
                "{label}"
            );
        }
    }
}

/// An identifier spelled with an escape is not a run of identifier
/// characters, so a component whose source or walked bodies contain one
/// merges every declaration.
#[test]
fn an_escaped_identifier_merges_every_declaration() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    let mut types = std::string::String::from("export interface A { a: string }\n");
    for index in 0..MODELS {
        types.push_str(&format!("export interface B{index} {{ b: string }}\n"));
    }
    std::fs::write(dir.join("types.ts"), types).unwrap();
    let base_file = dir.join("Component.vue");
    let shared = ModuleGraph::new(synthetic_epoch());
    for (script, expected) in [
        ("import type * as T from './types'\ndefineProps<A>()\n", 1),
        (
            "import type * as T from './types'\ndefineProps<\\u0042>()\n",
            MODELS + 1,
        ),
    ] {
        let mut ctx = ScriptCompileContext::new("");
        let walked =
            ctx.walk_imported_declarations(&roots(script), &[], &base_file, None, Some(&shared));
        ctx.merge_referenced_declarations(&[], &walked.declarations, &[script]);
        assert_eq!(ctx.interfaces.len(), expected, "{script}");
    }
}

/// Compile `source` as `path` with nothing shared, then in a batch, and
/// return both results' observable output.
pub(super) fn compile_alone_and_in_batch(
    path: &Path,
    source: &str,
) -> (std::string::String, std::string::String) {
    let compile = || {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let mut options = SfcCompileOptions::default();
        options.script.id = Some(path.to_string_lossy().as_ref().into());
        match compile_sfc(&descriptor, options) {
            Ok(result) => format!(
                "code:\n{}\nerrors: {:?}\nwarnings: {:?}\ndependencies: {:?}",
                result.code, result.errors, result.warnings, result.dependencies
            ),
            Err(error) => format!("error: {error:?}"),
        }
    };
    let alone = compile();
    let batch = begin_type_resolution_batch();
    let in_batch = compile();
    // A second component of the batch reuses the walks the first one made.
    let again = compile();
    drop(batch);
    assert_eq!(in_batch, again, "{source}");
    (alone, in_batch)
}

pub(super) fn write(dir: &Path, files: &[(&str, &str)]) {
    for (name, text) in files {
        let path = dir.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
}

/// Barrel shapes, each with components that reference a name through it,
/// shadow it locally, reach it through a merged body or a `generic`
/// constraint, or cannot resolve it.
#[test]
fn compiles_through_barrels_match_with_and_without_a_batch() {
    let _lock = lock_epoch_state();
    let models = |count: usize| -> Vec<(std::string::String, std::string::String)> {
        (0..count)
            .map(|index| {
                (
                    format!("models/Model{index}.ts"),
                    format!(
                        "export interface Model{index} {{ field{index}: string; next?: Model{} }}\n",
                        (index + 1) % count
                    ),
                )
            })
            .collect()
    };
    let star: std::string::String = (0..MODELS)
        .map(|index| format!("export * from './Model{index}'\n"))
        .collect();
    let shapes: Vec<(&str, std::string::String)> = vec![
        ("star", star.clone()),
        (
            "named",
            "export { Model0, Model3 as Renamed } from './Model0'\nexport type { Model3 } from './Model3'\n"
                .into(),
        ),
        (
            "nested star",
            format!("export * from './inner'\nexport * from './Model{}'\n", MODELS - 1),
        ),
        (
            "cycle",
            "export * from './index'\nexport * from './inner'\n".into(),
        ),
        (
            "namespace re-export",
            "export * as Models from './inner'\nexport * from './Model2'\n".into(),
        ),
        (
            "shadowing",
            format!("{star}export interface Model0 {{ shadowed: number }}\nexport type Model1 = {{ alias: boolean }}\n"),
        ),
    ];
    let components = [
        "<script setup lang=\"ts\">\nimport type { Model0 } from '../models'\nconst props = defineProps<Model0>()\n</script>\n",
        "<script setup lang=\"ts\">\nimport type { Model3, Renamed } from '../models'\ndefineProps<Model3 & { other?: Renamed }>()\n</script>\n",
        "<script setup lang=\"ts\">\nimport type { Model1 } from '../models'\ntype Model1 = { local: string }\ndefineProps<Model1>()\n</script>\n",
        "<script lang=\"ts\">\nexport interface Model2 { fromNormal: string }\n</script>\n<script setup lang=\"ts\">\nimport type { Model2 } from '../models'\ndefineProps<Model2>()\n</script>\n",
        "<script setup lang=\"ts\" generic=\"T extends Model4\">\nimport type * as M from '../models'\ndefineProps<{ value: T; list: Model5[] }>()\n</script>\n",
        "<script setup lang=\"ts\">\nimport type { Missing } from '../models'\ndefineProps<Missing>()\n</script>\n",
        "<script setup lang=\"ts\">\nimport type { Model6 } from '../models'\nconst emit = defineEmits<{ change: [value: Model7] }>()\nconst model = defineModel<Model6>()\ndefineProps<{ label: string }>()\n</script>\n",
        "<script setup lang=\"ts\">\nimport type { Models } from '../models'\nwithDefaults(defineProps<Partial<Model8>>(), { field8: 'x' })\n</script>\n",
    ];
    for (shape, barrel) in &shapes {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path().canonicalize().unwrap();
        let mut files = models(MODELS);
        files.push(("models/index.ts".into(), barrel.clone()));
        files.push((
            "models/inner.ts".into(),
            (0..MODELS - 1)
                .map(|index| format!("export * from './Model{index}'\n"))
                .collect(),
        ));
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(name, text)| (name.as_str(), text.as_str()))
            .collect();
        write(&dir, &files);
        write(
            &dir,
            &[(
                "tsconfig.json",
                r#"{ "compilerOptions": { "moduleResolution": "bundler", "strict": true } }"#,
            )],
        );
        for (index, component) in components.iter().enumerate() {
            let path = dir.join(format!("components/Item{index}.vue"));
            write(&dir, &[(&format!("components/Item{index}.vue"), component)]);
            let (alone, in_batch) = compile_alone_and_in_batch(&path, component);
            assert_eq!(in_batch, alone, "{shape}: component {index}\n{component}");
        }
    }
}
