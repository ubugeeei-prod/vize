<!-- Customer-quote carousel bound with v-model, with previous/next buttons and slide picker tabs. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  CarouselIndicator,
  CarouselIndicatorGroup,
  CarouselNext,
  CarouselPrevious,
  CarouselRoot,
  CarouselSlide,
  CarouselViewport,
} from "../carousel.ts";

const quotes = [
  { author: "Maya, platform lead", text: "Our build times dropped by half in a week." },
  { author: "Kenji, design engineer", text: "Headless parts let us keep our own design system." },
  {
    author: "Lena, accessibility lead",
    text: "Keyboard and screen reader support worked out of the box.",
  },
];
const active = ref(0);
</script>

<template>
  <CarouselRoot v-model="active" :slide-count="quotes.length" aria-label="Customer quotes">
    <CarouselPrevious>Previous quote</CarouselPrevious>
    <CarouselNext>Next quote</CarouselNext>
    <CarouselViewport>
      <CarouselSlide v-for="(quote, index) in quotes" :key="quote.author" :index>
        <blockquote>{{ quote.text }}</blockquote>
        <p>{{ quote.author }}</p>
      </CarouselSlide>
    </CarouselViewport>
    <CarouselIndicatorGroup aria-label="Choose quote">
      <CarouselIndicator v-for="(quote, index) in quotes" :key="quote.author" :index />
    </CarouselIndicatorGroup>
  </CarouselRoot>
</template>
