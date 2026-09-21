<script setup lang="ts">
defineArt("./AfButton.vue", {
  category: "Base",
  title: "AfButton",
});
</script>

<art>
  <variant name="Primary" default>
    <AfButton color="primary">Primary</AfButton>
  </variant>
  <variant name="Secondary">
    <AfButton color="secondary" label="Hi" />
  </variant>
  <variant name="Mystery">
    <AfButton />
    <!-- TODO(vize musea migrate): unsupported story; port manually -->
  </variant>
</art>
