//! The shared walk against the per-component reference on generated module
//! graphs: duplicate names across modules, import cycles, renamed and star
//! re-exports, Vue modules, imports that resolve to nothing and files that
//! cannot be read. A component must end with the same declarations, imports
//! and dependencies whichever walk resolved its types.

use std::path::Path;
use std::sync::Barrier;

use vize_carton::{FxHashMap, String};

use super::super::super::batch_epoch::NO_EPOCH;
use super::super::modules::ModuleGraph;
use super::super::walk::TypeDependency;
use super::super::{FileTypeSummary, ScriptCompileContext, extract_script_summary};
use super::{reference, synthetic_epoch};

/// xorshift64*: the same graphs on every run and every platform.
pub(super) struct Rng(pub(super) u64);

impl Rng {
    pub(super) fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as usize % bound
    }
}

/// Few names, so several modules declare the same one and order decides.
const NAMES: [&str; 5] = ["A", "B", "C", "D", "E"];

fn name(rng: &mut Rng) -> &'static str {
    NAMES[rng.below(NAMES.len())]
}

fn module_file(index: usize) -> std::string::String {
    if index % 7 == 3 {
        format!("m{index}.vue")
    } else {
        format!("m{index}.ts")
    }
}

fn specifier(rng: &mut Rng, modules: usize) -> std::string::String {
    let target = rng.below(modules);
    match rng.below(10) {
        0 => format!("./missing{target}"),
        _ if target % 7 == 3 => format!("./m{target}.vue"),
        _ => format!("./m{target}"),
    }
}

/// Imports and re-exports in every form the summary distinguishes.
fn edges(rng: &mut Rng, modules: usize, source: &mut std::string::String) {
    for _ in 0..rng.below(4) {
        let target = specifier(rng, modules);
        let (first, second) = (name(rng), name(rng));
        source.push_str(&match rng.below(8) {
            0 => format!("import type {{ {first}, {second} }} from '{target}'\n"),
            1 => format!("import {{ type {first} }} from '{target}'\n"),
            2 => format!("import type * as N{first} from '{target}'\n"),
            3 => format!("import type {{}} from '{target}'\n"),
            4 => format!("export type {{ {first} as {second} }} from '{target}'\n"),
            5 => format!("export {{ {first}, {second} }} from '{target}'\n"),
            _ => format!("export * from '{target}'\n"),
        });
    }
}

fn write_module(rng: &mut Rng, dir: &Path, index: usize, modules: usize) {
    let path = dir.join(module_file(index));
    if rng.below(16) == 0 {
        // Resolves, but is not text.
        std::fs::write(path, [0xff_u8, 0xfe, 0x00]).unwrap();
        return;
    }
    let mut source = std::string::String::new();
    for declaration in 0..rng.below(3) {
        let name = name(rng);
        source.push_str(&match rng.below(3) {
            0 => format!("export type {name} = {{ m{index}t{declaration}: number }}\n"),
            1 => format!("interface {name} {{ m{index}p{declaration}: string }}\n"),
            _ => format!("export interface {name} {{ m{index}i{declaration}: string }}\n"),
        });
    }
    edges(rng, modules, &mut source);
    if index % 7 == 3 {
        source = format!("<script lang=\"ts\">\n{source}</script>\n");
    }
    std::fs::write(path, source).unwrap();
}

fn roots(script: &str) -> Vec<TypeDependency> {
    let mut summary = FileTypeSummary::default();
    extract_script_summary(script, &mut summary, false);
    summary.dependencies
}

/// What a component's compile reads from a walk.
#[derive(Debug, PartialEq)]
struct Outcome {
    interfaces: FxHashMap<String, String>,
    type_aliases: FxHashMap<String, String>,
    /// Each import once, at its first position: a macro type is attributed
    /// to the first import of its name.
    imports: Vec<(String, String)>,
    files: Vec<String>,
}

