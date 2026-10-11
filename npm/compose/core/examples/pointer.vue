<script setup lang="ts">
import { ref, useTemplateRef } from "vue";
import { usePointer } from "@vizejs/composable/pointer";

const pad = useTemplateRef<HTMLElement>("pad");
const visible = ref(true);
const { state, isInside } = usePointer({ target: pad, pointerTypes: ["mouse"] });
</script>

<template>
  <div class="composable-example">
    <label class="pointer-toggle"
      ><input v-model="visible" type="checkbox" /> Show pointer guide pad</label
    >
    <section v-if="visible" ref="pad" class="pointer-pad" aria-label="Pointer guide pad">
      <strong>Inspect an actual mouse pointer</strong>
      <p>Move across this pad, then press and release.</p>
    </section>
    <output aria-live="polite">
      Latest pointer: {{ state.pointerType ?? "none" }} · Inside: {{ isInside ? "yes" : "no" }} ·
      Client: {{ Math.round(state.x) }}, {{ Math.round(state.y) }}
    </output>
    <dl>
      <div>
        <dt>Pressure</dt>
        <dd class="pointer-pressure">{{ state.pressure }}</dd>
      </div>
      <div>
        <dt>Pointer identifier</dt>
        <dd>{{ state.pointerId }}</dd>
      </div>
    </dl>
    <p>
      These readonly values describe the latest accepted event in this iframe's client coordinates,
      not the pad's local coordinates. This example records mouse snapshots only; its checks cover
      mouse input.
    </p>
    <p>
      Leaving the pad changes Inside without clearing the last snapshot. Hiding it removes its
      listeners; the last snapshot remains. Showing it attaches the new pad. Server rendering starts
      with zero values, no pointer kind and Inside: no.
    </p>
  </div>
</template>

<style scoped>
.pointer-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;

  input {
    inline-size: auto;
  }
}
.pointer-pad {
  min-block-size: 120px;
  padding: 12px;
  border: 1px solid currentColor;
  border-radius: 8px;
}
</style>
