<script setup lang="ts">
defineArt("../src/Button.vue", {
  title: "Button",
  category: "Components",
  tags: ["button", "ui"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>

  <variant name="Secondary">
    <Button variant="secondary">Click me</Button>
  </variant>

  <variant name="Disabled">
    <Button variant="primary" disabled>Disabled</Button>
  </variant>
</art>

<style scoped>
.art-preview {
  padding: 0.5rem 1rem;
  display: flex;
  gap: 0.75rem;
  align-items: center;
}
</style>
