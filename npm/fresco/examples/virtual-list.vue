<script setup lang="ts">
import { ref } from "vue";

import { Box, ScrollBox, Text } from "../src/index.js";

const itemCount = 100_000;
const offset = ref(0);
</script>

<template>
  <Box flex-direction="column">
    <Text :bold="true">100k bounded virtual rows</Text>
    <ScrollBox v-model="offset" :item-count :viewport-size="12" :overscan="2" :focused="true">
      <template #default="{ index }">
        <Text :content="`row ${index}`" />
      </template>
      <template #end>
        <Text :dim="true" :content="`${offset + 1} / ${itemCount}`" />
      </template>
    </ScrollBox>
  </Box>
</template>
