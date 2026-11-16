use super::super::{
    batch::{PluginSpec, build_batch},
    facts::{REGISTRY, resolve_demands},
};
use super::*;

const SOURCE: &str = r#"<script setup lang="ts" generic="T extends 'a b'">
import ChildAlias from './Child.vue'
import type { Public as Alias } from './types'
import { ref } from 'vue'
defineOptions({ name: 'PublicComponent' })
const props = defineProps<{ label: 'a b'; choice: Alias; value?: T }>()
defineEmits<{ save: [value: T] }>()
defineSlots<{ default(props: { item: T }): any }>()
const model = defineModel<string>('title')
const count = ref(0)
defineExpose({ count })
function privateBody() { return 1 }
</script><template><ChildAlias>{{ props.label }} {{ privateBody() }}</ChildAlias></template>"#;

fn names() -> Vec<String> {
    Facet::ALL
        .into_iter()
        .map(|facet| facet.group().to_owned())
        .collect()
}

fn projected(source: &str) -> Map<String, Value> {
    project(
        &PluginDocument::build(source, "Public.vue").unwrap(),
        &names(),
    )
    .unwrap()
}

fn raw_rows(entries: &[vize_l2::summary::AlphaEntry]) -> Value {
    Value::Array(
        entries
            .iter()
            .map(|entry| {
                json!([
                    entry.name.as_str(),
                    serde_json::from_str::<Value>(&entry.contract).unwrap()
                ])
            })
            .collect(),
    )
}

#[test]
fn all_six_groups_equal_the_actual_producer_values_and_keys() {
    let document = PluginDocument::build(SOURCE, "src/Fancy = file.vue").unwrap();
    let croquis = document.production.get(SOURCE).unwrap();
    let pages = croquis
        .alpha_pages(&document.filename, Some("T extends 'a b'"))
        .unwrap();
    let found = project(&document, &names()).unwrap();
    let expected = Map::from_iter([
        (
            Facet::Signature.group().to_owned(),
            json!([[
                pages.signature.name.as_str(),
                serde_json::from_str::<Value>(&pages.signature.params).unwrap()
            ]]),
        ),
        (Facet::Prop.group().to_owned(), raw_rows(&pages.props)),
        (Facet::Emit.group().to_owned(), raw_rows(&pages.emits)),
        (Facet::Slot.group().to_owned(), raw_rows(&pages.slots)),
        (
            Facet::Reactivity.group().to_owned(),
            raw_rows(&pages.reactivity),
        ),
        (
            Facet::Component.group().to_owned(),
            raw_rows(&pages.components),
        ),
    ]);
    assert_eq!(found, expected);
    for rows in found.values() {
        assert!(!rows.as_array().unwrap().is_empty());
    }
    let signature = found
        .get("component-signature")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .get(1)
        .unwrap();
    assert_eq!(signature.get("generic"), Some(&json!("T extends 'a b'")));
    assert_eq!(signature.get("name"), Some(&json!("src/Fancy = file.vue")));
    assert_eq!(
        signature.get("declared_name"),
        Some(&json!("PublicComponent"))
    );
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE
    );
    assert!(core::ptr::eq(
        document
            .production
            .pages(SOURCE, &document.filename)
            .unwrap(),
        document
            .production
            .pages(SOURCE, &document.filename)
            .unwrap()
    ));
}

#[test]
fn alpha_demands_filter_batches_and_leave_beta_managers_independent() {
    let document = PluginDocument::build(SOURCE, "Public.vue").unwrap();
    let demands = vec!["prop-types".to_owned()];
    assert_eq!(resolve_demands("alpha", &demands).unwrap(), Demand::NONE);
    let spec = PluginSpec {
        name: "alpha",
        version: "1",
        fingerprint: "code",
        visit: None,
        demands: &demands,
    };
    let mut manager = FactManager::new(&REGISTRY);
    let batch: Value =
        serde_json::from_str(&build_batch(&document, &spec, &mut manager).unwrap().json).unwrap();
    let facts = batch.get("facts").unwrap().as_object().unwrap();
    assert_eq!(facts.keys().collect::<Vec<_>>(), vec!["prop-types"]);
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE
    );
    assert_eq!(manager.computed(), Demand::NONE);
    let both = project(&document, &["prop-types".into(), "bindings".into()]).unwrap();
    assert_eq!(both.len(), 2);
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE.with(Bindings::ID)
    );
}

