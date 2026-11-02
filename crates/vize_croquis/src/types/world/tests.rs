use super::{
    ResolvedTypeWorld, TypeDeclaration, TypeDeclarationKind, TypeExportBinding, TypeImport,
    TypeLookup, TypeModule, TypeModuleReference, UnknownTypeReason,
};
use std::sync::Arc;
use vize_carton::{CompactString, FxHashMap};

fn target(module: &str) -> TypeModuleReference {
    TypeModuleReference {
        specifier: CompactString::new(module),
        module: Some(CompactString::new(module)),
    }
}

fn declaration(module: &mut TypeModule, name: &str, body: &str) {
    module.declarations.insert(
        CompactString::new(name),
        TypeDeclaration {
            kind: TypeDeclarationKind::Alias,
            body: CompactString::new(body),
            extends: Vec::new(),
            type_parameters: None,
            declaration_source: CompactString::new(body),
        },
    );
    module.exports.insert(
        CompactString::new(name),
        TypeExportBinding::Local(CompactString::new(name)),
    );
}

fn module() -> TypeModule {
    TypeModule {
        complete: true,
        ..TypeModule::default()
    }
}

fn world() -> ResolvedTypeWorld {
    let mut root = module();
    root.imports.insert(
        CompactString::new("Public"),
        TypeImport {
            target: target("barrel"),
            exported: Some(CompactString::new("Exported")),
        },
    );
    root.imports.insert(
        CompactString::new("Api"),
        TypeImport {
            target: target("barrel"),
            exported: None,
        },
    );
    let mut barrel = module();
    barrel.exports.insert(
        CompactString::new("Exported"),
        TypeExportBinding::Forward {
            target: target("api"),
            exported: CompactString::new("PrivateName"),
        },
    );
    let mut api = module();
    declaration(&mut api, "PrivateName", "{ value: string }");
    let mut unrelated = module();
    declaration(&mut unrelated, "Public", "{ wrong: number }");
    ResolvedTypeWorld {
        root_module: CompactString::new("root"),
        modules: FxHashMap::from_iter([
            (CompactString::new("root"), Arc::new(root)),
            (CompactString::new("barrel"), Arc::new(barrel)),
            (CompactString::new("api"), Arc::new(api)),
            (CompactString::new("unrelated"), Arc::new(unrelated)),
        ]),
    }
}

#[test]
fn aliases_and_namespace_members_keep_declaration_identity() {
    let world = world();
    let TypeLookup::Found(id) = world.resolve("root", "Public") else {
        panic!("public import should resolve")
    };
    assert_eq!(id.module, "api");
    assert_eq!(id.name, "PrivateName");
    assert_eq!(
        world.resolve("root", "Api.Exported"),
        TypeLookup::Found(id.clone())
    );
    assert_eq!(world.declaration(&id).unwrap().body, "{ value: string }");
    assert!(matches!(
        world.resolve("api", "Public"),
        TypeLookup::Unknown(_)
    ));
}

#[test]
fn missing_star_does_not_prove_a_unique_export() {
    for reverse in [false, true] {
        let mut world = world();
        let mut stars = vec![target("api"), target("missing")];
        if reverse {
            stars.reverse();
        }
        Arc::make_mut(world.modules.get_mut("barrel").unwrap()).star_exports = stars;
        let TypeLookup::Unknown(reference) = world.resolve("root", "Api.PrivateName") else {
            panic!("missing star may contain an ambiguous export")
        };
        assert_eq!(reference.reason, UnknownTypeReason::MissingModule);
    }
}

#[test]
fn duplicate_stars_accept_the_same_identity_and_reject_distinct_declarations() {
    let mut world = world();
    Arc::make_mut(world.modules.get_mut("barrel").unwrap()).star_exports =
        vec![target("api"), target("api")];
    assert!(matches!(
        world.resolve("root", "Api.PrivateName"),
        TypeLookup::Found(_)
    ));
    let mut duplicate = module();
    declaration(&mut duplicate, "PrivateName", "number");
    world
        .modules
        .insert(CompactString::new("duplicate"), Arc::new(duplicate));
    Arc::make_mut(world.modules.get_mut("barrel").unwrap())
        .star_exports
        .push(target("duplicate"));
    let TypeLookup::Unknown(reference) = world.resolve("root", "Api.PrivateName") else {
        panic!("distinct declarations are ambiguous")
    };
    assert_eq!(reference.reason, UnknownTypeReason::AmbiguousExport);
}

#[test]
fn reexport_cycles_are_bounded_and_explicit() {
    let mut world = world();
    Arc::make_mut(world.modules.get_mut("barrel").unwrap())
        .exports
        .insert(
            CompactString::new("Loop"),
            TypeExportBinding::Forward {
                target: target("barrel"),
                exported: CompactString::new("Loop"),
            },
        );
    let TypeLookup::Unknown(reference) = world.resolve("root", "Api.Loop") else {
        panic!("cycle cannot resolve")
    };
    assert_eq!(reference.reason, UnknownTypeReason::ResolutionCycle);
}
