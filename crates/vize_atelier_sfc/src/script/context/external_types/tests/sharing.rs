//! How the work of imported-type resolution grows with the components of a
//! batch and with the modules behind the type they share.
//!
//! The project is the reporter's reproduction (#13557, #13558): every
//! component types its props with `ItemProps`, which extends `Model0`, taken
//! from a star barrel of `modules` one-interface modules. The reference walk
//! is what each component ran before; the shared walk is one module graph per
//! batch. Both are counted in the same steps, and both must produce the same
//! declarations and dependencies for every component.

use std::path::{Path, PathBuf};
use std::sync::Barrier;

use vize_carton::{FxHashMap, String};

use super::super::modules::ModuleGraph;
use super::super::resolution::{canonical_base_file, path_key, project};
use super::super::walk::TypeDependency;
use super::super::work::{self, Work};
use super::super::{
    FileTypeSummary, ScriptCompileContext, build_file_summary, extract_script_summary,
};
use super::{reference, synthetic_epoch};

const SCRIPT: &str =
    "\nimport type { ItemProps } from \"../types\";\n\nconst props = defineProps<ItemProps>();\n";

struct Reproduction {
    components: Vec<(PathBuf, Vec<TypeDependency>)>,
    /// The dependency names every component reports, sorted.
    files: Vec<String>,
    interfaces: FxHashMap<String, String>,
}

fn write_reproduction(root: &Path, components: usize, modules: usize) -> Reproduction {
    let src = root.join("src");
    std::fs::create_dir_all(src.join("components")).unwrap();
    std::fs::create_dir_all(src.join("models")).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{ "compilerOptions": { "moduleResolution": "bundler", "strict": true }, "include": ["src"] }"#,
    )
    .unwrap();
    let mut barrel = std::string::String::new();
    let mut sources = vec![src.join("models/index.ts"), src.join("types.ts")];
    for index in 0..modules {
        let model = src.join(format!("models/Model{index}.ts"));
        std::fs::write(
            &model,
            format!("export interface Model{index} {{\n  field{index}: string;\n}}\n"),
        )
        .unwrap();
        barrel.push_str(&format!("export * from \"./Model{index}\";\n"));
        sources.push(model);
    }
    std::fs::write(src.join("models/index.ts"), barrel).unwrap();
    std::fs::write(
        src.join("types.ts"),
        "import type { Model0 } from \"./models\";\n\nexport interface ItemProps extends Model0 {\n  id: string;\n  label: string;\n}\n",
    )
    .unwrap();
    // Every walked file declares one interface under a name of its own.
    let mut files = Vec::new();
    let mut interfaces = FxHashMap::default();
    for source in &sources {
        files.push(path_key(&source.canonicalize().unwrap()));
        interfaces.extend(build_file_summary(source).unwrap().interfaces);
    }
    files.sort();
    assert_eq!(interfaces.len(), modules + 1);

    let mut root_summary = FileTypeSummary::default();
    extract_script_summary(SCRIPT, &mut root_summary, false);
    let components = (0..components)
        .map(|index| {
            let path = src.join(format!("components/Item{index}.vue"));
            std::fs::write(
                &path,
                format!("<script setup lang=\"ts\">{SCRIPT}</script>\n\n<template><div>{{{{ props.label }}}} {index}</div></template>\n"),
            )
            .unwrap();
            (
                canonical_base_file(path.to_str().unwrap()),
                root_summary.dependencies.clone(),
            )
        })
        .collect();
    Reproduction {
        components,
        files,
        interfaces,
    }
}

#[derive(Clone, Copy)]
enum Walker<'a> {
    /// The per-component walk, in the batch generation given.
    Reference(u64),
    /// One graph for the batch.
    Shared(&'a ModuleGraph),
}

/// Resolve every component as a batch's pool would: `threads` workers start
/// together and each takes every `threads`-th component. Returns the work of
/// all workers and the largest number of files one of them read.
fn run_batch(reproduction: &Reproduction, threads: usize, walker: Walker<'_>) -> (Work, usize) {
    let start = Barrier::new(threads);
    let mut total = Work::default();
    let mut busiest = 0;
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                let start = &start;
                scope.spawn(move || {
                    work::take();
                    start.wait();
                    let components = reproduction.components.iter().skip(worker);
                    for (base_file, roots) in components.step_by(threads) {
                        let project = project::for_file(base_file).unwrap();
                        let project = project.as_deref();
                        let mut ctx = ScriptCompileContext::new(SCRIPT);
                        let (_, files) = match walker {
                            Walker::Reference(epoch) => {
                                reference::collect(&mut ctx, roots, base_file, project, epoch)
                            }
                            Walker::Shared(graph) => {
                                ctx.walk_imported_modules(roots, base_file, project, Some(graph))
                            }
                        };
                        assert_eq!(files, reproduction.files);
                        assert_eq!(ctx.interfaces, reproduction.interfaces);
                        assert_eq!(ctx.type_aliases, FxHashMap::default());
                    }
                    work::take()
                })
            })
            .collect();
        for worker in workers {
            let work = worker.join().unwrap();
            busiest = busiest.max(work.source_reads);
            total.add(work);
        }
    });
    (total, busiest)
}

