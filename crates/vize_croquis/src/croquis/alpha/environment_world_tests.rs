use super::{PropContract, SignatureContract};
use crate::Croquis;
use crate::drawer::{Drawer, DrawerOptions};
use crate::types::world::{
    ResolvedTypeWorld, TypeDeclaration, TypeDeclarationKind, TypeExportBinding, TypeImport,
    TypeModule, TypeModuleReference,
};
use vize_carton::{CompactString, cstr};
use vize_l2::summary::{Facet, SfcSummary};

fn declaration(name: &str, body: &str) -> TypeDeclaration {
    TypeDeclaration {
        kind: TypeDeclarationKind::Alias,
        body: CompactString::new(body),
        extends: Vec::new(),
        type_parameters: None,
        declaration_source: cstr!("type {name} = {body};"),
    }
}

fn world(helper: &str) -> ResolvedTypeWorld {
    let target = TypeModuleReference {
        specifier: "./types".into(),
        module: Some("/types.ts".into()),
    };
    let mut root = TypeModule {
        complete: true,
        ..Default::default()
    };
    root.imports.insert(
        "Alias".into(),
        TypeImport {
            target: target.clone(),
            exported: Some("Public".into()),
        },
    );
    root.direct_imports.insert("./types".into(), target);
    root.declarations
        .insert("Helper".into(), declaration("Helper", "boolean"));
    let mut external = TypeModule {
        complete: true,
        ..Default::default()
    };
    external.declarations.insert(
        "Internal".into(),
        declaration("Internal", "{ value: Helper }"),
    );
    external
        .declarations
        .insert("Helper".into(), declaration("Helper", helper));
    external
        .exports
        .insert("Public".into(), TypeExportBinding::Local("Internal".into()));
    ResolvedTypeWorld {
        root_module: "/Component.vue".into(),
        modules: [
            ("/Component.vue".into(), root),
            ("/types.ts".into(), external),
        ]
        .into_iter()
        .collect(),
    }
}

fn croquis(source: &str, world: ResolvedTypeWorld) -> Croquis {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(source);
    let mut croquis = drawer.finish();
    croquis.types.set_resolved_world(world);
    croquis
}

fn summary(croquis: &Croquis) -> SfcSummary {
    SfcSummary::from_alpha(croquis.alpha_pages("Component", None).expect("pages")).expect("summary")
}

#[test]
fn imported_alias_fingerprints_follow_the_declaring_modules_own_helpers() {
    let source = "import type { Public as Alias } from './types'; type Helper = boolean; \
        defineProps<{ value: Alias; sibling: boolean; literal: 'Alias' }>(); \
        const item: Alias = null as Alias; defineExpose({ item });";
    let before = croquis(source, world("string"));
    let after = croquis(source, world("number"));
    let changed = summary(&before).changed(&summary(&after));
    assert!(
        changed
            .iter()
            .any(|id| id.facet() == Facet::Prop && id.name() == "value")
    );
    assert!(
        changed
            .iter()
            .any(|id| id.facet() == Facet::Reactivity && id.name() == "item")
    );
    assert!(
        !changed
            .iter()
            .any(|id| id.facet() == Facet::Prop && matches!(id.name(), "sibling" | "literal"))
    );
    let pages = before.alpha_pages("Component", None).expect("pages");
    let value: PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "value")
            .expect("value")
            .contract,
    )
    .expect("contract");
    assert!(value.type_dependencies.complete);
    assert_eq!(value.prop_type.as_deref(), Some("Alias"));
    assert!(
        value
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Helper"
                && dependency.module.as_deref() == Some("/types.ts")
                && dependency.body.as_deref() == Some("string"))
    );
    assert!(
        !value
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.module.as_deref() == Some("/Component.vue"))
    );
    let mut private_edit = world("string");
    private_edit
        .modules
        .get_mut("/Component.vue")
        .expect("root")
        .declarations
        .insert(
            "Helper".into(),
            declaration("Helper", "'unrelated private edit'"),
        );
    assert!(
        summary(&before)
            .changed(&summary(&croquis(source, private_edit)))
            .is_empty()
    );
}

