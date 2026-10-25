/** Real native/API/browser regression input. No metadata is supplied by the fixture. */
export const runtimeComponent = `<script setup lang="ts">
defineProps({
  label: { type: String, default: "true" },
  code: { type: String, default: "007" },
  count: { type: Number, required: true, default: 3 },
  enabled: { type: Boolean, default: true },
  items: { type: Array, default: () => ["a", "日本語"] },
  data: { type: Object, default: () => ({ flag: true, count: 2 }) },
  callback: { type: Function, default: () => "function retained" },
  opaque: { type: Object, default: () => ({ state: Math.max(1, 2) }) },
});
defineEmits(["change", "update:modelValue"]);
</script>
<template>
  <output data-runtime-probe>
    <span data-label>{{ label }}</span><span data-code>{{ code }}</span>
    <span data-count>{{ count }}</span><span data-enabled>{{ enabled }}</span>
    <span data-items>{{ JSON.stringify(items) }}</span><span data-data>{{ JSON.stringify(data) }}</span>
    <span data-callback>{{ callback() }}</span><span data-opaque>{{ opaque.state }}</span>
  </output>
</template>`;

export const runtimeArt = `<script setup lang="ts">
defineArt("./RuntimeProbe.vue", { title: "Runtime Controls" });
</script>
<art><variant name="Default" default args='{"label":"Variant","count":9}'><RuntimeProbe label="Variant" :count="9" /></variant></art>`;

export const expectedRuntimeDefaults = {
  label: "true",
  code: "007",
  count: 3,
  enabled: true,
  items: ["a", "日本語"],
  data: { flag: true, count: 2 },
};
