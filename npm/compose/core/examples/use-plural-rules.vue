<script setup lang="ts">
import { ref } from "vue";
import { usePluralRules } from "@vizejs/composable/use-plural-rules";

const count = ref(0);
const { message, category } = usePluralRules(
  count,
  {
    "=0": "No unread messages",
    one: (_count, formatted) => `${formatted} unread message`,
    other: (_count, formatted) => `${formatted} unread messages`,
  },
  { locale: "en-US" },
);
</script>

<template>
  <div class="composable-example">
    <p>Pick the right wording for an unread-message count.</p>
    <label>
      Unread messages
      <input v-model.number="count" type="number" min="0" step="1" />
    </label>
    <output aria-live="polite">{{ message }}</output>
    <p>
      Plural category: <code>{{ category }}</code>
    </p>
  </div>
</template>
