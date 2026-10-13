<script setup lang="ts">
import { onMounted, ref, useTemplateRef } from "vue";
import { useDraggable } from "@vizejs/composable/draggable";
import type { DragAxis } from "@vizejs/composable/draggable";

const board = useTemplateRef<HTMLElement>("board");
const note = useTemplateRef<HTMLElement>("note");
const handle = useTemplateRef<HTMLButtonElement>("handle");
const axis = ref<DragAxis>("both");
const locked = ref(false);
const stopped = ref(false);
const draft = ref("Review the native preview");
const starts = ref(0);
const ends = ref(0);
const { x, y, style, isDragging, stop } = useDraggable(note, {
  handle,
  bounds: board,
  axis,
  disabled: locked,
  preventDefault: true,
  initialValue: { x: 24, y: 160 },
  onStart: () => {
    starts.value += 1;
  },
  onEnd: () => {
    ends.value += 1;
  },
});

function placeNote(dx: number, dy: number, reset = false): void {
  const bounds = board.value?.getBoundingClientRect();
  const size = note.value?.getBoundingClientRect();
  if (!bounds || !size) return;
  x.value = Math.min(
    Math.max(bounds.left, reset ? bounds.left + 12 : x.value + dx),
    Math.max(bounds.left, bounds.right - size.width),
  );
  y.value = Math.min(
    Math.max(bounds.top, reset ? bounds.top + 12 : y.value + dy),
    Math.max(bounds.top, bounds.bottom - size.height),
  );
}

function stopDragging(): void {
  stop();
  stopped.value = true;
}

onMounted(() => placeNote(0, 0, true));
</script>

<template>
  <div class="composable-example">
    <div class="drag-controls">
      <label>Note text <input v-model="draft" /></label>
      <label>
        Drag axis
        <select v-model="axis">
          <option value="both">Both axes</option>
          <option value="x">X only</option>
          <option value="y">Y only</option>
        </select>
      </label>
      <label class="drag-lock"
        ><input v-model="locked" type="checkbox" /> Lock pointer dragging</label
      >
    </div>
    <section ref="board" class="drag-board" aria-label="Bounded note board">
      <article ref="note" class="drag-note" :style aria-label="Draggable review note">
        <button ref="handle" type="button" aria-label="Drag review note">Drag note</button>
        <span>{{ draft }}</span>
      </article>
    </section>
    <div class="example-actions">
      <button type="button" @click="() => placeNote(24, 0)">Move note right</button>
      <button type="button" @click="() => placeNote(0, 24)">Move note down</button>
      <button type="button" @click="() => placeNote(0, 0, true)">Reset note position</button>
      <button type="button" :disabled="stopped" @click="stopDragging">Stop drag listeners</button>
    </div>
    <output aria-live="polite">
      Position: {{ Math.round(x) }}, {{ Math.round(y) }} · Dragging:
      {{ isDragging ? "yes" : "no" }} · Starts: {{ starts }} · Ends: {{ ends }}
    </output>
    <details>
      <summary>How pointer placement works</summary>
      <p>
        Drag the handle; moves continue on the owner document and keep the whole note inside the
        board. Axis and Lock constrain pointer dragging. The keyboard buttons place the note
        independently and explicitly apply the same bounds, including after listeners stop.
      </p>
      <p>
        Position uses this iframe's client coordinates and fixed positioning. Server rendering uses
        the chosen 24, 160 fallback; mounting places it inside the actual board. The local draft
        survives movement. Bounds are read during pointer moves; Reset places the note again after
        scrolling or layout changes. Nothing is saved or sent.
      </p>
    </details>
  </div>
</template>

<style scoped>
.drag-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: end;

  label {
    display: grid;
    gap: 4px;
    flex: 1 1 120px;
    min-inline-size: 0;
  }

  .drag-lock {
    display: flex;
    align-items: center;
    flex-basis: 180px;

    input {
      inline-size: auto;
    }
  }
}
.drag-board {
  block-size: 160px;
  border: 1px dashed currentColor;
}
.drag-note {
  --note-surface: Canvas;
  --note-layer: 1;
  position: fixed;
  z-index: var(--note-layer);
  box-sizing: border-box;
  inline-size: 160px;
  block-size: 120px;
  padding: 8px;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 8px;
  border: 1px solid currentColor;
  border-radius: 8px;
  background: var(--note-surface);
  overflow-wrap: anywhere;

  button {
    touch-action: none;
    cursor: grab;
  }

  span {
    min-block-size: 0;
    overflow: auto;
  }
}
</style>
