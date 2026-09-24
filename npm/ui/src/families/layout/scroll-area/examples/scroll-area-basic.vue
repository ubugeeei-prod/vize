<!-- Focusable, labelled scroll region with a fixed block size so its content overflows. -->
<script setup lang="ts">
import { useId } from "vue";

import { ScrollArea } from "../scroll-area.ts";

const titleId = useId();
const releases = ["2.4.0", "2.3.2", "2.3.1", "2.3.0", "2.2.0", "2.1.1", "2.1.0", "2.0.0"];
</script>

<template>
  <div>
    <h2 :id="titleId">Changelog</h2>
    <ScrollArea block-size="8rem" focusable :aria-labelledby="titleId">
      <ul>
        <li v-for="release in releases" :key="release">Version {{ release }}</li>
      </ul>
    </ScrollArea>
  </div>
</template>
