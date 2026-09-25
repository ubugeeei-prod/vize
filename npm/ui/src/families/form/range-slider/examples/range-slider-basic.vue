<!-- Controlled two-thumb price range slider with labelled thumbs. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  RangeSlider,
  RangeSliderRange,
  RangeSliderThumb,
  RangeSliderTrack,
} from "../range-slider.ts";

const price = ref<readonly number[]>([20, 80]);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Price range</span>
    <RangeSlider v-model="price" name="price" :step="5" :aria-labelledby="labelId">
      <RangeSliderTrack>
        <RangeSliderRange />
        <RangeSliderThumb :index="0" aria-label="Minimum price" />
        <RangeSliderThumb :index="1" aria-label="Maximum price" />
      </RangeSliderTrack>
    </RangeSlider>
    <output>${{ price[0] }} – ${{ price[1] }}</output>
  </div>
</template>
