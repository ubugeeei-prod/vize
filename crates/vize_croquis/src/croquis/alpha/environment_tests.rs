use super::PropContract;
use crate::Croquis;
use crate::drawer::{Drawer, DrawerOptions};
use vize_l2::summary::{Facet, SfcSummary};

fn draw(source: &str) -> Croquis {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(source);
    drawer.finish()
}

fn summary(croquis: &Croquis) -> SfcSummary {
    SfcSummary::from_alpha(croquis.alpha_pages("Component", None).expect("pages")).expect("summary")
}

#[test]
fn reachable_alias_edits_dirty_their_prop_and_exposed_member() {
    let before = draw(
        "type Public = string; type Private = 1; \
        defineProps<{ value: Public; sibling: boolean; literal: 'Public' }>(); \
        const item: Public = null as Public; defineExpose({ item });",
    );
    let after = draw(
        "type Public = number; type Private = 1; \
        defineProps<{ value: Public; sibling: boolean; literal: 'Public' }>(); \
        const item: Public = null as Public; defineExpose({ item });",
    );
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
    let private_edit = draw(
        "type Public = string; type Private = 'unused changed'; \
        defineProps<{ value: Public; sibling: boolean; literal: 'Public' }>(); \
        const item: Public = null as Public; defineExpose({ item });",
    );
    assert!(summary(&before).changed(&summary(&private_edit)).is_empty());
}

#[test]
fn cycles_terminate_and_interface_heritage_is_a_dependency() {
    let croquis = draw(
        "type A = { next: B }; type B = { next: A }; \
        interface Base { value: string }; interface Derived extends Base { enabled: boolean }; \
        defineProps<{ tree: A; derived: Derived }>();",
    );
    let pages = croquis.alpha_pages("Component", None).expect("pages");
    let tree: PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "tree")
            .expect("tree")
            .contract,
    )
    .expect("contract");
    assert!(tree.type_dependencies.complete);
    assert_eq!(tree.type_dependencies.declarations.len(), 2);
    let derived: PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "derived")
            .expect("derived")
            .contract,
    )
    .expect("contract");
    assert!(
        derived
            .type_dependencies
            .declarations
            .iter()
            .any(|declaration| declaration.name == "Base")
    );
    assert!(
        derived
            .type_dependencies
            .declarations
            .iter()
            .any(|declaration| declaration.name == "Derived" && declaration.extends == ["Base"])
    );
}

#[test]
fn generic_function_mapped_and_infer_bindings_do_not_capture_global_aliases() {
    let croquis = draw(
        "type T = string; type K = string; type U = string; \
        type Box<T> = { value: T }; \
        defineProps<{ box: Box<number>; fn: <T>(value: T) => T; \
          map: { [K in 'key']: K }; inferred: string extends infer U ? U : never; \
          inferredFn: (() => string) extends (() => infer U) ? U : never }>();",
    );
    let pages = croquis.alpha_pages("Component", Some("T")).expect("pages");
    for entry in &pages.props {
        let prop: PropContract = serde_json::from_str(&entry.contract).expect("contract");
        assert!(
            !prop
                .type_dependencies
                .declarations
                .iter()
                .any(|dependency| matches!(dependency.name.as_str(), "T" | "K" | "U")),
            "lexical type binders must shadow outer aliases"
        );
    }
}

#[test]
fn imported_type_aliases_retain_exported_identity_and_remain_unresolved() {
    let croquis = draw(
        "import type { Public as Alias } from './types'; \
        import type * as Types from './types'; \
        defineProps<{ value: Alias; nested: Types.Nested; literal: 'Alias' }>();",
    );
    let pages = croquis.alpha_pages("Component", None).expect("pages");
    let value: PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "value")
            .expect("value")
            .contract,
    )
    .expect("contract");
    assert!(!value.type_dependencies.complete);
    let imported = value
        .type_dependencies
        .declarations
        .first()
        .expect("external identity");
    assert_eq!(imported.module.as_deref(), Some("./types"));
    assert_eq!(imported.export.as_deref(), Some("Public"));
    let literal: PropContract = serde_json::from_str(
        &pages
            .props
            .iter()
            .find(|entry| entry.name == "literal")
            .expect("literal")
            .contract,
    )
    .expect("contract");
    assert!(literal.type_dependencies.complete);
    assert!(literal.type_dependencies.declarations.is_empty());
}

#[test]
fn generic_bounds_and_model_modifier_aliases_are_reachable_contracts() {
    let mut before = draw(
        "type Bound = string; type Flags = 'trim'; type T = 'private'; \
        const model = defineModel<string, Flags>(); defineProps<{ value: T }>();",
    );
    let mut after = draw(
        "type Bound = number; type Flags = 'trim'; type T = 'private'; \
        const model = defineModel<string, Flags>(); defineProps<{ value: T }>();",
    );
    let before_pages = before
        .alpha_pages("Component", Some("T extends Bound"))
        .expect("pages");
    let after_pages = after
        .alpha_pages("Component", Some("T extends Bound"))
        .expect("pages");
    let before_summary = SfcSummary::from_alpha(before_pages).expect("summary");
    let after_summary = SfcSummary::from_alpha(after_pages).expect("summary");
    assert!(
        before_summary
            .changed(&after_summary)
            .iter()
            .any(|id| id.facet() == Facet::Prop && id.name() == "value")
    );
    // Use tracker facts directly as well: the modifier type is an independent
    // authored annotation, even when a parser cannot elaborate its alias.
    before
        .macros
        .set_model_modifier_type("modelValue".into(), "Flags".into());
    after
        .macros
        .set_model_modifier_type("modelValue".into(), "Flags".into());
    after.types.add_type_alias("Flags", "'capitalize'");
    assert!(
        summary(&before)
            .changed(&summary(&after))
            .iter()
            .any(|id| id.facet() == Facet::Prop && id.name() == "modelValue")
    );
}
