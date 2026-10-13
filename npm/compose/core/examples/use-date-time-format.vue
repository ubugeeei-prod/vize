<script setup lang="ts">
import { computed, ref } from "vue";
import { useDateTimeFormat } from "@vizejs/composable/use-date-time-format";

const locale = ref("en-US");
const date = new Date(Date.UTC(2026, 9, 24));
const { formatted } = useDateTimeFormat(
  date,
  computed(() => ({ locale: locale.value, dateStyle: "long" as const, timeZone: "UTC" })),
);
</script>

<template>
  <div class="composable-example">
    <p>Write a conference date the way each reader writes dates.</p>
    <label>
      Reader locale
      <select v-model="locale">
        <option value="en-US">English (United States)</option>
        <option value="de-DE">German (Germany)</option>
        <option value="ja-JP">Japanese (Japan)</option>
      </select>
    </label>
    <output aria-live="polite">{{ formatted }}</output>
  </div>
</template>
