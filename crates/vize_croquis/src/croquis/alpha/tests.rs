use super::{
    ALPHA_CONTRACT_SCHEMA, AlphaSchema, ComponentContract, EmitContract, PropContract,
    ReactivityContract, SignatureContract, SlotContract, declaration_key,
};
use crate::Croquis;
use crate::drawer::{Drawer, DrawerOptions};
use crate::macros::{EmitDefinition, ModelDefinition, PropDefinition, SlotsDefinition};
use vize_armature::parse;
use vize_carton::CompactString;
use vize_l2::summary::{Facet, SfcSummary};

fn draw(script: &str, template: &str) -> Croquis {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template must be valid");
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(script);
    drawer.draw_template(&root);
    drawer.finish()
}

fn summary(croquis: &Croquis) -> SfcSummary {
    SfcSummary::from_alpha(
        croquis
            .alpha_pages("Component", None)
            .expect("JSON contracts"),
    )
    .expect("valid alpha declarations")
}

#[test]
fn body_edits_and_private_bindings_do_not_dirty_the_interface() {
    let before = draw(
        "import { ref } from 'vue'; defineProps<{ title: string }>(); \
         defineEmits<{ save: [value: number] }>(); const count = ref(1); \
         defineExpose({ count }); const privateState = ref('a'); function work() { return 1; }",
        "<button @click='work'>{{count}}</button>",
    );
    let after = draw(
        "import { ref } from 'vue'; defineProps<{ title: string }>(); \
         defineEmits<{ save: [value: number] }>(); const privateAdded = ref(99); \
         const count = ref(42); defineExpose({ count }); const privateState = ref('b'); \
         function work() { return privateAdded.value * 2; }",
        "<button @click='work'>changed body {{count + 1}}</button>",
    );
    assert!(summary(&before).changed(&summary(&after)).is_empty());
    let pages = before.alpha_pages("Component", None).expect("pages");
    assert_eq!(pages.reactivity.len(), 1);
    let count: ReactivityContract =
        serde_json::from_str(&pages.reactivity.first().expect("exposed count").contract)
            .expect("typed contract");
    assert_eq!(count.name, "count");
    assert_eq!(count.kind.as_deref(), Some("ref"));
    assert_eq!(count.verdict, "proven");
}

#[test]
fn type_literal_whitespace_is_semantic_without_dirtying_sibling_facets() {
    let before = draw("defineProps<{ title: 'a b' }>();", "<p></p>");
    let after = draw("defineProps<{ title: 'ab' }>();", "<p></p>");
    let changed = summary(&before).changed(&summary(&after));
    // The signature conservatively keeps exact macro type arguments until
    // extraction can prove completeness, so it changes along with the prop.
    assert_eq!(changed.len(), 2);
    assert!(changed.iter().any(|decl| decl.facet() == Facet::Signature));
    let declaration = changed
        .iter()
        .find(|decl| decl.facet() == Facet::Prop)
        .expect("changed prop");
    assert_eq!(declaration.name(), "title");
}

#[test]
fn aliases_and_repeated_usages_export_one_resolved_component() {
    let before = draw(
        "import { Child as Alias } from './child.vue';",
        "<Alias/><Alias/>",
    );
    let after = draw(
        "import { Child as Renamed } from './child.vue';",
        "<Renamed/>",
    );
    let before = before.alpha_pages("Component", None).expect("pages");
    let after = after.alpha_pages("Component", None).expect("pages");
    assert_eq!(before.components, after.components);
    assert_eq!(before.components.len(), 1);
    let identity: ComponentContract =
        serde_json::from_str(&before.components.first().expect("component").contract)
            .expect("typed identity");
    assert_eq!(identity.module.as_deref(), Some("./child.vue"));
    assert_eq!(identity.export, "Child");
}

