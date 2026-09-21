
<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
  description: "A versatile button component",
  category: "atoms",
  tags: ["ui", "input"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Primary Button</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>
  <variant name="With Icon">
    <Button variant="primary" icon="plus">Add Item</Button>
  </variant>
</art>

<style scoped>
.art-container {
  padding: 20px;
}
</style>
