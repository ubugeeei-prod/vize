//! Integration tests for `vize musea migrate`.

use std::fs;
use std::process::Command;

fn run_migrate(dir: &std::path::Path, extra: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(dir).args(["musea", "migrate"]);
    command.args(extra);
    command.output().expect("failed to run vize musea migrate")
}

#[test]
fn migrates_render_args_and_unsupported_story() {
    let dir = tempfile::tempdir().unwrap();
    let story = dir.path().join("AfButton.stories.tsx");
    fs::write(
        &story,
        r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = { render: () => <AfButton color="primary">Primary</AfButton> };
export const Secondary: StoryObj = { args: { color: "secondary", label: "Hi" } };
export const Mystery = { decorators: [withFoo] };
"#,
    )
    .unwrap();

    let output = run_migrate(dir.path(), &["AfButton.stories.tsx"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        std::string::String::from_utf8_lossy(&output.stderr)
    );

    let generated = fs::read_to_string(dir.path().join("AfButton.art.vue")).unwrap();
    assert_eq!(
        generated,
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
}

#[test]
fn migrates_nested_render_function_jsx_children() {
    let dir = tempfile::tempdir().unwrap();
    let story = dir.path().join("AfButton.stories.tsx");
    fs::write(
        &story,
        r#"import type { Meta, StoryObj } from "@storybook/vue3";
import AfButton from "./AfButton.vue";
const meta = { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Primary: Story = {
  args: { color: "primary" },
  render: args => () => <AfButton {...args}>Primary</AfButton>,
};
"#,
    )
    .unwrap();

    let output = run_migrate(dir.path(), &["AfButton.stories.tsx"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        std::string::String::from_utf8_lossy(&output.stderr)
    );

    let generated = fs::read_to_string(dir.path().join("AfButton.art.vue")).unwrap();
    assert!(generated.contains(r#"<AfButton color="primary">Primary</AfButton>"#));
    assert!(!generated.contains("TODO(vize musea migrate)"));
}

#[test]
fn migrates_meta_args_or_todos_unsafe_bindings() {
    let dir = tempfile::tempdir().unwrap();
    let safe_story = dir.path().join("SafeButton.stories.tsx");
    fs::write(
        &safe_story,
        r#"import SafeButton from "./SafeButton.vue";
export default { component: SafeButton, title: "Base/SafeButton", args: { color: "primary", disabled: true } } satisfies Meta<typeof SafeButton>;
export const Primary = { args: {} };
export const Secondary = { args: { color: "secondary" } };
"#,
    )
    .unwrap();
    let unsafe_story = dir.path().join("UnsafeCard.stories.tsx");
    fs::write(
        &unsafe_story,
        r#"import UnsafeCard from "./UnsafeCard.vue";
const base = createFixture();
export default { component: UnsafeCard, title: "Base/UnsafeCard", args: { data: base } } satisfies Meta<typeof UnsafeCard>;
export const Primary = { args: {} };
"#,
    )
    .unwrap();

    let output = run_migrate(
        dir.path(),
        &["SafeButton.stories.tsx", "UnsafeCard.stories.tsx"],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        std::string::String::from_utf8_lossy(&output.stderr)
    );

    let safe_generated = fs::read_to_string(dir.path().join("SafeButton.art.vue")).unwrap();
    assert!(safe_generated.contains(r#"<SafeButton color="primary" :disabled="true" />"#));
    assert!(safe_generated.contains(r#"<SafeButton :disabled="true" color="secondary" />"#));

    let unsafe_generated = fs::read_to_string(dir.path().join("UnsafeCard.art.vue")).unwrap();
    assert!(unsafe_generated.contains("TODO(vize musea migrate)"));
    assert!(!unsafe_generated.contains(":data"));
}

#[test]
fn migrates_unsupported_tsx_storyfn_as_todo_variant() {
    let dir = tempfile::tempdir().unwrap();
    let story = dir.path().join("Toggle.stories.tsx");
    fs::write(
        &story,
        r#"import type { Meta, StoryFn } from "@storybook/vue3";
import Toggle from "./Toggle.vue";
export default { component: Toggle, title: "Controls/Toggle" } satisfies Meta<typeof Toggle>;
const Template: StoryFn = (args) => <Toggle {...args} />;
export const Checked = Template.bind({});
Checked.args = { checked: true };
"#,
    )
    .unwrap();

    let output = run_migrate(dir.path(), &["Toggle.stories.tsx"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        std::string::String::from_utf8_lossy(&output.stderr)
    );

    let generated = fs::read_to_string(dir.path().join("Toggle.art.vue")).unwrap();
    assert!(generated.contains(r#"<variant name="Checked" default>"#));
    assert!(generated.contains("TODO(vize musea migrate)"));
}

#[test]
fn migrates_unsupported_render_output_as_todo_variants() {
    let dir = tempfile::tempdir().unwrap();
    let story = dir.path().join("AfsForm.stories.tsx");
    fs::write(
        &story,
        r#"import AfsForm from "./AfsForm.vue";
const loginSchema = createSchema();
export default { component: AfsForm, title: "Forms/AfsForm" } satisfies Meta<typeof AfsForm>;
export const UsesLocalBinding = { render: () => <AfsForm schema={loginSchema} /> };
export const UsesRenderArgs = { render: args => <AfsForm value={args.value} /> };
export const UsesSlotObject = {
  render: () => <AfsForm>{{ default: () => <div>body</div> }}</AfsForm>,
};
"#,
    )
    .unwrap();

    let output = run_migrate(dir.path(), &["AfsForm.stories.tsx"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        std::string::String::from_utf8_lossy(&output.stderr)
    );

    let stderr = std::string::String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("3 variant(s) generated, 3 TODO fallback(s)"));
    let generated = fs::read_to_string(dir.path().join("AfsForm.art.vue")).unwrap();
    assert!(generated.contains(r#"<variant name="UsesLocalBinding" default>"#));
    assert!(generated.contains(r#"<variant name="UsesRenderArgs">"#));
    assert!(generated.contains(r#"<variant name="UsesSlotObject">"#));
    assert!(!generated.contains(":schema=\"loginSchema\""));
    assert!(!generated.contains(":value=\"args.value\""));
    assert!(!generated.contains("=> <div>"));
}

#[test]
fn dry_run_prints_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let story = dir.path().join("Box.stories.tsx");
    fs::write(
        &story,
        r#"import Box from "./Box.vue";
export default { component: Box, title: "Box" } as Meta;
export const First = { name: "Custom Name", render: () => <Box a="x" /> };
"#,
    )
    .unwrap();

    let output = run_migrate(dir.path(), &["--dry-run", "Box.stories.tsx"]);
    assert!(output.status.success());

    let stdout = std::string::String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout,
        r#"// Box.art.vue
<script setup lang="ts">
defineArt("./Box.vue", {
  title: "Box",
});
</script>

<art>
  <variant name="Custom Name" default>
    <Box a="x" />
  </variant>
</art>
"#
    );

    assert!(
        !dir.path().join("Box.art.vue").exists(),
        "--dry-run must not write files"
    );
}

#[test]
fn out_dir_redirects_generated_files() {
    let dir = tempfile::tempdir().unwrap();
    let story = dir.path().join("Plain.stories.ts");
    fs::write(
        &story,
        r#"import Plain from "./Plain.vue";
export default { component: Plain, title: "Group/Plain" } satisfies Meta<typeof Plain>;
export const Big = { args: { size: "lg", count: 3 } };
"#,
    )
    .unwrap();

    let output = run_migrate(dir.path(), &["--out-dir", "art", "Plain.stories.ts"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        std::string::String::from_utf8_lossy(&output.stderr)
    );

    assert!(!dir.path().join("Plain.art.vue").exists());
    let generated = fs::read_to_string(dir.path().join("art/Plain.art.vue")).unwrap();
    assert_eq!(
        generated,
        r#"<script setup lang="ts">
defineArt("./Plain.vue", {
  category: "Group",
  title: "Plain",
});
</script>

<art>
  <variant name="Big" default>
    <Plain size="lg" :count="3" />
  </variant>
</art>
"#
    );
}
