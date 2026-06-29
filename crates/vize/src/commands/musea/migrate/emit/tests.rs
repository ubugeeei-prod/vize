use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::super::csf::extract_csf;
use super::emit_art;

fn emit(source: &str) -> (std::string::String, usize, usize) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    assert!(!parsed.panicked, "fixture should parse");
    let module = extract_csf(&parsed.program);
    let component_path = module
        .component_path
        .clone()
        .unwrap_or_else(|| "./Component.vue".into());
    let result = emit_art(&module, "AfButton", component_path.as_str(), source);
    (
        result.content.as_str().to_owned(),
        result.variants,
        result.todos,
    )
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
fn emits_todo_for_args_referencing_local_fixture_identifiers() {
    let source = r#"import AfButton from "./AfButton.vue";
const fixture = { label: "Hi" };
export default { component: AfButton, title: "AfButton" } satisfies Meta<typeof AfButton>;
export const Big = { args: { data: fixture } };
"#;
    let (content, variants, todos) = emit(source);

    assert!(content.contains("<AfButton />"));
    assert!(content.contains("TODO(vize musea migrate)"));
    assert!(!content.contains(":data=\"fixture\""));
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
