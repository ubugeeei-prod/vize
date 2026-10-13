<script setup lang="ts">
import { computed, ref } from "vue";
import { useNumberFormat } from "@vizejs/composable/use-number-format";

const locale = ref("en-US");
const total = ref(1234.5);
const options = computed(() => ({
  locale: locale.value,
  style: "currency" as const,
  currency: "EUR",
}));
const { formatted } = useNumberFormat(total, options);
</script>

<template>
  <div class="composable-example">
    <p>Show an order total in the reader's own number format.</p>
    <label>
      Reader locale
      <select v-model="locale">
        <option value="en-US">English (United States)</option>
        <option value="de-DE">German (Germany)</option>
        <option value="ja-JP">Japanese (Japan)</option>
      </select>
    </label>
    <label>
      Order total in euros
      <input v-model.number="total" type="number" min="0" step="0.01" />
    </label>
    <output aria-live="polite">{{ formatted }}</output>
  </div>
</template>