fn outcome(
    local_alias: Option<&str>,
    walk: impl FnOnce(&mut ScriptCompileContext) -> (Vec<(String, String)>, Vec<String>),
) -> Outcome {
    let mut ctx = ScriptCompileContext::new("");
    if let Some(name) = local_alias {
        // A name the component's normal script already declared.
        ctx.type_aliases.insert(name.into(), "local".into());
    }
    let (walked, files) = walk(&mut ctx);
    let mut imports: Vec<(String, String)> = Vec::new();
    for import in walked {
        if !imports.iter().any(|seen| *seen == import) {
            imports.push(import);
        }
    }
    Outcome {
        interfaces: ctx.interfaces,
        type_aliases: ctx.type_aliases,
        imports,
        files,
    }
}

/// The reference walk, a walk over a graph of its own, and a walk over
/// `shared`, which earlier components of the same project already used.
/// Returns what every walk must produce.
fn assert_same_outcome(
    label: &str,
    script: &str,
    base_file: &Path,
    local_alias: Option<&str>,
    shared: &ModuleGraph,
) -> Outcome {
    let roots = roots(script);
    let expected = outcome(local_alias, |ctx| {
        reference::collect(ctx, &roots, base_file, None, NO_EPOCH)
    });
    let alone = outcome(local_alias, |ctx| {
        ctx.walk_imported_modules(&roots, base_file, None, None)
    });
    assert_eq!(alone, expected, "{label}: own graph\n{script}");
    for _ in 0..2 {
        let batch = outcome(local_alias, |ctx| {
            ctx.walk_imported_modules(&roots, base_file, None, Some(shared))
        });
        assert_eq!(batch, expected, "{label}: shared graph\n{script}");
    }
    expected
}

#[test]
fn shared_walk_matches_the_reference_on_generated_graphs() {
    for seed in 1..=160_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path().canonicalize().unwrap();
        let modules = 2 + rng.below(9);
        for index in 0..modules {
            write_module(&mut rng, &dir, index, modules);
        }
        let shared = ModuleGraph::new(synthetic_epoch());
        let mut components = Vec::new();
        for component in 0..8 {
            let mut script = std::string::String::new();
            while script.is_empty() {
                edges(&mut rng, modules, &mut script);
            }
            let local_alias = (rng.below(4) == 0).then(|| name(&mut rng));
            let base_file = dir.join(format!("Component{component}.vue"));
            let expected = assert_same_outcome(
                &format!("seed {seed}, component {component}"),
                &script,
                &base_file,
                local_alias,
                &shared,
            );
            components.push((roots(&script), base_file, local_alias, expected));
        }

        // The same components as the workers of one batch: they start
        // together on a graph nobody has walked, so they walk the same
        // modules, cycles included, at the same time.
        let batch = ModuleGraph::new(synthetic_epoch());
        let start = Barrier::new(components.len());
        std::thread::scope(|scope| {
            for (roots, base_file, local_alias, expected) in &components {
                let (batch, start) = (&batch, &start);
                scope.spawn(move || {
                    start.wait();
                    let together = outcome(*local_alias, |ctx| {
                        ctx.walk_imported_modules(roots, base_file, None, Some(batch))
                    });
                    assert_eq!(together, *expected, "seed {seed}: workers together");
                });
            }
        });
    }
}

/// `a` and `x` re-export each other, and `z` and `w` both declare `Foo`.
/// Entering at `a` reaches `w` first; entering at `x` reaches `z` first. A
/// closure therefore belongs to the module a root import resolved to, and is
/// never reused as a part of another module's walk.
#[test]
fn a_cycle_keeps_the_order_of_the_module_it_was_entered_from() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    std::fs::write(
        dir.join("a.ts"),
        "export * from './x'\nexport * from './z'\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("x.ts"),
        "export * from './a'\nexport * from './w'\n",
    )
    .unwrap();
    std::fs::write(dir.join("z.ts"), "export interface Foo { z: string }\n").unwrap();
    std::fs::write(dir.join("w.ts"), "export interface Foo { w: string }\n").unwrap();
    let base_file = dir.join("Component.vue");
    let shared = ModuleGraph::new(synthetic_epoch());
    let foo = |script: &str| {
        assert_same_outcome("cycle", script, &base_file, None, &shared);
        let mut ctx = ScriptCompileContext::new("");
        ctx.walk_imported_modules(&roots(script), &base_file, None, Some(&shared));
        ctx.interfaces
    };

    let through_a = foo("import type * as A from './a'\n");
    let through_x = foo("import type * as X from './x'\nimport type * as A from './a'\n");
    let through_a_again = foo("import type * as A from './a'\n");

    let declares = |body: &str| FxHashMap::from_iter([(String::from("Foo"), String::from(body))]);
    assert_eq!(through_a, declares("{ w: string }"));
    assert_eq!(through_x, declares("{ z: string }"));
    assert_eq!(through_a_again, declares("{ w: string }"));
}

