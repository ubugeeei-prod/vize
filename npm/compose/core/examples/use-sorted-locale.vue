<script setup lang="ts">
import { computed, ref } from "vue";
import { useSortedLocale } from "@vizejs/composable/use-sorted-locale";

const locale = ref("de-DE");
const names = ["Zoe", "Ärla", "Anna", "Oskar", "Örjan"];
const { sorted } = useSortedLocale(
  names,
  computed(() => ({ locale: locale.value })),
);
</script>

<template>
  <div class="composable-example">
    <p>Sort names the way readers of each language expect.</p>
    <label>
      Sort for
      <select v-model="locale">
        <option value="de-DE">German</option>
        <option value="sv-SE">Swedish</option>
      </select>
    </label>
    <output aria-live="polite">{{ sorted.join(", ") }}</output>
  </div>
</template>
