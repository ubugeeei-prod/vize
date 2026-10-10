<script setup lang="ts">
import { ref, useId } from "vue";
import { useSelection } from "@vizejs/composable/use-selection";

const selectionId = useId();
const revised = ref(false);
const sessions = ref([
  { id: "compiler", title: "Compiler clinic", full: false },
  { id: "types", title: "Type checking clinic", full: false },
  { id: "ssr", title: "SSR workshop (full)", full: true },
  { id: "editor", title: "Editor tools clinic", full: false },
]);
const { selected, count, isSelected, toggle, selectAll, clear } = useSelection({
  items: sessions,
  multiple: true,
  max: 2,
  getKey: (item) => item.id,
  isSelectable: (item) => !item.full,
});

function reviseCompilerTitle(): void {
  sessions.value = sessions.value.map((item) => ({
    ...item,
    title: item.id === "compiler" ? "Compiler clinic (updated)" : item.title,
  }));
  revised.value = true;
}
</script>

<template>
  <div class="composable-example">
    <p>
      Choose up to two sessions from this local catalogue. The full workshop cannot be selected.
    </p>
    <fieldset>
      <legend>Workshop sessions</legend>
      <div v-for="item in sessions" :key="item.id">
        <input
          :id="`${selectionId}-${item.id}`"
          type="checkbox"
          class="session-checkbox"
          :checked="isSelected(item)"
          :disabled="item.full || (count >= 2 && !isSelected(item))"
          @change="() => toggle(item)"
        />
        <label :for="`${selectionId}-${item.id}`" class="session-label">{{ item.title }}</label>
      </div>
    </fieldset>
    <output aria-live="polite">{{ count }} of 2 places used</output>
    <p v-if="count === 2">Your two-session limit is reached. Clear a choice to select another.</p>
    <p v-if="count === 0">No sessions selected.</p>
    <ul v-else aria-label="Selected sessions">
      <li v-for="item in selected" :key="item.id">{{ item.title }}</li>
    </ul>
    <div class="example-actions">
      <button type="button" @click="selectAll">Select available sessions</button>
      <button type="button" :disabled="count === 0" @click="clear">Clear session choices</button>
      <button type="button" :disabled="revised" @click="reviseCompilerTitle">
        Revise compiler title
      </button>
    </div>
    <p>
      Revising the title replaces the catalogue's objects. Selected sessions keep their stable ids.
    </p>
  </div>
</template>

<style scoped>
.session-checkbox {
  inline-size: auto;
}
.session-label {
  display: inline;
}
</style>
