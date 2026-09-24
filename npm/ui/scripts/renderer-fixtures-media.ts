export const mediaRendererFixtures = [
  {
    filename: "CarouselConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  CarouselAutoplayToggle,
  CarouselIndicator,
  CarouselIndicatorGroup,
  CarouselNext,
  CarouselPrevious,
  CarouselRoot,
  CarouselSlide,
  CarouselViewport,
} from "./families/media/carousel/carousel.ts";

const slides = ["Lamp", "Chair", "Desk"];
const index = ref(0);
const playing = ref(true);
</script>

<template>
  <CarouselRoot
    v-model="index"
    v-model:playing="playing"
    :slide-count="slides.length"
    aria-label="Featured products"
    loop
  >
    <template #default="{ autoplay }">
      <CarouselAutoplayToggle>
        {{ autoplay === "stopped" ? "Start slide rotation" : "Stop slide rotation" }}
      </CarouselAutoplayToggle>
      <CarouselPrevious aria-label="Previous slide" />
      <CarouselNext aria-label="Next slide" />
      <CarouselViewport>
        <CarouselSlide v-for="(name, slideIndex) in slides" :key="name" :index="slideIndex">
          <template #default="{ state }">{{ name }} {{ state }}</template>
        </CarouselSlide>
      </CarouselViewport>
      <CarouselIndicatorGroup aria-label="Choose slide">
        <CarouselIndicator v-for="(name, slideIndex) in slides" :key="name" :index="slideIndex" />
      </CarouselIndicatorGroup>
    </template>
  </CarouselRoot>
</template>
`,
  },
  {
    filename: "ImageConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  ImageContent,
  ImageFallback,
  ImagePlaceholder,
  ImageRoot,
} from "./families/media/image/image.ts";
import type { ImageStatus } from "./families/media/image/image.ts";

function onStatusChange(status: ImageStatus): void {
  void status;
}
</script>

<template>
  <ImageRoot :src="['/cover.avif', '/cover.jpg']" defer @status-change="onStatusChange">
    <ImageContent alt="Album cover" :width="320" :height="320" />
    <ImagePlaceholder :delay="150">
      <template #default="{ status }">Loading {{ status }}</template>
    </ImagePlaceholder>
    <ImageFallback>AC</ImageFallback>
  </ImageRoot>
</template>
`,
  },
  {
    filename: "InfiniteScrollConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  InfiniteScrollItem,
  InfiniteScrollLoadMore,
  InfiniteScrollRoot,
  InfiniteScrollSentinel,
  InfiniteScrollStatus,
} from "./families/data/infinite-scroll/infinite-scroll.ts";

const items = ref(["Alpha", "Bravo"]);
const hasMore = ref(true);

async function loadPage(): Promise<void> {
  items.value = [...items.value, "Charlie"];
  hasMore.value = false;
}
</script>

<template>
  <InfiniteScrollRoot :loader="loadPage" :has-more="hasMore" feed aria-label="Results">
    <InfiniteScrollItem v-for="(item, index) in items" :key="item" :index="index">
      {{ item }}
    </InfiniteScrollItem>
    <InfiniteScrollSentinel />
    <InfiniteScrollLoadMore>
      <template #default="{ state }">{{ state === "error" ? "Retry" : "Load more" }}</template>
    </InfiniteScrollLoadMore>
    <InfiniteScrollStatus>
      <template #default="{ state }">{{ state === "loading" ? "Loading more results" : "" }}</template>
    </InfiniteScrollStatus>
  </InfiniteScrollRoot>
</template>
`,
  },
] as const;
