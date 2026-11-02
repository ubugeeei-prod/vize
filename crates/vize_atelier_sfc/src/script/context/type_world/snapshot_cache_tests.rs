use super::super::{ScriptCompileContext, TypeSourceSnapshot};
use std::sync::Arc;
use vize_carton::cstr;
use vize_croquis::types::{ResolvedTypeWorld, TypeLookup};

fn body(world: &ResolvedTypeWorld, name: &str) -> vize_carton::String {
    let TypeLookup::Found(id) = world.resolve(&world.root_module, name) else {
        panic!("authored type must resolve: {name}")
    };
    world.declaration(&id).unwrap().body.clone()
}

#[test]
fn warm_dependencies_preserve_root_buffers_and_private_lexical_bindings() {
    let dir = tempfile::tempdir().unwrap();
    let api = dir.path().join("api.ts");
    std::fs::write(&api, "type Private = string; export type Public<T = Private> = { item: T }; export type { Other } from './other'").unwrap();
    std::fs::write(dir.path().join("other.ts"), "export type Other = boolean").unwrap();
    let filename = dir.path().join("App.vue");
    let sources = TypeSourceSnapshot::default();
    for root_type in ["number", "string", "boolean"] {
        let source = cstr!(
            "import type {{ Public, Other }} from './api'; type Root = {root_type}; type Props = Public<Root> & {{ flag: Other }}"
        );
        let world = ScriptCompileContext::new(&source).resolve_type_world_with_sources(
            filename.to_str().unwrap(),
            None,
            false,
            &sources,
        );
        assert_eq!(body(&world, "Root"), root_type);
        assert_eq!(body(&world, "Other"), "boolean");
        let TypeLookup::Found(public) = world.resolve(&world.root_module, "Public") else {
            panic!("public generic resolves")
        };
        let TypeLookup::Found(private) = world.resolve(&public.module, "Private") else {
            panic!("dependency private binding resolves in its declaring module")
        };
        assert_eq!(world.declaration(&private).unwrap().body, "string");
        assert!(matches!(
            world.resolve(&world.root_module, "Private"),
            TypeLookup::Unknown(_)
        ));
    }
    // Targets are filled once per snapshot; the shared resolved facts carry
    // them to every world, while the three root buffers never enter the cache.
    let resolved = sources.resolved.lock().unwrap();
    assert_eq!(resolved.len(), 2);
    let api = resolved
        .get(&api.canonicalize().unwrap())
        .unwrap()
        .get()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(api.module.exports.values().all(|export| match export {
        vize_croquis::types::world::TypeExportBinding::Forward { target, .. } =>
            target.module.is_some(),
        _ => true,
    }));
}

/// Sorted facts of every dependency module, with the private root left out.
fn dependency_signature(world: &ResolvedTypeWorld) -> Vec<vize_carton::String> {
    let mut rows: Vec<_> = world
        .modules
        .iter()
        .filter(|(key, _)| **key != world.root_module)
        .map(|(key, module)| cstr!("{key}: {module:?}"))
        .collect();
    rows.sort();
    rows
}

#[test]
fn shared_snapshot_freezes_dependency_text_and_new_revision_refreshes_it() {
    let dir = tempfile::tempdir().unwrap();
    let api = dir.path().join("api.tsx");
    let filename = dir.path().join("App.vue");
    std::fs::write(&api, "export type Public = string; const node = <div />").unwrap();
    let source = "import type { Public } from './api'; type Root = Public";
    let ctx = ScriptCompileContext::new(source);
    let saved = TypeSourceSnapshot::default();
    assert_eq!(
        body(
            &ctx.resolve_type_world_with_sources(filename.to_str().unwrap(), None, false, &saved),
            "Public"
        ),
        "string"
    );
    std::fs::write(&api, "export type Public = number; const node = <span />").unwrap();
    assert_eq!(
        body(
            &ctx.resolve_type_world_with_sources(filename.to_str().unwrap(), None, false, &saved),
            "Public"
        ),
        "string"
    );
    let edited = TypeSourceSnapshot::new([(
        api,
        Arc::<str>::from("export type Public = boolean; const node = <p />"),
    )]);
    for _ in 0..2 {
        assert_eq!(
            body(
                &ctx.resolve_type_world_with_sources(
                    filename.to_str().unwrap(),
                    None,
                    false,
                    &edited
                ),
                "Public"
            ),
            "boolean"
        );
    }
    let closed = TypeSourceSnapshot::default();
    assert_eq!(
        body(
            &ctx.resolve_type_world_with_sources(filename.to_str().unwrap(), None, false, &closed),
            "Public"
        ),
        "number"
    );
}

