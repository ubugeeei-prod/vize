<script setup lang="ts">
import { useId } from "vue";
import { useTextSelection } from "@vizejs/composable/text-selection";

const excerptLabelId = useId();
const { text, ranges, rects } = useTextSelection();

function clearSelection(): void {
  document.getSelection()?.removeAllRanges();
}
</script>

<template>
  <div class="composable-example">
    <p :id="excerptLabelId">Editable guide excerpt</p>
    <div
      class="selection-excerpt"
      contenteditable="true"
      role="textbox"
      aria-multiline="true"
      :aria-labelledby="excerptLabelId"
      tabindex="0"
    >
      Inspect this Vue guide.
    </div>
    <output aria-live="polite">Selected text: {{ text || "none" }}</output>
    <p class="selection-counts">Ranges: {{ ranges.length }} · Rectangles: {{ rects.length }}</p>
    <p class="selection-geometry">
      First rectangle: {{ Math.round(rects[0]?.width ?? 0) }} ×
      {{ Math.round(rects[0]?.height ?? 0) }} CSS pixels
    </p>
    <p class="selection-shape">
      {{
        ranges.length === 0
          ? "No range"
          : ranges[0]?.collapsed
            ? "Collapsed caret"
            : "Selected range"
      }}
    </p>
    <button type="button" @click="clearSelection">Clear text selection</button>
    <p>
      Focus the excerpt and select with Shift and arrow keys or your mouse. Select all and edit it
      to try a different local passage. The readonly text, ranges and native rectangles refresh when
      the document selection changes; editing does not save this excerpt.
    </p>
    <p>
      The browser can mutate the same Selection object in place. An empty selected text can still
      have one collapsed caret range; Clear removes all ranges. This observes the document's
      selection, not input or textarea selectionStart/selectionEnd. Server rendering starts with
      empty text and no ranges; the actual document is read after mounting, and scope disposal
      removes its listener.
    </p>
  </div>
</template>

<style scoped>
.selection-excerpt {
  min-block-size: 80px;
  padding: 12px;
  border: 1px solid currentColor;
  border-radius: 8px;
  overflow-wrap: anywhere;
}
</style>
