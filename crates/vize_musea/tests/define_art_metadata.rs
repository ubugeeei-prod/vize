use vize_carton::Bump;
use vize_musea::{ArtParseOptions, ArtStatus, parse_art};

#[test]
fn parse_define_art_metadata_matrix() {
    let allocator = Bump::new();
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
