use crate::{Drawer, DrawerOptions, ScopeKind, TemplateExpressionKind};
use serde_json::{Value, json};
use vize_armature::parse;
use vize_carton::Allocator;

fn component_usage_json(source: &str) -> Value {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let snapshot = drawer.finish().semantic_snapshot();
    serde_json::to_value(snapshot.component_usages).unwrap()
}

fn range(source: &str, needle: &str) -> Value {
    let start = source.find(needle).unwrap() as u32;
    json!({ "start": start, "end": start + needle.len() as u32 })
}

#[test]
fn directive_argument_dynamics_have_an_exact_semantic_contract() {
    let source = concat!(
        "<p>雪</p>",
        "<Child title=\"fixed\" :foo=\"value\" :short ",
        "v-bind:[propName]=\"other\" :[shorthand] ",
        "v-model=\"model\" v-model:page=\"page\" ",
        "v-model:[modelName]=\"dynamicModel\" ",
        "@click=\"onClick\" @[eventName].once=\"onEvent\">",
        "<template #[slotName]=\"{ row }\">{{ row }}</template>",
        "<span>fallback</span>",
        "</Child>",
    );
    let component_start = source.find("<Child").unwrap() as u32;
    let component_open_end = source.find("<template").unwrap() as u32;

    let expected = json!([{
        "id": format!("component:Child@{component_start}"),
        "name": "Child",
        "range": { "start": component_start, "end": component_open_end },
        "scopeId": 0,
        "vifGuard": null,
        "hasSpreadAttrs": false,
        "props": [
            {
                "name": "title",
                "nameIsDynamic": false,
                "value": "fixed",
                "range": range(source, "title=\"fixed\""),
                "dynamic": false
            },
            {
                "name": "foo",
                "nameIsDynamic": false,
                "value": "value",
                "range": range(source, ":foo=\"value\""),
                "dynamic": true
            },
            {
                "name": "short",
                "nameIsDynamic": false,
                "value": "short",
                "range": range(source, ":short"),
                "dynamic": true
            },
            {
                "name": "propName",
                "nameIsDynamic": true,
                "value": "other",
                "range": range(source, "v-bind:[propName]=\"other\""),
                "dynamic": true
            },
            {
                "name": "shorthand",
                "nameIsDynamic": true,
                "value": "shorthand",
                "range": range(source, ":[shorthand]"),
                "dynamic": true
            },
            {
                "name": "modelValue",
                "nameIsDynamic": false,
                "value": "model",
                "range": range(source, "v-model=\"model\""),
                "dynamic": true
            },
            {
                "name": "page",
                "nameIsDynamic": false,
                "value": "page",
                "range": range(source, "v-model:page=\"page\""),
                "dynamic": true
            },
            {
                "name": "modelName",
                "nameIsDynamic": true,
                "value": "dynamicModel",
                "range": range(source, "v-model:[modelName]=\"dynamicModel\""),
                "dynamic": true
            }
        ],
        "events": [
            {
                "name": "update:modelValue",
                "nameIsDynamic": false,
                "handler": "model",
                "modifiers": [],
                "range": range(source, "v-model=\"model\"")
            },
            {
                "name": "update:page",
                "nameIsDynamic": false,
                "handler": "page",
                "modifiers": [],
                "range": range(source, "v-model:page=\"page\"")
            },
            {
                "name": "update:modelName",
                "nameIsDynamic": true,
                "handler": "dynamicModel",
                "modifiers": [],
                "range": range(source, "v-model:[modelName]=\"dynamicModel\"")
            },
            {
                "name": "click",
                "nameIsDynamic": false,
                "handler": "onClick",
                "modifiers": [],
                "range": range(source, "@click=\"onClick\"")
            },
            {
                "name": "eventName",
                "nameIsDynamic": true,
                "handler": "onEvent",
                "modifiers": ["once"],
                "range": range(source, "@[eventName].once=\"onEvent\"")
            }
        ],
        "slots": [{
            "name": "slotName",
            "nameIsDynamic": true,
            "scopeVars": ["row"],
            "range": range(source, "#[slotName]=\"{ row }\""),
            "scoped": true
        }, {
            "name": "default",
            "nameIsDynamic": false,
            "scopeVars": [],
            "range": range(source, "<span>"),
            "scoped": false
        }]
    }]);

    let first = component_usage_json(source);
    assert_eq!(first, expected);
    assert_eq!(component_usage_json(source), first);
}

#[test]
fn dynamic_name_and_dynamic_value_are_independent() {
    let source = "<Child static-prop :bound=\"value\" :[name]=\"value\" />";
    let usage = component_usage_json(source);
    let props = usage[0]["props"].as_array().unwrap();

    assert_eq!(props[0]["dynamic"], false);
    assert_eq!(props[0]["nameIsDynamic"], false);
    assert_eq!(props[1]["dynamic"], true);
    assert_eq!(props[1]["nameIsDynamic"], false);
    assert_eq!(props[2]["dynamic"], true);
    assert_eq!(props[2]["nameIsDynamic"], true);
}

