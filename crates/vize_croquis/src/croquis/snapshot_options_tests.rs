use crate::{Drawer, DrawerOptions};

#[test]
fn semantic_snapshot_preserves_options_api_and_nested_template_contracts() {
    use vize_armature::parse;
    use vize_carton::Allocator;

    let script = r#"
import Panel from './Panel.vue'

export default {
  components: { Panel },
  props: { title: String },
  inject: ['theme'],
  data() {
    return {
      count: 0,
      ready: true,
      items: [{ id: 1 }],
    }
  },
  computed: {
    doubled() {
      return this.count * 2
    },
  },
  methods: {
    format(item) {
      return `${this.title}: ${item.id}`
    },
  },
}
"#;
    let template = r#"<Panel
  v-for="item in items"
  v-if="ready && item"
  :key="item.id"
  :title="format(item)"
  :aria-label="row"
  v-slot="{ row }"
>
  {{ title }} {{ count }} {{ doubled }} {{ theme }} {{ row }}
</Panel>"#;

    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "template should parse without errors");

    let mut drawer = Drawer::with_options(DrawerOptions::full()).with_options_api();
    drawer.draw_script_plain(script);
    drawer.draw_template(&root);
    let croquis = drawer.finish();
    assert_eq!(
        croquis
            .undefined_refs
            .iter()
            .map(|reference| reference.name.as_str())
            .collect::<Vec<_>>(),
        ["item", "row"]
    );
    let snapshot = croquis.semantic_snapshot();

    for (name, kind, category) in [
        ("title", "props", "props"),
        ("theme", "options", "options"),
        ("count", "data", "data"),
        ("ready", "data", "data"),
        ("items", "data", "data"),
        ("doubled", "options", "options"),
        ("format", "options", "options"),
    ] {
        let binding = snapshot
            .binding_by_name(name)
            .unwrap_or_else(|| panic!("{name} binding should be exposed"));
        assert_eq!(binding.kind, kind, "binding kind for {name}");
        assert_eq!(binding.category, category, "binding category for {name}");
    }

    let panel = snapshot
        .component_usages_by_name("Panel")
        .next()
        .expect("Panel usage should be exposed");
    assert_eq!(panel.props.len(), 3);
    assert!(panel.props.iter().any(|prop| prop.name == "title"));
    assert!(panel.slots.iter().any(|slot| {
        slot.name == "default" && slot.scoped && slot.scope_vars.iter().any(|name| name == "row")
    }));

    let for_scope = snapshot
        .scopes
        .iter()
        .find(|scope| scope.kind == "v-for")
        .expect("v-for scope should be exposed");
    assert!(
        for_scope
            .bindings
            .iter()
            .any(|binding| binding.name == "item")
    );
    let slot_scope = snapshot
        .scopes
        .iter()
        .find(|scope| scope.kind == "v-slot")
        .expect("v-slot scope should be exposed");
    assert!(
        slot_scope
            .bindings
            .iter()
            .any(|binding| binding.name == "row")
    );
    assert!(slot_scope.parent_ids.contains(&for_scope.id));
    assert_eq!(panel.scope_id, for_scope.id);

    let vif = snapshot
        .template_expressions
        .iter()
        .find(|expression| expression.kind == "vIf" && expression.content == "ready && item")
        .expect("v-if expression should be exposed");
    assert_ne!(vif.scope_id, for_scope.id);
    assert_ne!(vif.scope_id, slot_scope.id);
    assert_eq!(vif.vif_guard.as_deref(), Some("(ready && item)"));
    assert!(
        snapshot.template_expressions.iter().any(|expression| {
            expression.kind == "vBind"
                && expression.content == "format(item)"
                && expression.scope_id == for_scope.id
        }),
        "template expressions: {:#?}",
        snapshot.template_expressions
    );
    assert!(snapshot.template_expressions.iter().any(|expression| {
        expression.kind == "interpolation"
            && expression.content == "row"
            && expression.scope_id == slot_scope.id
    }));
}
