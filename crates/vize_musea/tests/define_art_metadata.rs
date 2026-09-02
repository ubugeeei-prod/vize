use vize_musea::{ArtParseOptions, ArtStatus, parse_art};
use vize_s0::Allocator;

fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    filename: &str,
) -> vize_musea::ArtDescriptor<'a> {
    parse_art(
        allocator,
        source,
        ArtParseOptions {
            filename: filename.into(),
        },
    )
    .unwrap()
}

#[test]
fn parse_define_art_metadata_matrix() {
    let allocator = Allocator::new();
    let source = r#"
<script>
export const localKind = "mixed";
</script>

<script setup>
import { default as AliasButton } from "./AliasButton.vue";

defineArt(AliasButton, {
  title: "Alias Button",
  category: "Components",
  tags: ["alias", localKind],
  status: "ready",
});
</script>

<art>
  <variant name="Primary" default>
    <AliasButton>Primary</AliasButton>
  </variant>
</art>
"#;

    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();

    assert_eq!(desc.metadata.title, "Alias Button");
    assert_eq!(desc.metadata.component, Some("./AliasButton.vue"));
    assert_eq!(desc.metadata.category, Some("Components"));
    assert_eq!(desc.metadata.tags.as_slice(), ["alias"]);
    assert_eq!(desc.metadata.status, ArtStatus::Ready);
}

#[test]
fn unknown_define_art_status_falls_back_to_draft() {
    let allocator = Allocator::new();
    let source = r#"
<script setup>
import Button from "./Button.vue";

defineArt(Button, {
  title: "Button",
  status: "wip",
});
</script>

<art>
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
"#;

    let desc = parse(&allocator, source, "button.art.vue");
    assert_eq!(desc.metadata.status, ArtStatus::Draft);
}

#[test]
fn omitted_define_art_status_stays_ready() {
    let allocator = Allocator::new();
    let source = r#"
<script setup>
import Button from "./Button.vue";

defineArt(Button, {
  title: "Button",
});
</script>

<art>
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
"#;

    let desc = parse(&allocator, source, "button.art.vue");
    assert_eq!(desc.metadata.status, ArtStatus::Ready);
}

#[test]
fn unknown_art_status_attr_falls_back_to_draft() {
    let allocator = Allocator::new();
    let source = r#"
<art title="Button" status="wip">
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
"#;

    let desc = parse(&allocator, source, "button.art.vue");
    assert_eq!(desc.metadata.status, ArtStatus::Draft);
}

#[test]
fn unknown_status_without_filename_uses_anonymous() {
    let allocator = Allocator::new();
    let source = r#"
<script setup>
defineArt("./Button.vue", { title: "Button", status: "wip" });
</script>
<art>
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
"#;

    let desc = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
    assert_eq!(desc.metadata.status, ArtStatus::Draft);
}
