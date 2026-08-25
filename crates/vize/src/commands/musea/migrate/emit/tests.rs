use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_s0::String;

use super::super::csf::extract_csf;
use super::emit_art;

fn emit(source: &str) -> (String, usize, usize) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    assert!(!parsed.panicked, "fixture should parse");
    let module = extract_csf(&parsed.program);
    let component_path = module
        .component_path
        .clone()
        .unwrap_or_else(|| "./Component.vue".into());
    let result = emit_art(&module, "AfButton", component_path.as_str(), source);
    (result.content, result.variants, result.todos)
}

#[test]
fn emits_render_and_args_and_todo() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = { render: () => <AfButton color="primary">Primary</AfButton> };
export const Secondary: StoryObj = { args: { color: "secondary", label: "Hi" } };
export const Mystery = { decorators: [withFoo] };
"#;
    let (content, variants, todos) = emit(source);

    assert_eq!(
        content,
        r#"<script setup lang="ts">
defineArt("./AfButton.vue", {
  category: "Base",
  title: "AfButton",
});
</script>

<art>
  <variant name="Primary" default>
    <AfButton color="primary">Primary</AfButton>
  </variant>
  <variant name="Secondary">
    <AfButton color="secondary" label="Hi" />
  </variant>
  <variant name="Mystery">
    <AfButton />
    <!-- TODO(vize musea migrate): unsupported story; port manually -->
  </variant>
</art>
"#
    );
    assert_eq!(variants, 3);
    assert_eq!(todos, 1);
}

#[test]
fn emits_nested_render_children_with_story_args() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = {
  args: { color: "primary" },
  render: args => () => <AfButton {...args}>Primary</AfButton>,
};
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<AfButton color="primary">Primary</AfButton>"#));
    assert_eq!(variants, 1);
    assert_eq!(todos, 0);
}

