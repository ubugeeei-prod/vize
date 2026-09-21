<script setup lang="ts">
defineArt("./AfButton.vue", {
  title: "AfButton",
});
</script>

<art>
  <variant name="Big" default>
    <AfButton size="lg" :count="3" :active="true" />
  </variant>
</art>