#[test]
fn defaults_models_generics_and_unknown_contracts_remain_structured() {
    let mut croquis = Croquis::new();
    croquis
        .macros
        .set_define_options_name(Some("Declared".into()));
    croquis.macros.add_prop(PropDefinition {
        name: "label".into(),
        prop_type: Some("'a b'".into()),
        required: false,
        default_value: Some("'a b'".into()),
    });
    croquis.macros.add_model(ModelDefinition {
        name: "title".into(),
        local_name: "localModel".into(),
        model_type: Some("T".into()),
        required: true,
        default_value: Some("defaultTitle".into()),
    });
    croquis
        .macros
        .set_model_modifier_type("title".into(), "'trim' | 'a b'".into());
    croquis.macros.add_emit(EmitDefinition {
        name: "save".into(),
        payload_type: None,
    });
    croquis.macros.add_slot(SlotsDefinition {
        name: "default".into(),
        props_type: None,
    });
    croquis.bindings.add("unknown", crate::BindingType::Props);
    let pages = croquis
        .alpha_pages("Public", Some("T extends 'a b'"))
        .expect("pages");
    let signature: SignatureContract =
        serde_json::from_str(&pages.signature.params).expect("signature");
    assert_eq!(signature.name, "Public");
    assert_eq!(signature.declared_name.as_deref(), Some("Declared"));
    assert_eq!(signature.generic.as_deref(), Some("T extends 'a b'"));
    let props: Vec<PropContract> = pages
        .props
        .iter()
        .map(|entry| serde_json::from_str(&entry.contract).expect("typed prop"))
        .collect();
    let model = props
        .iter()
        .find(|prop| prop.name == "title")
        .expect("model");
    assert_eq!(model.required, Some(true));
    assert_eq!(model.default_value.as_deref(), Some("defaultTitle"));
    assert_eq!(model.model_modifiers.as_deref(), Some("'trim' | 'a b'"));
    assert!(pages.emits.iter().any(|entry| entry.name == "update:title"));
    let unknown = props
        .iter()
        .find(|prop| prop.name == "unknown")
        .expect("unknown prop");
    assert_eq!(unknown.required, None);
    assert_eq!(unknown.prop_type, None);
    let model_reactivity: ReactivityContract =
        serde_json::from_str(&pages.reactivity.first().expect("model reactivity").contract)
            .expect("reactivity");
    assert_eq!(model_reactivity.verdict, "unknown");
    assert_eq!(model_reactivity.class, None);
    assert!(SfcSummary::from_alpha(pages).is_ok());
}

#[test]
fn unusual_names_have_injective_keys_and_roundtrip_original_spelling() {
    let mut croquis = Croquis::new();
    for name in ["a b", "a=b", "%612062", "雪\n", ""] {
        croquis.macros.add_prop(PropDefinition {
            name: CompactString::new(name),
            prop_type: Some("'a b' |\n 'ab'".into()),
            required: false,
            default_value: None,
        });
    }
    let pages = croquis.alpha_pages("odd = component", None).expect("pages");
    assert_eq!(pages.props.len(), 5);
    assert_eq!(declaration_key("ordinary"), "ordinary");
    assert_ne!(declaration_key("a b"), declaration_key("%612062"));
    for entry in &pages.props {
        assert!(!entry.name.chars().any(|ch| ch.is_whitespace() || ch == '='));
        assert!(!entry.contract.contains('\n'));
        let prop: PropContract =
            serde_json::from_str(&entry.contract).expect("JSON escaped literal");
        assert_eq!(declaration_key(&prop.name), entry.name);
        assert_eq!(prop.prop_type.as_deref(), Some("'a b' |\n 'ab'"));
    }
    assert!(SfcSummary::from_alpha(pages).is_ok());
}

#[test]
fn known_macro_prop_wins_over_duplicate_model_and_binding_fallback() {
    let mut croquis = Croquis::new();
    croquis.bindings.add("title", crate::BindingType::Props);
    croquis.macros.add_prop(PropDefinition {
        name: "title".into(),
        prop_type: Some("string".into()),
        required: true,
        default_value: None,
    });
    croquis.macros.add_model(ModelDefinition {
        name: "title".into(),
        local_name: "model".into(),
        model_type: Some("number".into()),
        required: false,
        default_value: Some("1".into()),
    });
    let pages = croquis.alpha_pages("Component", None).expect("pages");
    assert_eq!(pages.props.len(), 1);
    let prop: PropContract =
        serde_json::from_str(&pages.props.first().expect("title").contract).expect("typed prop");
    assert_eq!(prop.prop_type.as_deref(), Some("string"));
    assert_eq!(prop.required, Some(true));
    assert_eq!(prop.default_value, None);
}

