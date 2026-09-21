
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
