<!-- Controlled five-star rating built from native radios with a clearable value. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Rating } from "../rating.ts";

const score = ref<number | null>(3);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Rate this recipe</span>
    <Rating v-model="score" name="recipe-score" clearable :aria-labelledby="labelId">
      <template #item="{ active }">{{ active ? "★" : "☆" }}</template>
    </Rating>
    <output>{{ score === null ? "Not rated" : `${score} of 5` }}</output>
  </div>
</template>