#[test]
fn literal_template_slot_names_are_static() {
    let source = concat!(
        "<Child>",
        "<template #[`item.name`]=\"{ item }\">{{ item.name }}</template>",
        "<template #[`item.${suffix}`]=\"{ row }\">{{ row }}</template>",
        "<template #[`item.\\${name}`]=\"{ escaped }\">{{ escaped }}</template>",
        "<template #[`item.\\`name\\``]=\"{ quoted }\">{{ quoted }}</template>",
        "</Child>",
    );
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup("const suffix = 'name';");
    drawer.draw_template(&root);
    let croquis = drawer.finish();
    let usage = serde_json::to_value(&croquis.semantic_snapshot().component_usages).unwrap();

    assert_eq!(usage[0]["slots"][0]["name"], "item.name");
    assert_eq!(usage[0]["slots"][0]["nameIsDynamic"], false);
    assert_eq!(usage[0]["slots"][1]["name"], "`item.${suffix}`");
    assert_eq!(usage[0]["slots"][1]["nameIsDynamic"], true);
    assert_eq!(usage[0]["slots"][2]["name"], "item.${name}");
    assert_eq!(usage[0]["slots"][2]["nameIsDynamic"], false);
    assert_eq!(usage[0]["slots"][3]["name"], "item.`name`");
    assert_eq!(usage[0]["slots"][3]["nameIsDynamic"], false);

    let arguments = croquis
        .template_expressions
        .iter()
        .filter(|expression| expression.kind == TemplateExpressionKind::DynamicDirectiveArgument)
        .map(|expression| expression.content.as_str())
        .collect::<Vec<_>>();
    assert_eq!(arguments, ["`item.${suffix}`"]);
}

#[test]
fn runtime_directive_arguments_are_checked_as_expressions() {
    let source = concat!(
        "<Child :[known]=\"value\" @[missingEvent]=\"handler\">",
        "<template #[missingSlot]=\"{ row }\">{{ row }}</template>",
        "</Child>",
    );
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup("const known = 'title'; const value = 1; const handler = () => {};");
    drawer.draw_template(&root);
    let croquis = drawer.finish();

    let arguments = croquis
        .template_expressions
        .iter()
        .filter(|expression| expression.kind == TemplateExpressionKind::DynamicDirectiveArgument)
        .map(|expression| {
            (
                expression.content.as_str(),
                expression.start,
                expression.end,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        arguments,
        [
            (
                "known",
                source.find("known").unwrap() as u32,
                (source.find("known").unwrap() + "known".len()) as u32,
            ),
            (
                "missingEvent",
                source.find("missingEvent").unwrap() as u32,
                (source.find("missingEvent").unwrap() + "missingEvent".len()) as u32,
            ),
            (
                "missingSlot",
                source.find("missingSlot").unwrap() as u32,
                (source.find("missingSlot").unwrap() + "missingSlot".len()) as u32,
            ),
        ]
    );
    let undefined = croquis
        .undefined_refs
        .iter()
        .map(|reference| (reference.name.as_str(), reference.offset))
        .collect::<Vec<_>>();
    assert_eq!(
        undefined,
        [
            ("missingEvent", source.find("missingEvent").unwrap() as u32,),
            ("missingSlot", source.find("missingSlot").unwrap() as u32,),
        ]
    );
}

#[test]
fn dynamic_slot_name_sees_v_for_alias_but_not_its_own_slot_props() {
    let source = concat!(
        "<Child>",
        "<template v-for=\"(_, slotName) in $slots\" #[slotName]=\"{ row }\">{{ row }}</template>",
        "<template #[localOnly]=\"{ localOnly }\">{{ localOnly }}</template>",
        "<template #[emptySlot]><span></span></template>",
        "</Child>",
        "<Other :[after] />",
    );
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup("const emptySlot = 'empty'; const after = 'title';");
    drawer.draw_template(&root);
    let croquis = drawer.finish();

    let arguments = croquis
        .template_expressions
        .iter()
        .filter(|expression| expression.kind == TemplateExpressionKind::DynamicDirectiveArgument)
        .map(|expression| {
            (
                expression.content.as_str(),
                croquis.scopes.get_scope(expression.scope_id).unwrap().kind,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        arguments,
        [
            ("slotName", ScopeKind::VFor),
            ("localOnly", ScopeKind::ScriptSetup),
            ("emptySlot", ScopeKind::ScriptSetup),
            ("after", ScopeKind::ScriptSetup),
        ]
    );

    let undefined = croquis
        .undefined_refs
        .iter()
        .map(|reference| reference.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(undefined, ["localOnly"]);
}
