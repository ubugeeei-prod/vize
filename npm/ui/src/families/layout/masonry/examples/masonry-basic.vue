<!-- Photo wall balanced into three columns using estimated heights on the server. -->
<script setup lang="ts">
import { Masonry } from "../masonry.ts";

interface Photo {
  readonly id: string;
  readonly caption: string;
  readonly height: number;
}

const photos: readonly Photo[] = [
  { id: "dunes", caption: "Dunes at dawn", height: 180 },
  { id: "harbor", caption: "Harbor lights", height: 120 },
  { id: "forest", caption: "Forest trail", height: 240 },
  { id: "glacier", caption: "Glacier edge", height: 150 },
  { id: "market", caption: "Night market", height: 200 },
];

function estimate(photo: Photo): number {
  return photo.height;
}

function keyOf(photo: Photo): string {
  return photo.id;
}
</script>

<template>
  <Masonry :items="photos" :columns="3" :gap="12" :estimate-height="estimate" :get-key="keyOf">
    <template #item="{ item }">
      <figure>
        <figcaption>{{ item.caption }}</figcaption>
      </figure>
    </template>
  </Masonry>
</template>
