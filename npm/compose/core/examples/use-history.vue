<script setup lang="ts">
import { ref, useId } from "vue";
import { useHistory } from "@vizejs/composable/use-history";

const title = ref("Draft release notes");
const titleId = useId();
const { canUndo, canRedo, undoCount, redoCount, undo, redo, batch, clear } = useHistory(title, {
  capacity: 10,
});
function publishTitle(): void {
  batch(() => {
    title.value = title.value.trim();
    title.value = "Vize release notes";
  });
}
</script>

<template>
  <div class="composable-example">
    <label :for="titleId">Release title</label>
    <input :id="titleId" v-model="title" />
    <div class="example-actions">
      <button type="button" :disabled="!canUndo" @click="() => undo()">Undo</button>
      <button type="button" :disabled="!canRedo" @click="() => redo()">Redo</button>
      <button type="button" @click="publishTitle">Apply publication title</button>
      <button type="button" @click="() => clear()">Clear history</button>
    </div>
    <output aria-live="polite"
      >{{ undoCount }} undo {{ undoCount === 1 ? "step" : "steps" }} · {{ redoCount }} redo
      {{ redoCount === 1 ? "step" : "steps" }}</output
    >
  </div>
</template>
