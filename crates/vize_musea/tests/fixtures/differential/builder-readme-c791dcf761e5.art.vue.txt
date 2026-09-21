<!-- Button.art.vue -->
<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
  category: "UI",
  tags: ["button", "form"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Disabled</Button>
  </variant>
</art>
