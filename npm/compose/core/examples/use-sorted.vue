<script setup lang="ts">
import { ref } from "vue";
import { useSorted } from "@vizejs/composable/use-sorted";

const scores = ref([
  { id: 1, value: 72 },
  { id: 2, value: 95 },
  { id: 3, value: 64 },
]);
const ranking = useSorted(scores, (left, right) => right.value - left.value);
const score = ref(80);
let nextId = 4;

function addScore() {
  scores.value = [...scores.value, { id: nextId, value: score.value }];
  nextId += 1;
}
</script>

<template>
  <div class="composable-example">
    <p>Show a leaderboard from scores entered in any order.</p>
    <label>
      New score
      <input v-model.number="score" type="number" min="0" max="100" />
    </label>
    <button type="button" @click="addScore">Add score</button>
    <ol aria-label="Leaderboard">
      <li v-for="entry in ranking" :key="entry.id">{{ entry.value }}</li>
    </ol>
    <output aria-live="polite"
      >Entered order: {{ scores.map((entry) => entry.value).join(", ") }}</output
    >
  </div>
</template>
