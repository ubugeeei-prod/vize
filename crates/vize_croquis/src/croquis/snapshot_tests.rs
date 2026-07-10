use super::{CroquisSemanticSnapshot, TemplateExpressionKind};
use crate::{Drawer, DrawerOptions};

#[test]
fn semantic_snapshot_collects_contract_facts_from_parsed_sfc_parts() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let script = r#"
import { computed, inject, provide, reactive, ref } from 'vue'

const props = defineProps<{ msg: string }>()
const emit = defineEmits<{ (e: 'save', value: string): void }>()
const count = ref(0)
const state = reactive({ name: 'Ada' })
const doubled = computed(() => count.value * 2)
const theme = inject('theme', 'light')

provide('theme', theme)

function save() {
  emit('save', props.msg)
}
"#;
    let template = r#"<section>
  <Child v-if="count > 0" :msg="props.msg" @save.once="save" v-bind="$attrs">
    <template #default="{ row }">{{ row }}</template>
  </Child>
  <input id="name" v-model="state.name" />
</section>"#;

    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse without errors");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(script);
    drawer.draw_template(&root);

    let snapshot = drawer.finish().semantic_snapshot();

    assert_eq!(snapshot.summary.component_usage_count, 1);
    assert_eq!(snapshot.summary.provide_count, 1);
    assert_eq!(snapshot.summary.inject_count, 1);
    assert_eq!(snapshot.summary.reactive_source_count, 4);
    assert_eq!(snapshot.summary.v_if_expression_count, 1);
    assert_eq!(snapshot.summary.v_model_expression_count, 1);

    let count = snapshot
        .bindings
        .iter()
        .find(|binding| binding.name == "count")
        .expect("count binding should be exposed through the facade");
    assert_eq!(count.kind, "setupRef");
    assert_eq!(count.category, "setup");
    assert!(count.needs_value_in_script);

    let child = snapshot
        .component_usages
        .iter()
        .find(|usage| usage.name == "Child")
        .expect("Child component usage should be exposed");
    assert_eq!(child.props[0].name, "msg");
    assert_eq!(child.events[0].name, "save");
    assert_eq!(child.events[0].modifiers, ["once"]);
    assert!(child.has_spread_attrs);
    assert!(child.slots.iter().any(|slot| {
        slot.name == "default" && slot.scoped && slot.scope_vars.iter().any(|name| name == "row")
    }));

    assert!(
        snapshot
            .template_expressions
            .iter()
            .any(|expression| expression.kind == "vIf" && expression.content == "count > 0")
    );
    assert!(
        snapshot
            .template_expressions
            .iter()
            .any(|expression| expression.kind == "vModel" && expression.content == "state.name")
    );
    assert!(snapshot.provides.iter().any(|provide| {
        provide.key == "theme" && provide.key_kind == "string" && provide.value == "theme"
    }));
    assert!(snapshot.injects.iter().any(|inject| {
        inject.key == "theme"
            && inject.local_name == "theme"
            && inject.default_value.as_deref() == Some("'light'")
    }));
    let doubled = snapshot
        .reactive_sources
        .iter()
        .find(|source| source.name == "doubled")
        .expect("computed source should be exposed");
    let doubled_offset = script.find("doubled").unwrap() as u32;
    assert_eq!(doubled.kind, "computed");
    assert_eq!(doubled.category, "computed");
    assert_eq!(doubled.declaration_offset, doubled_offset);
    assert_eq!(doubled.id, format!("reactive:doubled@{doubled_offset}"));
    assert!(
        snapshot
            .scopes
            .iter()
            .any(|scope| scope.kind == "v-slot" && scope.bindings.iter().any(|b| b.name == "row"))
    );

    let json = serde_json::to_string(&snapshot).expect("snapshot should serialize");
    assert!(json.contains(r#""componentUsages""#));
    assert!(json.contains(r#""reactiveSources""#));
}

#[test]
fn semantic_snapshot_is_deterministic_for_manual_croquis_facts() {
    use super::{
        ComponentUsage, Croquis, EventListener, PassedProp, SlotUsage, TemplateExpression,
    };
    use crate::BindingType;
    use crate::provide::{InjectPattern, ProvideKey};
    use crate::reactivity::{ReactiveKind, ReactivityLoss, ReactivityLossKind};
    use crate::scope::ScopeId;
    use vize_carton::{CompactString, smallvec};

    let mut croquis = Croquis::new();
    croquis.bindings.add("zeta", BindingType::SetupConst);
    croquis.bindings.add("alpha", BindingType::SetupRef);
    croquis
        .binding_spans
        .insert(CompactString::new("alpha"), (20, 25));
    croquis.component_usages.push(ComponentUsage {
        name: CompactString::new("Panel"),
        start: 40,
        end: 80,
        props: smallvec![PassedProp {
            name: CompactString::new("title"),
            name_is_dynamic: false,
            value: Some(CompactString::new("alpha")),
            start: 45,
            end: 60,
            is_dynamic: true,
        }],
        events: smallvec![EventListener {
            name: CompactString::new("close"),
            name_is_dynamic: false,
            handler: Some(CompactString::new("zeta")),
            modifiers: smallvec![CompactString::new("stop")],
            start: 61,
            end: 70,
        }],
        slots: smallvec![SlotUsage {
            name: CompactString::new("default"),
            name_is_dynamic: false,
            scope_vars: smallvec![],
            start: 71,
            end: 79,
            has_scope: false,
        }],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    croquis.template_expressions.push(TemplateExpression {
        content: CompactString::new("alpha"),
        kind: TemplateExpressionKind::Interpolation,
        start: 81,
        end: 86,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    croquis.provide_inject.add_provide(
        ProvideKey::String(CompactString::new("theme")),
        CompactString::new("alpha"),
        None,
        None,
        90,
        100,
    );
    croquis.provide_inject.add_inject(
        ProvideKey::String(CompactString::new("theme")),
        CompactString::new("theme"),
        None,
        None,
        InjectPattern::ObjectDestructure(vec![CompactString::new("color")]),
        None,
        101,
        120,
    );
    croquis
        .reactivity
        .register(CompactString::new("alpha"), ReactiveKind::Ref, 20);
    croquis.reactivity.add_loss(ReactivityLoss {
        kind: ReactivityLossKind::RefValueExtract {
            source_name: CompactString::new("alpha"),
            target_name: CompactString::new("plain"),
        },
        start: 121,
        end: 130,
    });

    let snapshot = CroquisSemanticSnapshot::from_croquis(&croquis);

    assert_eq!(snapshot.bindings[0].name, "alpha");
    assert_eq!(snapshot.bindings[1].name, "zeta");
    assert_eq!(snapshot.component_usages[0].id, "component:Panel@40");
    assert_eq!(snapshot.provides[0].id, "provide:theme@90");
    assert_eq!(snapshot.injects[0].pattern, "objectDestructure");
    assert_eq!(snapshot.injects[0].destructured_names, ["color"]);
    assert_eq!(snapshot.reactive_sources[0].id, "reactive:alpha@20");
    assert_eq!(snapshot.reactivity_losses[0].kind, "refValueExtract");

    let alpha = snapshot
        .binding_by_name("alpha")
        .expect("binding helper should find alpha");
    assert_eq!(alpha.id, "binding:alpha@20");
    assert!(snapshot.binding_by_name("missing").is_none());
    assert!(snapshot.scope_by_id(ScopeId::ROOT.as_u32()).is_some());

    let panel_usages: Vec<_> = snapshot.component_usages_by_name("Panel").collect();
    assert_eq!(panel_usages.len(), 1);
    assert_eq!(panel_usages[0].props[0].name, "title");

    let root_expressions: Vec<_> = snapshot
        .template_expressions_in_scope(ScopeId::ROOT.as_u32())
        .collect();
    assert_eq!(root_expressions.len(), 1);
    assert_eq!(root_expressions[0].content, "alpha");

    let theme_provides: Vec<_> = snapshot.provides_by_key("theme").collect();
    assert_eq!(theme_provides.len(), 1);
    assert_eq!(theme_provides[0].value, "alpha");

    let theme_injects: Vec<_> = snapshot.injects_by_key("theme").collect();
    assert_eq!(theme_injects.len(), 1);
    assert_eq!(theme_injects[0].destructured_names, ["color"]);

    let reactive_alpha = snapshot
        .reactive_source_by_name("alpha")
        .expect("reactive helper should find alpha");
    assert_eq!(reactive_alpha.kind, "ref");
}
