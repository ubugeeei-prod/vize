<script setup lang="ts">
import { useId, useTemplateRef } from "vue";
import { useScroll } from "@vizejs/composable/scroll";

const previewId = useId();
const preview = useTemplateRef<HTMLTextAreaElement>("preview");
const guide = [
  "Workshop preparation guide",
  "",
  "1. Install the Vue tooling in your existing project.",
  "Keep the dependency changes separate from application changes.",
  "",
  "2. Compile a representative component.",
  "Check its emitted behavior and its diagnostics.",
  "",
  "3. Review keyboard interaction.",
  "Use the native controls before adding custom shortcuts.",
  "",
  "4. Check a narrow viewport.",
  "Verify that content wraps and controls stay reachable.",
  "",
  "5. Publish the verified result.",
  "Keep the source and the demonstrated behavior aligned.",
].join("\n");
const { y, arrivedState, isScrolling, scrollTo } = useScroll(preview);
</script>

<template>
  <div class="composable-example">
    <label :for="previewId">Preparation guide preview</label>
    <textarea :id="previewId" ref="preview" class="guide-preview" :value="guide" readonly />
    <div class="example-actions">
      <button type="button" :disabled="arrivedState.top" @click="() => (y = y - 120)">
        Previous guide section
      </button>
      <button type="button" :disabled="arrivedState.bottom" @click="() => (y = y + 120)">
        Next guide section
      </button>
      <button type="button" @click="() => scrollTo({ top: 0 })">Return to the beginning</button>
    </div>
    <output aria-live="polite">
      Vertical offset: {{ Math.round(y) }} pixels · At beginning:
      {{ arrivedState.top ? "yes" : "no" }} · At end: {{ arrivedState.bottom ? "yes" : "no" }}
    </output>
    <p>Scrolling activity: {{ isScrolling ? "active" : "idle" }}</p>
    <p>
      Scroll the native read-only preview or use the buttons. Assigning the writable y ref scrolls
      this element; edge flags follow its real dimensions and disable the corresponding controls.
      This preview does not scroll the surrounding documentation page.
    </p>
  </div>
</template>

<style scoped>
.guide-preview {
  box-sizing: border-box;
  inline-size: 100%;
  block-size: 140px;
  resize: none;
  padding: 12px;
  line-height: 1.6;
}
</style>