#[test]
fn direct_import_type_changes_and_opaque_macro_arguments_reach_their_contracts() {
    let source = "defineProps<{ value: import('./types').Public; sibling: boolean }>();";
    let before = croquis(source, world("string"));
    let after = croquis(source, world("number"));
    let changed = summary(&before).changed(&summary(&after));
    assert!(
        changed
            .iter()
            .any(|id| id.facet() == Facet::Prop && id.name() == "value")
    );
    assert!(
        !changed
            .iter()
            .any(|id| id.facet() == Facet::Prop && id.name() == "sibling")
    );
    let opaque = "import type { Public as Alias } from './types'; defineProps<Alias>();";
    let before = croquis(opaque, world("string"));
    let after = croquis(opaque, world("number"));
    assert!(
        summary(&before)
            .changed(&summary(&after))
            .iter()
            .any(|id| id.facet() == Facet::Signature)
    );
    let pages = before.alpha_pages("Component", None).expect("pages");
    let signature: SignatureContract =
        serde_json::from_str(&pages.signature.params).expect("signature");
    assert!(signature.type_dependencies.complete);
    assert!(
        signature
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Internal")
    );
}

#[test]
fn imported_declarations_do_not_capture_sfc_generic_parameters() {
    let mut type_world = world("T");
    type_world
        .modules
        .get_mut("/types.ts")
        .expect("external")
        .declarations
        .insert("T".into(), declaration("T", "'external literal'"));
    let croquis = croquis(
        "import type { Public as Alias } from './types'; defineProps<{ value: Alias }>();",
        type_world,
    );
    let pages = croquis.alpha_pages("Component", Some("T")).expect("pages");
    let prop: PropContract =
        serde_json::from_str(&pages.props.first().expect("value").contract).expect("prop");
    assert!(prop.type_dependencies.complete);
    assert!(
        prop.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "T"
                && dependency.body.as_deref() == Some("'external literal'"))
    );
}

#[test]
fn expanded_imported_prop_retains_the_scoped_macro_contract() {
    let source = "import type { Public as Alias } from './types'; type Helper = boolean; \
        defineProps<Alias>();";
    let build = |helper| {
        let mut result = croquis(source, world(helper));
        result.macros.add_prop(crate::macros::PropDefinition {
            name: "value".into(),
            prop_type: Some("Helper".into()),
            required: true,
            default_value: None,
        });
        result
    };
    let before = build("string");
    let after = build("number");
    assert_ne!(
        summary(&before).fingerprint(Facet::Prop, "value"),
        summary(&after).fingerprint(Facet::Prop, "value")
    );
    let pages = before.alpha_pages("Component", None).expect("pages");
    let prop: PropContract = serde_json::from_str(&pages.props[0].contract).expect("contract");
    assert!(
        prop.type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.name == "Helper"
                && dependency.module.as_deref() == Some("/types.ts")
                && dependency.body.as_deref() == Some("string"))
    );
    assert!(
        !prop
            .type_dependencies
            .declarations
            .iter()
            .any(|dependency| dependency.module.as_deref() == Some("/Component.vue"))
    );
}

#[test]
fn unsupported_merge_parts_and_their_helpers_remain_observable() {
    let source = "import type { Public as Alias } from './types'; \
        defineProps<{ value: Alias; sibling: boolean }>(); \
        const item: Alias = null as Alias; defineExpose({ item });";
    let merged_world = |first: &str, helper: &str| {
        let mut result = world(helper);
        let module = result.modules.get_mut("/types.ts").expect("types");
        module.unsupported_declarations.insert("Internal".into());
        module.unsupported_contracts.insert(
            "Internal".into(),
            vec![
                cstr!("interface Internal {{ first: {first} }}"),
                "interface Internal { value: Helper }".into(),
            ],
        );
        result
    };
    let before = croquis(source, merged_world("string", "string"));
    for (first, helper) in [("number", "string"), ("string", "number")] {
        let after = croquis(source, merged_world(first, helper));
        let changed = summary(&before).changed(&summary(&after));
        for facet in [Facet::Prop, Facet::Reactivity] {
            assert!(changed.iter().any(|id| id.facet() == facet
                && id.name()
                    == if facet == Facet::Prop {
                        "value"
                    } else {
                        "item"
                    }));
        }
        assert!(
            !changed
                .iter()
                .any(|id| id.facet() == Facet::Prop && id.name() == "sibling")
        );
    }
    let pages = before.alpha_pages("Component", None).expect("pages");
    let prop: PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "value")
            .expect("value")
            .contract,
    )
    .expect("contract");
    assert!(!prop.type_dependencies.complete);
    let dependency = prop
        .type_dependencies
        .declarations
        .iter()
        .find(|dependency| dependency.name == "Internal")
        .expect("merged type");
    assert_eq!(
        dependency.body.as_deref(),
        Some("36:interface Internal { first: string }36:interface Internal { value: Helper }")
    );
}
