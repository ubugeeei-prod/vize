<script setup lang="ts">
import { ref, useId } from "vue";
import { useMediaQuery } from "@vizejs/composable/media-query";

const breakpointId = useId();
const breakpoint = ref(600);
const roomy = useMediaQuery(() => `(min-width: ${breakpoint.value}px)`, { ssrValue: false });
</script>

<template>
  <div class="composable-example">
    <label :for="breakpointId">Roomy layout starts at</label>
    <select
      :id="breakpointId"
      v-model.number="breakpoint"
      :aria-describedby="`${breakpointId}-help`"
    >
      <option :value="600">600 pixels</option>
      <option :value="800">800 pixels</option>
      <option :value="1000">1000 pixels</option>
    </select>
    <p :id="`${breakpointId}-help`">
      Resize this example's window or change the threshold. In the embedded preview, the query uses
      the preview frame's width.
    </p>
    <output aria-live="polite">Layout: {{ roomy ? "Roomy" : "Compact" }}</output>
    <p>Live query: (min-width: {{ breakpoint }}px)</p>
    <ul
      aria-label="Recommended guides"
      :style="{
        display: 'grid',
        gridTemplateColumns: roomy ? 'repeat(2, minmax(0, 1fr))' : 'minmax(0, 1fr)',
        gap: '16px',
        padding: '0',
        listStyle: 'none',
      }"
    >
      <li>
        <h3>Write components</h3>
        <p>Start with a template and add reactive behavior.</p>
      </li>
      <li>
        <h3>Check a project</h3>
        <p>Use diagnostics to catch mistakes before publishing.</p>
      </li>
    </ul>
    <p>The server renders the compact layout until the browser can evaluate the query.</p>
  </div>
</template>
