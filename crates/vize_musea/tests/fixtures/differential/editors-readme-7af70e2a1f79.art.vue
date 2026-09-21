<!-- Button.art.vue -->
<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
  category: "UI",
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Click me</Button>
  </variant>
</art>