#[test]
fn emits_template_bind_storyfn_exports_with_assigned_args() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
const Template: StoryFn = (args) => <AfButton {...args} />;
export const Primary = Template.bind({});
Primary.args = { color: "primary" };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<variant name="Primary" default>"#));
    assert!(content.contains(r#"<AfButton color="primary" />"#));
    assert!(!content.contains("TODO(vize musea migrate)"));
    assert_eq!(variants, 1);
    assert_eq!(todos, 0);
}

#[test]
fn emits_todo_when_render_object_would_drop_state_or_slots() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export const Stateful = {
  args: { color: "primary" },
  render: args => ({
    setup() {
      const open = ref(false);
      return { args, open };
    },
    template: '<AfButton v-bind="args"><span>Primary</span></AfButton>',
  }),
};
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<variant name="Stateful" default>"#));
    assert!(content.contains("TODO(vize musea migrate)"));
    assert!(!content.contains(r#"<AfButton color="primary" />"#));
    assert_eq!(variants, 1);
    assert_eq!(todos, 1);
}

#[test]
fn emits_todo_for_render_output_requiring_story_bindings() {
    let source = r#"import AfButton from "./AfButton.vue";
const schema = createSchema();
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export const LocalBinding = { render: () => <AfButton data={schema} /> };
export const ArgsMember = { render: args => <AfButton value={args.value} /> };
export const SlotObject = {
  render: () => <AfButton>{{ default: () => <span>Primary</span> }}</AfButton>,
};
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<variant name="LocalBinding" default>"#));
    assert!(content.contains(r#"<variant name="ArgsMember">"#));
    assert!(content.contains(r#"<variant name="SlotObject">"#));
    assert!(!content.contains(":data=\"schema\""));
    assert!(!content.contains(":value=\"args.value\""));
    assert!(!content.contains("=> <span>"));
    assert_eq!(variants, 3);
    assert_eq!(todos, 3);
}

#[test]
fn emits_plain_title_without_category() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Big = { args: { size: "lg", count: 3, active: true } };
"#;
    let (content, variants, todos) = emit(source);

    assert_eq!(
        content,
        r#"<script setup lang="ts">
defineArt("./AfButton.vue", {
  title: "AfButton",
});
</script>

<art>
  <variant name="Big" default>
    <AfButton size="lg" :count="3" :active="true" />
  </variant>
</art>
"#
    );
    assert_eq!(variants, 1);
    assert_eq!(todos, 0);
}

#[test]
fn inlines_static_local_fixture_args() {
    let source = r#"import AfButton from "./AfButton.vue";
const fixture = { label: "Hi" };
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Big = { args: { data: fixture } };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<AfButton :data='{ label: "Hi" }' />"#));
    assert!(!content.contains("TODO(vize musea migrate)"));
    assert!(!content.contains(":data=\"fixture\""));
    assert_eq!(variants, 1);
    assert_eq!(todos, 0);
}

#[test]
fn skips_exported_fixture_object_variants() {
    let source = r#"import AfButton from "./AfButton.vue";
export const fixture = { label: "Hi" };
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = { args: { data: fixture } };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<variant name="Primary" default>"#));
    assert!(!content.contains(r#"<variant name="fixture""#));
    assert!(content.contains(r#"<AfButton :data='{ label: "Hi" }' />"#));
    assert_eq!(variants, 1);
    assert_eq!(todos, 0);
}

#[test]
fn emits_meta_args_and_lets_story_args_override() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "AfButton", args: { color: "primary", disabled: true, count: 1 } } satisfies Meta<typeof AfButton>;
export const Primary = { args: {} };
export const Secondary = { args: { color: "secondary", label: "Hi" } };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<AfButton color="primary" :disabled="true" :count="1" />"#));
    assert!(
        content
            .contains(r#"<AfButton :disabled="true" :count="1" color="secondary" label="Hi" />"#)
    );
    assert!(!content.contains(r#"color="primary" :disabled="true" :count="1" color="secondary""#));
    assert_eq!(variants, 2);
    assert_eq!(todos, 0);
}

#[test]
fn emits_meta_args_inside_render_args_spread() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "AfButton", args: { color: "primary", disabled: true } } satisfies Meta<typeof AfButton>;
export const Primary = {
  args: { color: "secondary" },
  render: args => () => <AfButton {...args}>Primary</AfButton>,
};
"#;
    let (content, _variants, todos) = emit(source);

    assert!(content.contains(r#"<AfButton :disabled="true" color="secondary">Primary</AfButton>"#));
    assert_eq!(todos, 0);
}

#[test]
fn emits_story_args_inside_renamed_render_param_spread() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = {
  args: { color: "primary", disabled: true },
  render: props => <AfButton {...props}>Primary</AfButton>,
};
"#;
    let (content, _variants, todos) = emit(source);

    assert!(content.contains(r#"<AfButton color="primary" :disabled="true">Primary</AfButton>"#));
    assert!(!content.contains("v-bind=\"props\""));
    assert_eq!(todos, 0);
}

#[test]
fn emits_todo_for_meta_args_referencing_module_bindings() {
    let source = r#"import AfButton from "./AfButton.vue";
const base = createFixture();
export default { component: AfButton, title: "AfButton", args: { data: base } } satisfies Meta<typeof AfButton>;
export const Primary = { args: {} };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains("<AfButton />"));
    assert!(content.contains("TODO(vize musea migrate)"));
    assert!(!content.contains(":data"));
    assert_eq!(variants, 1);
    assert_eq!(todos, 1);
}

#[test]
fn inlines_static_meta_args_module_bindings() {
    let source = r#"import AfButton from "./AfButton.vue";
const base = { label: "Hi" } as const;
export default { component: AfButton, title: "AfButton", args: { data: base } } satisfies Meta<typeof AfButton>;
export const Primary = { args: {} };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<AfButton :data='{ label: "Hi" } as const' />"#));
    assert!(!content.contains("TODO(vize musea migrate)"));
    assert_eq!(variants, 1);
    assert_eq!(todos, 0);
}

#[test]
fn emits_todo_for_static_args_with_nested_module_bindings() {
    let source = r#"import AfButton from "./AfButton.vue";
const base = { label: "Hi" };
const fixture = { data: base };
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = { args: { data: fixture } };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains("<AfButton />"));
    assert!(content.contains("TODO(vize musea migrate)"));
    assert!(!content.contains(":data"));
    assert_eq!(variants, 1);
    assert_eq!(todos, 1);
}

#[test]
fn emits_todo_for_nested_story_args_module_bindings() {
    let source = r#"import AfButton from "./AfButton.vue";
const base = createFixture();
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = { args: { data: { ...base, status: Status.Ready } } };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains("<AfButton />"));
    assert!(content.contains("TODO(vize musea migrate)"));
    assert!(!content.contains(":data"));
    assert_eq!(variants, 1);
    assert_eq!(todos, 1);
}

#[test]
fn emits_todo_for_unresolved_story_spread() {
    let source = r#"import AfButton from "./AfButton.vue";
import { Primary as BaseStory } from "./Base.stories";
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Imported = { ...BaseStory };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains(r#"<variant name="Imported" default>"#));
    assert!(content.contains("TODO(vize musea migrate)"));
    assert_eq!(variants, 1);
    assert_eq!(todos, 1);
}

#[test]
fn emits_directive_expressions_without_quot_entities() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Big = { args: { to: { name: "students" } as NuxtRoute<"students", string>, field: { label: "Name" } } };
"#;
    let (content, _variants, todos) = emit(source);

    assert!(content.contains(r#":to='{ name: "students" } as NuxtRoute<"students", string>'"#));
    assert!(content.contains(r#":field='{ label: "Name" }'"#));
    assert!(!content.contains("&quot;"));
    assert_eq!(todos, 0);
}
