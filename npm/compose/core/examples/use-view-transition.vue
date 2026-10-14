<script setup lang="ts">
import { nextTick, ref } from "vue";
import { useViewTransition } from "@vizejs/composable/use-view-transition";

const layout = ref<"grid" | "list">("grid");
const { start } = useViewTransition();

function toggle() {
  void start(async () => {
    layout.value = layout.value === "grid" ? "list" : "grid";
    await nextTick();
  });
}
</script>

<template>
  <div class="composable-example">
    <p>Animate a layout change with a view transition where the browser supports it.</p>
    <button type="button" @click="toggle">Show as {{ layout === "grid" ? "list" : "grid" }}</button>
    <ul
      aria-label="Talks"
      :style="{
        display: layout === 'grid' ? 'grid' : 'block',
        gridTemplateColumns: 'repeat(2, 1fr)',
      }"
    >
      <li>Vapor mode</li>
      <li>Type checking</li>
      <li>Linting</li>
      <li>Formatting</li>
    </ul>
    <output aria-live="polite">Layout: {{ layout }}</output>
  </div>
</template>