#[test]
fn body_edits_reuse_interface_values_but_interface_edits_change_them() {
    let before = projected(SOURCE);
    let body = SOURCE
        .replace("return 1", "return 2")
        .replace("props.label }}", "props.label }} body changed");
    assert_eq!(projected(&body), before);
    let interface = SOURCE.replace("label: 'a b'", "label: 'ab'");
    let changed = projected(&interface);
    assert_ne!(changed.get("prop-types"), before.get("prop-types"));
    assert_eq!(
        changed.get("component-references"),
        before.get("component-references")
    );
}

#[test]
fn literal_whitespace_external_type_dependencies_and_nulls_survive() {
    let found = projected(SOURCE);
    let props = found.get("prop-types").unwrap().as_array().unwrap();
    let get = |name| {
        props
            .iter()
            .find(|row| row.get(0).and_then(Value::as_str) == Some(name))
            .unwrap()
            .get(1)
            .unwrap()
    };
    assert_eq!(get("label").get("type"), Some(&json!("'a b'")));
    let dependencies = get("choice").get("type_dependencies").unwrap();
    assert_eq!(dependencies.get("complete"), Some(&json!(false)));
    let imported = dependencies
        .get("declarations")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|dependency| dependency.get("name") == Some(&json!("Alias")))
        .unwrap();
    assert_eq!(imported.get("module"), Some(&json!("./types")));
    assert_eq!(imported.get("export"), Some(&json!("Public")));
    assert_eq!(get("label").get("default"), Some(&Value::Null));
    let mut croquis = Croquis::new();
    croquis
        .bindings
        .add("unknown", vize_croquis::BindingType::Props);
    let pages = croquis.alpha_pages("Unknown.vue", None).unwrap();
    let unknown = alpha::project(&pages, &["prop-types".into()]).unwrap();
    let value = unknown
        .get("prop-types")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .get(1)
        .unwrap();
    for field in ["type", "required", "default", "model_modifiers"] {
        assert_eq!(value.get(field), Some(&Value::Null));
    }
    assert_eq!(
        value.get("type_dependencies").unwrap().get("complete"),
        Some(&json!(false))
    );
}

#[test]
fn unsupported_schemas_unknown_nested_fields_and_missing_nulls_refuse() {
    let croquis = PluginDocument::build(SOURCE, "Public.vue").unwrap();
    let pages = croquis.production.pages(SOURCE, "Public.vue").unwrap();
    for mutate in [
        ("schema", json!(2)),
        ("unregistered_field", json!(true)),
        (
            "type_dependencies",
            json!({"complete": false, "declarations": [], "unregistered": true}),
        ),
    ] {
        let mut changed = pages.clone();
        let prop = changed.props.first_mut().unwrap();
        let mut value: Value = serde_json::from_str(&prop.contract).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert(mutate.0.into(), mutate.1);
        prop.contract = value.to_string().into();
        assert!(alpha::project(&changed, &["prop-types".into()]).is_err());
    }
    let mut changed = pages.clone();
    let prop = changed.props.first_mut().unwrap();
    let mut value: Value = serde_json::from_str(&prop.contract).unwrap();
    value.as_object_mut().unwrap().remove("default");
    prop.contract = value.to_string().into();
    assert!(alpha::project(&changed, &["prop-types".into()]).is_err());
}

#[test]
fn alpha_only_foreign_requests_refuse_without_changing_beta_state() {
    let document = PluginDocument::build(
        "<script setup lang=\"moonbit\">let x = 1</script><template>{{ x }}</template>",
        "Foreign.vue",
    )
    .unwrap();
    assert!(project(&document, &["component-signature".into()]).is_err());
    assert_eq!(
        document.production.manager.lock().unwrap().computed(),
        Demand::NONE
    );
    assert!(project(&document, &[]).unwrap().is_empty());
}
