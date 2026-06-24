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