#[test]
fn parallel_consumers_keep_cycles_and_missing_exports_explicit() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.ts"),
        "export * from './b'; export type Public = string",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("b.ts"),
        "export * from './a'; export type { Missing } from './absent'",
    )
    .unwrap();
    let sources = TypeSourceSnapshot::default();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|index| {
                let sources = &sources;
                let filename = dir.path().join(cstr!("Host{index}.vue").as_str());
                scope.spawn(move || {
                    let ctx = ScriptCompileContext::new(
                        "import type { Public, Missing } from './a'; type Root = Public",
                    );
                    let world = ctx.resolve_type_world_with_sources(
                        filename.to_str().unwrap(),
                        None,
                        false,
                        sources,
                    );
                    assert_eq!(body(&world, "Public"), "string");
                    assert!(matches!(
                        world.resolve(&world.root_module, "Missing"),
                        TypeLookup::Unknown(_)
                    ));
                    assert_eq!(world.modules.len(), 3);
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn warmed_dependency_facts_do_not_expand_the_world_admission_limit() {
    let dir = tempfile::tempdir().unwrap();
    let filename = dir.path().join("Host.vue");
    let count = super::MAX_MODULES + 1;
    let sources = TypeSourceSnapshot::new((0..count).map(|index| {
        (
            dir.path().join(cstr!("api{index}.ts").as_str()),
            Arc::<str>::from(cstr!("export type Type{index} = {index};").as_str()),
        )
    }));
    for index in 0..count {
        let source =
            cstr!("import type {{ Type{index} }} from './api{index}'; type Root = Type{index}");
        let world = ScriptCompileContext::new(&source).resolve_type_world_with_sources(
            filename.to_str().unwrap(),
            None,
            false,
            &sources,
        );
        assert_eq!(body(&world, &cstr!("Type{index}")), cstr!("{index}"));
    }
    let mut source = vize_carton::String::default();
    for index in 0..count {
        source.push_str(&cstr!(
            "import type {{ Type{index} }} from './api{index}';\n"
        ));
    }
    source.push_str("type Root = [");
    for index in 0..count {
        source.push_str(&cstr!("Type{index},"));
    }
    source.push_str("];");
    let world = ScriptCompileContext::new(&source).resolve_type_world_with_sources(
        filename.to_str().unwrap(),
        None,
        false,
        &sources,
    );
    assert_eq!(
        world
            .modules
            .values()
            .filter(|module| module.complete)
            .count(),
        super::MAX_MODULES
    );
    assert_eq!(
        world
            .modules
            .values()
            .filter(|module| !module.complete)
            .count(),
        2
    );
}

#[test]
fn dependency_modules_are_resolved_once_per_snapshot_and_shared_between_worlds() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("api.ts"),
        "export type { Other } from './other'; export * from './models'; export type Public = { item: Other }",
    )
    .unwrap();
    std::fs::write(dir.path().join("other.ts"), "export type Other = boolean").unwrap();
    std::fs::write(
        dir.path().join("models.ts"),
        "export interface Model { id: number }",
    )
    .unwrap();
    let sources = TypeSourceSnapshot::default();
    let resolve = |name: &str, sources: &TypeSourceSnapshot| {
        let source = cstr!(
            "import type {{ Public, Model }} from './api'; type Props{name} = {{ item: Public; model: Model }}"
        );
        ScriptCompileContext::new(&source).resolve_type_world_with_sources(
            dir.path().join(cstr!("{name}.vue")).to_str().unwrap(),
            None,
            false,
            sources,
        )
    };
    let first = resolve("First", &sources);
    let second = resolve("Second", &sources);

    // Both roots see the same declarations through the shared dependencies.
    let models_key = super::path_key(&super::source_path(&dir.path().join("models.ts")));
    for world in [&first, &second] {
        assert_eq!(body(world, "Public"), "{ item: Other }");
        let TypeLookup::Found(model) = world.resolve(&world.root_module, "Model") else {
            panic!("star-exported interface resolves")
        };
        assert_eq!(model.module, models_key);
        assert_eq!(world.modules.len(), 4);
    }
    assert_ne!(first.root_module, second.root_module);

    // Dependencies are resolved once per snapshot and shared; roots never are.
    for dependency in ["api.ts", "other.ts", "models.ts"] {
        let key = first
            .modules
            .keys()
            .find(|key| key.ends_with(dependency))
            .unwrap();
        assert!(
            Arc::ptr_eq(&first.modules[key], &second.modules[key]),
            "{dependency} must be resolved once per snapshot"
        );
    }
    let resolved = sources.resolved.lock().unwrap();
    assert_eq!(resolved.len(), 3);
    assert!(
        resolved
            .keys()
            .all(|path| path.extension().is_some_and(|ext| ext == "ts")),
        "root components never enter the resolved dependency cache: {resolved:?}"
    );
    drop(resolved);

    // A new snapshot resolves again with identical facts: nothing is shared
    // across snapshots, and sharing changes no fact.
    let fresh = TypeSourceSnapshot::default();
    let third = resolve("Third", &fresh);
    let api = first
        .modules
        .keys()
        .find(|key| key.ends_with("api.ts"))
        .unwrap();
    assert!(!Arc::ptr_eq(&first.modules[api], &third.modules[api]));
    assert_eq!(body(&third, "Public"), "{ item: Other }");
    assert_eq!(dependency_signature(&first), dependency_signature(&third));
    assert_eq!(dependency_signature(&first), dependency_signature(&second));
}

#[test]
fn barrel_dependencies_are_resolved_once_however_many_worlds_import_them() {
    let dir = tempfile::tempdir().unwrap();
    let models = dir.path().join("models");
    std::fs::create_dir(&models).unwrap();
    std::fs::write(
        dir.path().join("runtime.ts"),
        "export function request(path: string): Promise<unknown> { return fetch(path) }",
    )
    .unwrap();
    let mut barrel = vize_carton::String::default();
    for index in 0..32 {
        std::fs::write(
            models.join(cstr!("m{index}.ts").as_str()),
            cstr!(
                "import {{ request }} from '../runtime'; export interface Model{index} {{ id: number }} export function fetch{index}() {{ return request('/m{index}') }}"
            ),
        )
        .unwrap();
        barrel.push_str(&cstr!("export * from './models/m{index}';\n"));
    }
    std::fs::write(dir.path().join("api.ts"), barrel.as_str()).unwrap();

    let sources = TypeSourceSnapshot::default();
    let worlds: Vec<_> = (0..8)
        .map(|index| {
            let source = cstr!(
                "import type {{ Model{index} }} from './api'; defineProps<{{ item: Model{index} }}>()"
            );
            ScriptCompileContext::new(&source).resolve_type_world_with_sources(
                dir.path()
                    .join(cstr!("Host{index}.vue").as_str())
                    .to_str()
                    .unwrap(),
                None,
                false,
                &sources,
            )
        })
        .collect();

    // Each component reaches the whole barrel through `export *`: its root,
    // the barrel and all 32 models; the value-only runtime import is not a
    // type reference and stays out.
    for (index, world) in worlds.iter().enumerate() {
        let TypeLookup::Found(id) = world.resolve(&world.root_module, &cstr!("Model{index}"))
        else {
            panic!("barrel export resolves")
        };
        let model_key = super::path_key(&super::source_path(
            &models.join(cstr!("m{index}.ts").as_str()),
        ));
        assert_eq!(id.module, model_key);
        assert_eq!(world.modules.len(), 34);
    }
    // The 33 dependencies were resolved exactly once for all eight worlds.
    assert_eq!(sources.resolved.lock().unwrap().len(), 33);
    let first = &worlds[0];
    for world in &worlds[1..] {
        for (key, module) in &first.modules {
            if *key == first.root_module {
                continue;
            }
            assert!(Arc::ptr_eq(module, &world.modules[key]), "{key}");
        }
        assert_eq!(dependency_signature(first), dependency_signature(world));
    }
}
