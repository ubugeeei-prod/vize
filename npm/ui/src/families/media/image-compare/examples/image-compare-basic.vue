<!-- Before/after photo comparison with a keyboard-operable slider handle bound with v-model. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  ImageCompare,
  ImageCompareAfter,
  ImageCompareBefore,
  ImageCompareHandle,
  ImageCompareLabel,
} from "../image-compare.ts";

const position = ref(50);
</script>

<template>
  <ImageCompare v-model="position">
    <ImageCompareBefore>
      <img src="https://example.com/media/kitchen-before.jpg" alt="Kitchen before renovation" />
    </ImageCompareBefore>
    <ImageCompareAfter>
      <img src="https://example.com/media/kitchen-after.jpg" alt="Kitchen after renovation" />
    </ImageCompareAfter>
    <ImageCompareLabel side="before">Before</ImageCompareLabel>
    <ImageCompareLabel side="after">After</ImageCompareLabel>
    <ImageCompareHandle aria-label="Reveal renovated kitchen" />
  </ImageCompare>
  <output>{{ position }}% before</output>
</template>
