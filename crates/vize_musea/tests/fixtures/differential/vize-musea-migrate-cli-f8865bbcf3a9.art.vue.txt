<script setup lang="ts">
defineArt("./Plain.vue", {
  category: "Group",
  title: "Plain",
});
</script>

<art>
  <variant name="Big" default>
    <Plain size="lg" :count="3" />
  </variant>
</art>