/// A module that resolves but cannot be read is a failed attempt, not a fact
/// about the batch: the component that met it goes without, and the next one
/// reads again. What a batch did read stays as it was read.
#[test]
fn an_unreadable_module_is_read_again_by_the_next_component() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    std::fs::write(
        dir.join("barrel.ts"),
        "export * from './a'\nexport * from './b'\n",
    )
    .unwrap();
    std::fs::write(dir.join("a.ts"), "export interface A { a: string }\n").unwrap();
    std::fs::write(dir.join("b.ts"), [0xff_u8, 0xfe]).unwrap();
    let roots = roots("import type * as Types from './barrel'\n");
    let base_file = dir.join("Component.vue");
    let shared = ModuleGraph::new(synthetic_epoch());
    let generation = synthetic_epoch();
    let reference = || {
        outcome(None, |ctx| {
            reference::collect(ctx, &roots, &base_file, None, generation)
        })
    };
    let batch = || {
        outcome(None, |ctx| {
            ctx.walk_imported_modules(&roots, &base_file, None, Some(&shared))
        })
    };
    let declares = |names: &[(&str, &str)]| -> FxHashMap<String, String> {
        names
            .iter()
            .map(|(name, body)| (String::from(*name), String::from(*body)))
            .collect()
    };

    let unread = batch();
    assert_eq!(unread, reference());
    assert_eq!(unread.interfaces, declares(&[("A", "{ a: string }")]));

    std::fs::write(dir.join("b.ts"), "export interface B { b: string }\n").unwrap();
    let read = batch();
    assert_eq!(read, reference());
    assert_eq!(
        read.interfaces,
        declares(&[("A", "{ a: string }"), ("B", "{ b: string }")])
    );
    assert_eq!(read.files, unread.files);

    std::fs::write(dir.join("b.ts"), "export interface B { c: string }\n").unwrap();
    assert_eq!(batch(), read);
    assert_eq!(reference(), read);
}

/// `import * as` asks for a whole module; `import { "*" as }` asks for the
/// one export named `*`. Components of one batch that ask for each get their
/// own walk, in either order.
#[test]
fn a_namespace_import_and_an_export_named_star_are_different_requests() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    std::fs::write(
        dir.join("barrel.ts"),
        "export { Other } from './other'\nexport { Star as \"*\" } from './star'\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("other.ts"),
        "export interface Other { other: string }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("star.ts"),
        "export interface Star { star: string }\n",
    )
    .unwrap();
    let base_file = dir.join("Component.vue");
    let namespace = "import type * as Types from './barrel'\n";
    let named = "import type { \"*\" as Star } from './barrel'\n";
    let file = |name: &str| super::super::resolution::path_key(&dir.join(name));

    for order in [[named, namespace], [namespace, named]] {
        let shared = ModuleGraph::new(synthetic_epoch());
        for script in order {
            let walked = assert_same_outcome("star", script, &base_file, None, &shared);
            let expected = if script == namespace {
                vec![file("barrel.ts"), file("other.ts"), file("star.ts")]
            } else {
                vec![file("barrel.ts"), file("star.ts")]
            };
            assert_eq!(walked.files, expected, "{script}");
        }
    }
}
