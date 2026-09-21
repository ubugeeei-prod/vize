<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
  category: "Components",
  tags: ["button", "form"],
  status: "ready",
});
</script>

<art>
  <variant name="Default" default>
    <Button>Default Button</Button>
  </variant>
  <variant name="Primary">
    <Button variant="primary">Primary Button</Button>
  </variant>
</art>
