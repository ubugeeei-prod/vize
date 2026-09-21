
<script setup lang="ts">
import Button from "./Button.vue";

defineArt(Button, {
  title: "Button",
  description: "A button component",
  category: "Components",
  tags: ["button", "ui"],
  status: "draft",
  order: 2,
});
</script>

<art>
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
