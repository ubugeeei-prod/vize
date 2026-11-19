<script setup lang="ts">
import "./stage-rail-styles.vue";
import type { Rung } from "./ladder";
import type { StageId } from "./useStageLadder";

const { rungs, selected } = defineProps<{
  rungs: Rung[];
  selected: StageId;
}>();

const emit = defineEmits<{
  select: [StageId];
}>();
</script>

<template>
  <nav class="level-rail" aria-label="Compiler stages">
    <template v-for="rung in rungs" :key="rung.id">
      <button
        type="button"
        :class="['level-station', { active: selected === rung.id, empty: rung.pages.length === 0 }]"
        :aria-pressed="selected === rung.id"
        :data-stage="rung.id"
        @click="() => emit('select', rung.id)"
      >
        <span class="level-station-ordinal">{{ rung.ordinal }}</span>
        <span class="level-station-name">{{ rung.name }}</span>
        <span class="level-station-facts">{{ rung.facts.join(", ") || "no page" }}</span>
      </button>
      <span class="level-pounce" aria-hidden="true"></span>
    </template>
    <button
      type="button"
      :class="['level-station', { active: selected === 'l4' }]"
      :aria-pressed="selected === 'l4'"
      data-stage="l4"
      @click="() => emit('select', 'l4')"
    >
      <span class="level-station-ordinal">L4</span>
      <span class="level-station-name">Emit</span>
      <span class="level-station-facts">Backend + SFC module</span>
    </button>
  </nav>
</template>
