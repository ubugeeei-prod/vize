<script setup lang="ts">
import { computed, ref } from "vue";
import { useDisplayNames } from "@vizejs/composable/use-display-names";

const locale = ref("en");
const code = ref("ja");
const { name } = useDisplayNames(
  code,
  computed(() => ({ type: "language" as const, locale: locale.value })),
);
</script>

<template>
  <div class="composable-example">
    <p>Name a language in the reader's own language instead of showing its code.</p>
    <label>
      Language code
      <select v-model="code">
        <option value="ja">ja</option>
        <option value="fr">fr</option>
        <option value="pt-BR">pt-BR</option>
      </select>
    </label>
    <label>
      Reader locale
      <select v-model="locale">
        <option value="en">English</option>
        <option value="de">German</option>
        <option value="ja">Japanese</option>
      </select>
    </label>
    <output aria-live="polite">{{ name }}</output>
  </div>
</template>