fn row(walk: &str, batch: &str, shape: (usize, usize, usize), (work, busiest): (Work, usize)) {
    let (components, modules, threads) = shape;
    println!(
        "| {walk} | {batch} | {components} | {modules} | {threads} | {} | {busiest} | {} | {} | {} | {} | {} | {} | {} | {} |",
        work.source_reads,
        work.summary_parses,
        work.probes,
        work.resolutions,
        work.edges,
        work.modules,
        work.shared_locks,
        work.declarations,
        work.dependencies,
    );
}

fn header() {
    println!(
        "| walk | batch | N | M | threads | files read | most by one worker | parses | probes | resolutions | edges | modules walked | locks | declarations merged | dependency names |"
    );
    println!(
        "| :-- | :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: |"
    );
}

/// With N components and a barrel of M modules a walk reaches F = M + 2
/// files. The reference walk resolves and walks F modules for every
/// component. The shared walk does it once per batch. What stays per
/// component is its result: the declarations merged into its context and the
/// dependency names it reports.
#[test]
fn shared_walk_work_is_modules_plus_components() {
    header();
    for modules in [1, 25, 100, 400] {
        for components in [1, 8, 64] {
            let files = modules + 2;
            let shape = (components, modules, 1);

            let dir = tempfile::tempdir().unwrap();
            let reproduction = write_reproduction(dir.path(), components, modules);
            for (batch, parses) in [("first", files), ("later", 0)] {
                let work = run_batch(&reproduction, 1, Walker::Reference(synthetic_epoch()));
                row("reference", batch, shape, work);
                assert_eq!(
                    work.0,
                    Work {
                        source_reads: files,
                        summary_parses: parses,
                        probes: files,
                        resolutions: components * files,
                        edges: components * files,
                        modules: components * files,
                        declarations: components * (modules + 1),
                        dependencies: components * files,
                        // A summary lookup and a resolution per visit; a swap
                        // per probe and per parse.
                        shared_locks: 2 * components * files + files + parses,
                    }
                );
            }

            let dir = tempfile::tempdir().unwrap();
            let reproduction = write_reproduction(dir.path(), components, modules);
            for (batch, parses) in [("first", files), ("later", 0)] {
                let graph = ModuleGraph::new(synthetic_epoch());
                let work = run_batch(&reproduction, 1, Walker::Shared(&graph));
                row("shared", batch, shape, work);
                assert_eq!(work.0, shared_work(components, modules, parses));
            }
        }
    }
}

/// The work of one batch over a shared graph when one worker walks.
fn shared_work(components: usize, modules: usize, parses: usize) -> Work {
    let files = modules + 2;
    Work {
        source_reads: files,
        summary_parses: parses,
        probes: files,
        resolutions: files,
        // The barrel's stars and the import in `types.ts`, walked once, and
        // each component's own import.
        edges: modules + 1 + components,
        modules: files,
        declarations: components * (modules + 1),
        dependencies: components * files,
        // Per file: its id, its summary lookup, its resolution and the swap
        // of its probe; a swap per parse. Per component: the module its
        // import resolves to and that module's closure.
        shared_locks: 4 * files + parses + 2 * components,
    }
}

/// Workers that reach a shared type together all walk it, and the graph
/// gives each read and each resolution to one of them: the files read, the
/// probes and the locks of a batch are the same at every thread count. Only
/// the walk over modules already in memory is repeated, by the workers that
/// started before a result existed.
///
/// The reference walk reads a file once per worker that reaches it before
/// another worker has validated it, so its counts depend on scheduling and
/// are only printed. So is the largest number of files one worker read: how
/// the shared reads split between workers is up to the scheduler.
#[test]
fn shared_walk_reads_do_not_depend_on_threads() {
    let (components, modules) = (64, 100);
    header();
    for threads in [1, 4, 16] {
        let shape = (components, modules, threads);

        let dir = tempfile::tempdir().unwrap();
        let reproduction = write_reproduction(dir.path(), components, modules);
        for batch in ["first", "later", "later"] {
            let walker = Walker::Reference(synthetic_epoch());
            row(
                "reference",
                batch,
                shape,
                run_batch(&reproduction, threads, walker),
            );
        }

        let dir = tempfile::tempdir().unwrap();
        let reproduction = write_reproduction(dir.path(), components, modules);
        for (batch, parses) in [("first", modules + 2), ("later", 0), ("later", 0)] {
            let graph = ModuleGraph::new(synthetic_epoch());
            let (work, busiest) = run_batch(&reproduction, threads, Walker::Shared(&graph));
            row("shared", batch, shape, (work, busiest));
            let alone = shared_work(components, modules, parses);
            let unwalked = |work: Work| Work {
                modules: 0,
                edges: 0,
                ..work
            };
            assert_eq!(unwalked(work), unwalked(alone));
            let walked_edges = work.edges - components;
            assert!(work.modules >= alone.modules, "{work:?}");
            assert!(work.modules <= threads * alone.modules, "{work:?}");
            assert!(walked_edges > modules, "{work:?}");
            assert!(walked_edges <= threads * (modules + 1), "{work:?}");
        }
    }
}

