<script setup lang="ts">
import type { Rung } from "./ladder";
import type { StageId } from "./useDavinciLadder";

defineProps<{
  rungs: Rung[];
  selected: StageId;
}>();

const emit = defineEmits<{
  select: [StageId];
}>();
</script>

<template>
  <nav class="davinci-rail" aria-label="Compiler stages">
    <template v-for="rung in rungs" :key="rung.id">
      <button
        type="button"
        :class="[
          'davinci-station',
          { active: selected === rung.id, empty: rung.pages.length === 0 },
        ]"
        :aria-pressed="selected === rung.id"
        :data-stage="rung.id"
        @click="emit('select', rung.id)"
      >
        <span class="davinci-station-ordinal">{{ rung.ordinal }}</span>
        <span class="davinci-station-name">{{ rung.name }}</span>
        <span class="davinci-station-facts">{{ rung.facts.join(", ") || "no page" }}</span>
      </button>
      <span class="davinci-pounce" aria-hidden="true"></span>
    </template>
    <button
      type="button"
      :class="['davinci-station', { active: selected === 's4' }]"
      :aria-pressed="selected === 's4'"
      data-stage="s4"
      @click="emit('select', 's4')"
    >
      <span class="davinci-station-ordinal">S4</span>
      <span class="davinci-station-name">Output</span>
      <span class="davinci-station-facts">DOM, Vapor, SSR</span>
    </button>
  </nav>
</template>
