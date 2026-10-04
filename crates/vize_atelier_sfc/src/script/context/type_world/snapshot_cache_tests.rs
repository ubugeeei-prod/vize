use super::super::{ScriptCompileContext, TypeSourceSnapshot};
use std::sync::Arc;
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
        let source = format!(
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
    // Target filling must not mutate the reusable unresolved module facts.
    let modules = sources.modules.lock().unwrap();
    let (api, _) = modules
        .get(&api.canonicalize().unwrap())
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(api.exports.values().all(|export| match export {
        vize_croquis::types::world::TypeExportBinding::Forward { target, .. } =>
            target.module.is_none(),
        _ => true,
    }));
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
                let filename = dir.path().join(format!("Host{index}.vue"));
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