/// A graph with no room to keep walks keeps none: every component walks, but
/// over modules the graph read and resolved once. With room for one walk the
/// work is the usual one.
#[test]
fn a_graph_without_room_repeats_walks_but_not_reads() {
    let (components, modules) = (8, 25);
    let files = modules + 2;
    let dir = tempfile::tempdir().unwrap();
    let reproduction = write_reproduction(dir.path(), components, modules);

    let crowded = ModuleGraph::keeping(synthetic_epoch(), 0);
    assert_eq!(
        run_batch(&reproduction, 1, Walker::Shared(&crowded)).0,
        Work {
            modules: components * files,
            edges: components * (modules + 1) + components,
            ..shared_work(components, modules, files)
        }
    );

    // A walk holds each module it reached twice: in walk order and by name.
    let roomy = ModuleGraph::keeping(synthetic_epoch(), 2 * files);
    assert_eq!(
        run_batch(&reproduction, 1, Walker::Shared(&roomy)).0,
        shared_work(components, modules, 0)
    );
}

/// Module ids stay valid while the table grows past its first chunks.
#[test]
fn modules_keep_their_ids_as_a_graph_grows() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    let importer = dir.join("Component.vue");
    let graph = ModuleGraph::new(synthetic_epoch());
    let count = 200;
    for index in 0..count {
        std::fs::write(dir.join(format!("m{index}.ts")), "").unwrap();
    }
    let resolve = |index: usize| {
        let module = graph
            .resolve(&importer, &format!("./m{index}"), None)
            .unwrap();
        (module, graph.module(module).unwrap().key.clone())
    };
    let first: Vec<_> = (0..count).map(resolve).collect();
    let again: Vec<_> = (0..count).map(resolve).collect();
    assert_eq!(first, again);
    let keys: Vec<String> = first.into_iter().map(|(_, key)| key).collect();
    let expected: Vec<String> = (0..count)
        .map(|index| path_key(&dir.join(format!("m{index}.ts"))))
        .collect();
    assert_eq!(keys, expected);
}

/// One graph per batch generation and resolution context. A worker that is
/// behind the current generation, or that resolves through other options of
/// the same project, walks on its own instead of sharing files it may not
/// reuse; nothing outlives the batch.
#[test]
fn a_shared_graph_serves_one_generation_of_one_context() {
    use super::super::super::batch_epoch::{NO_EPOCH, tests::lock_epoch_state};
    use super::super::modules::{retire, shared};
    use super::super::resolution::project::Project;
    use vize_carton::typescript_project::FlattenedCompilerOptions;

    let same = |left: &Option<_>, right: &Option<_>| match (left, right) {
        (Some(left), Some(right)) => std::sync::Arc::ptr_eq(left, right),
        _ => false,
    };
    let _lock = lock_epoch_state();
    let newer = synthetic_epoch();
    let older = synthetic_epoch();
    assert!(older < newer);

    assert!(shared(None, NO_EPOCH).is_none());
    let first = shared(None, older);
    assert!(first.is_some());
    assert!(same(&first, &shared(None, older)));
    let next = shared(None, newer);
    assert!(next.is_some());
    assert!(!same(&first, &next));
    assert!(same(&next, &shared(None, newer)));
    assert!(shared(None, older).is_none());

    let config = Path::new("/project/tsconfig.json");
    let project = Project::from_flattened(config, FlattenedCompilerOptions::default()).unwrap();
    let rebased = Project::from_flattened(
        config,
        FlattenedCompilerOptions {
            base_url: Some("src".into()),
            ..FlattenedCompilerOptions::default()
        },
    )
    .unwrap();
    let configured = shared(Some(&project), newer);
    assert!(configured.is_some());
    assert!(!same(&configured, &next));
    assert!(same(&configured, &shared(Some(&project), newer)));
    assert!(shared(Some(&rebased), newer).is_none());

    retire();
    let reopened = shared(None, newer);
    assert!(reopened.is_some());
    assert!(!same(&reopened, &next));
    retire();
}