#[test]
fn every_production_facet_requires_the_supported_schema() {
    let mut croquis = draw("import Child from './Child.vue';", "<Child/>");
    croquis.macros.add_prop(PropDefinition {
        name: "label".into(),
        prop_type: None,
        required: false,
        default_value: None,
    });
    croquis.macros.add_model(ModelDefinition {
        name: "title".into(),
        local_name: "model".into(),
        model_type: None,
        required: false,
        default_value: None,
    });
    croquis.macros.add_slot(SlotsDefinition {
        name: "default".into(),
        props_type: None,
    });
    let pages = croquis.alpha_pages("Component", None).expect("pages");
    assert_eq!(Croquis::ALPHA_CONTRACT_SCHEMA, ALPHA_CONTRACT_SCHEMA);
    assert_eq!(
        serde_json::to_value(AlphaSchema).expect("schema"),
        serde_json::json!(1)
    );
    check_schema::<SignatureContract>(&pages.signature.params);
    check_schema::<PropContract>(&pages.props.first().expect("prop").contract);
    check_schema::<EmitContract>(&pages.emits.first().expect("model emit").contract);
    check_schema::<SlotContract>(&pages.slots.first().expect("slot").contract);
    check_schema::<ReactivityContract>(&pages.reactivity.first().expect("model").contract);
    check_schema::<ComponentContract>(&pages.components.first().expect("component").contract);
}

fn check_schema<T: serde::de::DeserializeOwned>(contract: &str) {
    assert!(serde_json::from_str::<T>(contract).is_ok());
    let original: serde_json::Value = serde_json::from_str(contract).expect("JSON contract");
    let mut missing = original.clone();
    missing
        .as_object_mut()
        .expect("contract object")
        .remove("schema");
    assert!(
        serde_json::from_value::<T>(missing).is_err(),
        "missing schema must be rejected"
    );
    let mut extended = original.clone();
    extended
        .as_object_mut()
        .expect("contract object")
        .insert("future_field".into(), serde_json::json!(true));
    assert!(
        serde_json::from_value::<T>(extended).is_err(),
        "unknown fields require an explicit contract schema"
    );
    for invalid in [
        serde_json::json!(0),
        serde_json::json!(2),
        serde_json::json!(65536),
        serde_json::json!(-1),
        serde_json::json!(1.0),
        serde_json::json!("1"),
        serde_json::Value::Null,
    ] {
        let mut unsupported = original.clone();
        unsupported
            .as_object_mut()
            .expect("contract object")
            .insert("schema".into(), invalid);
        assert!(
            serde_json::from_value::<T>(unsupported).is_err(),
            "unsupported schema must be rejected"
        );
    }
}

#[test]
fn signature_preserves_existing_declared_and_binding_completion_order() {
    let mut croquis = draw(
        "defineProps<{ z: string; a: number }>(); defineSlots<{ z(props: {}): void; a(props: {}): void }>();",
        "<div></div>",
    );
    for (name, kind) in [
        ("unknownZ", crate::BindingType::Props),
        ("private", crate::BindingType::SetupConst),
        ("unknownA", crate::BindingType::Props),
    ] {
        croquis.bindings.add(name, kind);
    }
    for name in ["z", "model"] {
        croquis.macros.add_model(ModelDefinition {
            name: name.into(),
            local_name: name.into(),
            model_type: None,
            required: false,
            default_value: None,
        });
    }
    let mut expected = vec!["z", "a", "model"];
    for (name, kind) in croquis.bindings.iter() {
        if kind == crate::BindingType::Props && !expected.contains(&name) {
            expected.push(name);
        }
    }
    let pages = croquis.alpha_pages("Component", None).expect("pages");
    let signature: SignatureContract =
        serde_json::from_str(&pages.signature.params).expect("signature");
    assert_eq!(signature.prop_order, expected);
    assert_eq!(signature.slot_order, ["z", "a"]);
    assert!(
        !signature.props_complete,
        "Drawer-only catalog has no completeness proof"
    );
    assert_eq!(pages.props[0].name, "a");
}
