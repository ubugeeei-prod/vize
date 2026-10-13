<script setup lang="ts">
import { computed, ref, useId, useTemplateRef } from "vue";
import { onStartTyping } from "@vizejs/composable/on-start-typing";

const searchId = useId();
const search = useTemplateRef<HTMLInputElement>("search");
const query = ref("");
const note = ref("");
const deliveries = ref(0);
const readingActions = ref(0);
const stopped = ref(false);
const guides = ["Compiler guide", "Linter guide", "Keyboard guide"];
const matches = computed(() =>
  guides.filter((guide) => guide.toLowerCase().includes(query.value.toLowerCase())),
);
const stop = onStartTyping(() => {
  deliveries.value += 1;
  search.value?.focus();
});

function stopShortcut(): void {
  stop();
  stopped.value = true;
}
</script>

<template>
  <div class="composable-example">
    <button type="button" @click="() => (readingActions += 1)">Reading control</button>
    <p>Native reading activations: {{ readingActions }}</p>
    <label :for="`${searchId}-query`">Search local guides</label>
    <input :id="`${searchId}-query`" ref="search" v-model="query" type="search" />
    <ul aria-label="Matching local guides">
      <li v-for="guide in matches" :key="guide">{{ guide }}</li>
    </ul>
    <p v-if="matches.length === 0">No local guides match.</p>
    <label :for="`${searchId}-note`">Workshop note</label>
    <input :id="`${searchId}-note`" v-model="note" />
    <button type="button" :disabled="stopped" @click="stopShortcut">Stop typing shortcut</button>
    <output aria-live="polite"
      >Typing shortcut deliveries: {{ deliveries }} · Active: {{ stopped ? "no" : "yes" }}</output
    >
    <p>
      Focus Reading control and press a printable character to focus the search. Space, navigation
      keys, and Ctrl/Meta/Alt shortcuts do not redirect focus. Editing either field stays ordinary
      native typing. Stop removes the shortcut; the fields and local filtering remain usable. This
      preview sends no search request or note save.
    </p>
  </div>
</template>
