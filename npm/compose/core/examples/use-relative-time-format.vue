<script setup lang="ts">
import { computed, ref } from "vue";
import { useRelativeTimeFormat } from "@vizejs/composable/use-relative-time-format";

const locale = ref("en-US");
const days = ref(3);
const options = computed(() => ({ locale: locale.value, numeric: "auto" as const }));
const { formatted } = useRelativeTimeFormat(days, "day", options);
</script>

<template>
  <div class="composable-example">
    <p>Say when an event happens in words the reader would use.</p>
    <label>
      Reader locale
      <select v-model="locale">
        <option value="en-US">English (United States)</option>
        <option value="de-DE">German (Germany)</option>
        <option value="ja-JP">Japanese (Japan)</option>
      </select>
    </label>
    <label>
      Days from today
      <input v-model.number="days" type="number" min="-30" max="30" step="1" />
    </label>
    <output aria-live="polite">The meetup is {{ formatted }}.</output>
  </div>
</template>
