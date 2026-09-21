<script lang="ts">
export const localKind = "mixed";
</script>

<script setup lang="ts">
import { buttonPath as componentPath, metadata } from "./art-meta";

defineArt(componentPath, {
  title: metadata.title,
  category: "Components",
  tags: ["button", localKind],
});

defineArt("./AliasButton.secondary.vue", {
  title: "AliasButtonSecondary",
});
</script>

<art>
  <variant name="Primary" default>
    <AliasButton>Primary</AliasButton>
  </variant>
</art>
