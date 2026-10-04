//! Public type-world workload: snapshot creation is outside warmed timings.
#![expect(
    clippy::disallowed_types,
    reason = "standalone benchmark uses shared source snapshots"
)]

use sha2::{Digest, Sha256};
use std::{hint::black_box, path::Path, time::Instant};
use vize_atelier_sfc::script::{ScriptCompileContext, TypeSourceSnapshot};
use vize_croquis::types::{ResolvedTypeWorld, TypeLookup};

fn ordered<T: std::fmt::Debug>(
    map: &vize_carton::FxHashMap<vize_carton::String, T>,
) -> Vec<String> {
    let mut fields: Vec<_> = map
        .iter()
        .map(|(key, value)| format!("{key}:{value:?}"))
        .collect();
    fields.sort();
    fields
}

fn contract(world: &ResolvedTypeWorld) -> String {
    let mut modules: Vec<_> = world
        .modules
        .iter()
        .map(|(path, module)| {
            let mut unsupported: Vec<_> = module.unsupported_declarations.iter().collect();
            unsupported.sort();
            format!(
                "{path}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
                module.complete,
                ordered(&module.declarations),
                unsupported,
                ordered(&module.unsupported_contracts),
                ordered(&module.imports),
                ordered(&module.exports),
                module.star_exports,
                ordered(&module.direct_imports),
                world.root_module
            )
        })
        .collect();
    modules.sort();
    modules.join("\n")
}

fn prepare(root: &Path, shape: &str) -> String {
    std::fs::create_dir_all(root).unwrap();
    let count = if shape == "local-only" { 0 } else { 32 };
    for index in 0..count {
        let mut source =
            format!("export interface Public{index} {{ value: string; index: {index} }}\n");
        for declaration in 0..64 {
            source.push_str(&format!("type Private{declaration}<T> = {{ value: T; optional?: string; nested: {{ key: number }} }};\n"));
        }
        if shape == "barrels" && index > 0 {
            source.push_str(&format!("export * from './api{}';\n", index - 1));
        }
        if shape == "mixed-syntax" {
            source.push_str("const render = () => <div><span>view</span></div>;\n");
            if index % 2 == 0 {
                source = format!(
                    "<script setup lang='tsx'>{source}</script><template><div /></template>"
                );
            }
        }
        let extension = if shape == "mixed-syntax" {
            if index % 2 == 0 { "vue" } else { "tsx" }
        } else {
            "ts"
        };
        std::fs::write(root.join(format!("api{index}.{extension}")), source).unwrap();
    }
    let imports = match shape {
        "barrels" => "import type { Public0 } from './api31';\n".to_owned(),
        "local-only" => "type Public0 = { value: string; index: 0 };\n".to_owned(),
        _ => (0..count)
            .map(|index| format!("import type {{ Public{index} }} from './api{index}';\n"))
            .collect(),
    };
    let members = if shape == "barrels" || count == 0 {
        1
    } else {
        count
    };
    let types: String = (0..members)
        .map(|index| format!("p{index}: Public{index};"))
        .collect();
    format!("{imports}type Props = {{ {types} }};\n")
}

fn measure(root: &Path, source: &str, workers: usize, warmed: bool) -> (u128, String, usize) {
    let sources = TypeSourceSnapshot::default();
    let hosts = if warmed { 128 } else { 1 };
    if warmed {
        let path = root.join("warmup.vue");
        black_box(
            ScriptCompileContext::new(source).resolve_type_world_with_sources(
                path.to_str().unwrap(),
                None,
                false,
                &sources,
            ),
        );
    }
    let contexts: Vec<_> = (0..hosts)
        .map(|index| {
            (
                root.join(format!("Host{index}.vue")),
                ScriptCompileContext::new(&format!("{source}type Root = {index};")),
            )
        })
        .collect();
    let start = Instant::now();
    let worlds = std::thread::scope(|scope| {
        let handles: Vec<_> = contexts
            .chunks(hosts.div_ceil(workers))
            .map(|chunk| {
                let sources = &sources;
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|(path, ctx)| {
                            ctx.resolve_type_world_with_sources(
                                path.to_str().unwrap(),
                                None,
                                false,
                                sources,
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let elapsed = start.elapsed().as_nanos();
    let mut digest = Sha256::new();
    let mut modules = 0;
    for (index, world) in worlds.iter().enumerate() {
        let TypeLookup::Found(root_type) = world.resolve(&world.root_module, "Root") else {
            panic!("root must resolve")
        };
        assert_eq!(
            world.declaration(&root_type).unwrap().body,
            index.to_string()
        );
        assert!(matches!(
            world.resolve(&world.root_module, "Public0"),
            TypeLookup::Found(_)
        ));
        modules += world.modules.len();
        digest.update(contract(world));
    }
    (elapsed, format!("{:x}", digest.finalize()), modules)
}

fn main() {
    let output = std::env::args().nth(1).expect("output JSON path");
    let directory = std::env::var("TYPE_SNAPSHOT_FIXTURE_DIR").expect("shared fixture directory");
    let root = Path::new(&directory);
    let mut rows = Vec::new();
    for shape in ["local-only", "fanout", "barrels", "mixed-syntax"] {
        let dir = root.join(shape);
        let source = prepare(&dir, shape);
        for (workers, warmed) in [(1usize, false), (1, true), (4, true)] {
            let (elapsed_ns, signature, modules) = measure(&dir, &source, workers, warmed);
            rows.push(serde_json::json!({
                "id": format!("{shape}-{}-{workers}t", if warmed { "warm" } else { "cold" }),
                "hosts": if warmed { 128 } else { 1 }, "modules": modules,
                "signature": signature, "elapsed_ns": elapsed_ns,
            }));
        }
    }
    let report = serde_json::json!({
        "sha": std::env::var("MEASURE_SHA").unwrap(),
        "side": std::env::var("MEASURE_SIDE").unwrap(), "rows": rows,
    });
    std::fs::write(output, serde_json::to_string_pretty(&report).unwrap()).unwrap();
}
