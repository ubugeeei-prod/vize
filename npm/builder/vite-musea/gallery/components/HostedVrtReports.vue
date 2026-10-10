<script setup lang="ts">
const emit = defineEmits<{ download: [kind: "html" | "json"] }>();

function downloadHtml() {
  emit("download", "html");
}

function downloadJson() {
  emit("download", "json");
}
</script>

<template>
  <div class="hosted-vrt-reports">
    <button type="button" @click="downloadHtml">Download HTML report</button>
    <button type="button" @click="downloadJson">Download JSON report</button>
  </div>
</template>

<style scoped>
.hosted-vrt-reports {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 1rem;
}
button {
  padding: 0.375rem 0.75rem;
  background: var(--musea-accent);
  border: none;
  border-radius: var(--musea-radius-sm);
  color: var(--musea-accent-contrast);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: background var(--musea-transition);
}
button:hover {
  background: var(--musea-accent-hover);
}
button:focus-visible {
  outline: 2px solid var(--musea-accent);
  outline-offset: 2px;
}
</style>
