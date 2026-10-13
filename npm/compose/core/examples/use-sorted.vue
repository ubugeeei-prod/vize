<script setup lang="ts">
import { ref } from "vue";
import { useSorted } from "@vizejs/composable/use-sorted";

const scores = ref([72, 95, 64]);
const ranking = useSorted(scores, (left, right) => right - left);
const score = ref(80);

function addScore() {
  scores.value = [...scores.value, score.value];
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
      <li v-for="(value, index) in ranking" :key="index">{{ value }}</li>
    </ol>
    <output aria-live="polite">Entered order: {{ scores.join(", ") }}</output>
  </div>
</template>
